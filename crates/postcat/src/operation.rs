use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use eframe::emath::Align2;
use egui_toast::Toasts;
use poll_promise::Promise;
use reqwest::blocking::{multipart, Client};
use reqwest::header::CONTENT_TYPE;
use reqwest::Method;

use crate::data::collections::{Collection, CollectionFolder};
use crate::data::environment::EnvironmentItemValue;
use crate::data::http::{
    BodyRawType, BodyType, Header, HttpBody, HttpRecord, LockWith, MultipartDataType,
};
use crate::data::logger::Logger;
use crate::data::{http, test};
use crate::script::script::{Context, JsResponse, ScriptRuntime, ScriptScope};
use crate::utils;

pub struct Operation {
    rest_sender: RestSender,
    open_windows: OpenWindows,
    lock_ui: HashMap<String, bool>,
    script_runtime: ScriptRuntime,
    toasts: Toasts,
}

impl Default for Operation {
    fn default() -> Self {
        Operation {
            rest_sender: Default::default(),
            open_windows: Default::default(),
            lock_ui: Default::default(),
            script_runtime: Default::default(),
            toasts: Toasts::default()
                .anchor(Align2::RIGHT_BOTTOM, (-10.0, -10.0))
                .direction(egui::Direction::BottomUp),
        }
    }
}

impl Operation {
    pub fn send_with_script(
        &self,
        request: http::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
        pre_request_scripts: Vec<ScriptScope>,
        test_scripts: Vec<ScriptScope>,
        client: Client,
    ) -> Promise<Result<(http::Request, http::Response, test::TestResult), String>> {
        let mut logger = Logger::default();
        Promise::spawn_thread("send_with_script", move || {
            let mut pre_request_context_result = Ok(Context {
                scope_name: "".to_string(),
                request: request.clone(),
                envs: envs.clone(),
                ..Default::default()
            });
            if pre_request_scripts.len() > 0 {
                pre_request_context_result = ScriptRuntime::run_block_many(
                    pre_request_scripts,
                    Context {
                        scope_name: "".to_string(),
                        request: request.clone(),
                        envs: envs.clone(),
                        ..Default::default()
                    },
                );
            }
            match pre_request_context_result {
                Ok(pre_request_context) => {
                    for log in pre_request_context.logger.logs.iter() {
                        logger.logs.push(log.clone());
                    }
                    let build_request = RestSender::build_request(
                        pre_request_context.request.clone(),
                        pre_request_context.envs.clone(),
                    );
                    logger.add_info(
                        "fetch".to_string(),
                        format!("start fetch request: {:?}", build_request),
                    );
                    match RestSender::reqwest_block_send(build_request, client) {
                        Ok((after_request, response)) => {
                            let mut after_response = response;
                            logger.add_info(
                                "fetch".to_string(),
                                format!("get response: {:?}", after_response),
                            );
                            after_response.logger = logger;
                            let mut test_result: test::TestResult = Default::default();
                            let mut test_context = pre_request_context.clone();
                            test_context.response =
                                JsResponse::from_data_response(after_response.clone());
                            if test_scripts.len() > 0 {
                                pre_request_context_result =
                                    ScriptRuntime::run_block_many(test_scripts, test_context);
                                match pre_request_context_result {
                                    Ok(test_context) => {
                                        for log in test_context.logger.logs.iter() {
                                            after_response.logger.logs.push(log.clone());
                                        }
                                        test_result = test_context.test_result.clone();
                                    }
                                    Err(e) => {
                                        return Err(e.to_string());
                                    }
                                }
                            }
                            Ok((after_request, after_response, test_result))
                        }
                        Err(e) => Err(e.to_string()),
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        })
    }
    pub fn lock_ui(&mut self, key: String, bool: bool) {
        self.lock_ui.insert(key, bool);
    }
    pub fn get_ui_lock(&self) -> bool {
        let mut result = false;
        for (_, lock) in self.lock_ui.iter() {
            result = result || (lock.clone());
        }
        result
    }
    pub fn rest_sender(&self) -> &RestSender {
        &self.rest_sender
    }
    pub fn open_windows(&mut self) -> &mut OpenWindows {
        &mut self.open_windows
    }
    pub fn script_runtime(&self) -> &ScriptRuntime {
        &self.script_runtime
    }
    pub fn toasts(&mut self) -> &mut Toasts {
        &mut self.toasts
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RestSender {}

impl RestSender {
    pub fn reqwest_block_send(
        request: http::Request,
        client: Client,
    ) -> reqwest::Result<(http::Request, http::Response)> {
        let reqwest_request = Self::build_reqwest_request(request.clone())?;
        let mut new_request = request.clone();
        for (hn, hv) in reqwest_request.headers().iter() {
            if new_request
                .headers
                .iter()
                .find(|h| {
                    h.key.to_lowercase() == hn.to_string().to_lowercase()
                        && h.value == hv.to_str().unwrap()
                })
                .is_none()
            {
                new_request.headers.push(Header {
                    key: hn.to_string(),
                    value: hv.to_str().unwrap().to_string(),
                    desc: "auto gen".to_string(),
                    enable: true,
                    lock_with: LockWith::LockWithAuto,
                })
            }
        }
        let start_time = Instant::now();
        let reqwest_response = client.execute(reqwest_request)?;
        let total_time = start_time.elapsed();
        Ok((
            new_request,
            http::Response {
                headers: Header::new_from_map(reqwest_response.headers()),
                status: reqwest_response.status().as_u16(),
                status_text: reqwest_response.status().to_string(),
                elapsed_time: total_time.as_millis(),
                logger: Logger::default(),
                body: Arc::new(HttpBody::new(reqwest_response.bytes()?.to_vec())),
            },
        ))
    }

    pub fn build_reqwest_request(
        request: http::Request,
    ) -> reqwest::Result<reqwest::blocking::Request> {
        let client = Client::new();
        let method = Method::from_str(request.method.to_string().to_uppercase().as_str()).unwrap();
        let mut builder = client.request(method, request.base_url);
        for header in request.headers.iter().filter(|h| h.enable) {
            builder = builder.header(header.key.clone(), header.value.clone());
        }
        let query: Vec<(String, String)> = request
            .params
            .iter()
            .filter(|q| q.enable)
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect();
        builder = builder.query(&query);
        match request.body.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {
                let mut form = multipart::Form::new();
                for md in request.body.body_form_data.iter().filter(|md| md.enable) {
                    match md.data_type {
                        MultipartDataType::File => {
                            form = form
                                .file(md.key.clone(), Path::new(md.value.as_str()).to_path_buf())
                                .unwrap();
                        }
                        MultipartDataType::Text => {
                            form = form.text(md.key.clone(), md.value.clone());
                        }
                    }
                }
                builder = builder.multipart(form);
            }
            BodyType::X_WWW_FROM_URLENCODED => {
                let mut params = HashMap::new();
                for md in request.body.body_xxx_form.iter().filter(|md| md.enable) {
                    params.insert(md.key.clone(), md.value.clone());
                }
                builder = builder.form(&params);
            }
            BodyType::RAW => match request.body.body_raw_type {
                BodyRawType::TEXT => {
                    builder = builder.header(CONTENT_TYPE, "text/plain");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::JSON => {
                    builder = builder.header(CONTENT_TYPE, "application/json");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::HTML => {
                    builder = builder.header(CONTENT_TYPE, "text/html");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::XML => {
                    builder = builder.header(CONTENT_TYPE, "application/xml");
                    builder = builder.body(request.body.body_str);
                }
                BodyRawType::JavaScript => {
                    builder = builder.header(CONTENT_TYPE, "application/javascript");
                    builder = builder.body(request.body.body_str);
                }
            },
            BodyType::BINARY => {
                let path = Path::new(request.body.body_file.as_str());
                let content_type = mime_guess::from_path(path);
                builder = builder.header(
                    CONTENT_TYPE,
                    content_type.first_or_octet_stream().to_string(),
                );
                let file_name = path.file_name().and_then(|filename| filename.to_str());
                let mut file =
                    File::open(path).expect(format!("open {:?} error", file_name).as_str());
                let mut inner: Vec<u8> = vec![];
                io::copy(&mut file, &mut inner).expect("add_stream io copy error");
                builder = builder.body(inner);
            }
        }
        builder.build()
    }

    fn build_request(
        request: http::Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
    ) -> http::Request {
        let mut build_request = request.clone();
        if !build_request.base_url.starts_with("http://")
            && !build_request.base_url.starts_with("https://")
        {
            build_request.base_url = "http://".to_string() + build_request.base_url.as_str();
        }
        build_request.headers = Self::build_header(request.headers.clone(), &envs);
        build_request.body.body_str =
            utils::replace_variable(build_request.body.body_str, envs.clone());
        for md in build_request.body.body_xxx_form.iter_mut() {
            md.key = utils::replace_variable(md.key.clone(), envs.clone());
            md.value = utils::replace_variable(md.value.clone(), envs.clone());
        }
        for md in build_request.body.body_form_data.iter_mut() {
            md.key = utils::replace_variable(md.key.clone(), envs.clone());
            md.value = utils::replace_variable(md.value.clone(), envs.clone());
        }
        build_request
    }

    fn build_header(
        headers: Vec<Header>,
        envs: &BTreeMap<String, EnvironmentItemValue>,
    ) -> Vec<Header> {
        headers
            .iter()
            .filter(|h| h.enable)
            .map(|h| Header {
                key: h.key.clone(),
                value: utils::replace_variable(h.value.clone(), envs.clone()),
                desc: h.desc.clone(),
                enable: h.enable,
                lock_with: h.lock_with.clone(),
            })
            .collect()
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct OpenWindows {
    pub save_opened: bool,
    pub edit: bool,
    pub collection_opened: bool,
    pub folder_opened: bool,
    pub cookies_opened: bool,
    pub http_record: HttpRecord,
    pub default_path: Option<String>,
    pub collection: Option<Collection>,
    pub parent_folder: Rc<RefCell<CollectionFolder>>,
    pub folder: Option<Rc<RefCell<CollectionFolder>>>,
    pub crt_id: String,
    pub save_crt_opened: bool,
}

impl OpenWindows {
    pub fn open_crt_save(&mut self, crt_id: String) {
        self.crt_id = crt_id;
        self.save_crt_opened = true;
    }
    pub fn open_save(&mut self, http_record: HttpRecord, default_path: Option<String>) {
        self.http_record = http_record;
        self.default_path = default_path;
        self.save_opened = true;
        self.edit = false;
    }
    pub fn open_edit(&mut self, http_record: HttpRecord, default_path: String) {
        self.http_record = http_record;
        self.default_path = Some(default_path);
        self.save_opened = true;
        self.edit = true
    }
    pub fn open_collection(&mut self, collection: Option<Collection>) {
        self.collection = collection;
        self.collection_opened = true;
    }
    pub fn open_folder(
        &mut self,
        collection: Collection,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Option<Rc<RefCell<CollectionFolder>>>,
    ) {
        self.collection = Some(collection);
        self.parent_folder = parent_folder;
        self.folder = folder;
        self.folder_opened = true;
    }

    pub fn open_cookies(&mut self) {
        self.cookies_opened = true
    }
}
