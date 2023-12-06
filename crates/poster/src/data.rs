use chrono::{DateTime, NaiveDate, Utc};
use eframe::epaint::ahash::HashMap;
use egui::TextBuffer;
use poll_promise::Promise;
use strum_macros::{Display, EnumIter, EnumString};
use uuid::Uuid;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct AppData {
    pub rest_sender: RestSender,
    pub central_request_data_list: CentralRequestDataList,
    pub history_data_list: HistoryDataList,
}

impl AppData {
    pub fn fake(&mut self) {
        self.history_data_list.date_group.insert(
            NaiveDate::default(),
            DateGroupHistoryList {
                history_list: vec![HistoryRestItem {
                    id: Uuid::new_v4().to_string(),
                    record_date: Default::default(),
                    rest: HttpRecord {
                        request: Request {
                            method: Default::default(),
                            base_url: "http://www.baidu.com".to_string(),
                            params: vec![],
                            headers: Default::default(),
                            body: vec![],
                            body_str: "".to_string(),
                            body_type: Default::default(),
                            body_raw_type: Default::default(),
                            body_form_data: vec![],
                            body_xxx_form: vec![],
                        },
                        response: Default::default(),
                    },
                }],
            },
        );
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct RestSender {}

impl RestSender {
    pub fn send(&mut self, rest: &mut HttpRecord) -> Promise<ehttp::Result<ehttp::Response>> {
        let (sender, promise) = Promise::new();
        if !rest.request.base_url.starts_with("http://")
            && !rest.request.base_url.starts_with("https://")
        {
            rest.request.base_url = "http://".to_string() + rest.request.base_url.as_str();
        }
        let request = ehttp::Request {
            method: rest.request.method.to_string(),
            url: rest.request.base_url.to_string(),
            body: vec![],
            headers: Default::default(),
        };
        ehttp::fetch(request, move |response| {
            sender.send(response);
        });
        return promise;
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestDataList {
    pub select_id: Option<String>,
    pub data_list: Vec<CentralRequestItem>,
    pub data_map: HashMap<String, CentralRequestItem>,
}

impl CentralRequestDataList {
    pub fn add_new(&mut self) {
        let crt = CentralRequestItem {
            id: Uuid::new_v4().to_string(),
            rest: Default::default(),
        };
        self.add_crt(crt.clone());
        self.select(crt.id.clone())
    }
    pub fn select(&mut self, id: String) {
        self.select_id = Some(id)
    }
    pub fn add_crt(&mut self, crt: CentralRequestItem) {
        if !self.data_map.contains_key(crt.id.as_str()) {
            self.data_map.insert(crt.id.clone(), crt.clone());
            self.data_list.push(crt.clone())
        }
        self.select(crt.id.clone())
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct CentralRequestItem {
    pub id: String,
    pub rest: HttpRecord,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct HistoryDataList {
    pub date_group: HashMap<NaiveDate, DateGroupHistoryList>,
}

impl HistoryDataList {
    pub fn record(&mut self, rest: HttpRecord) {
        let today = chrono::Local::now().naive_local().date();
        if !self.date_group.contains_key(&today) {
            self.date_group.insert(
                today,
                DateGroupHistoryList {
                    history_list: vec![],
                },
            );
        }
        self.date_group
            .get_mut(&today)
            .unwrap()
            .history_list
            .push(HistoryRestItem {
                id: Uuid::new_v4().to_string(),
                record_date: chrono::Local::now().with_timezone(&Utc),
                rest,
            });
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct DateGroupHistoryList {
    pub history_list: Vec<HistoryRestItem>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct HistoryRestItem {
    pub id: String,
    pub record_date: DateTime<Utc>,
    pub rest: HttpRecord,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct HttpRecord {
    pub request: Request,
    pub response: Response,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Request {
    pub method: Method,
    pub base_url: String,
    pub params: Vec<QueryParam>,
    pub headers: Vec<Header>,
    pub body: Vec<u8>,
    pub body_str: String,
    pub body_type: BodyType,
    pub body_raw_type: BodyRawType,
    pub body_form_data: Vec<FormData>,
    pub body_xxx_form: Vec<FormData>,
}

impl HttpRecord {
    pub fn sync(&mut self) {
        match self.request.body_type {
            BodyType::NONE => {}
            BodyType::FROM_DATA => {}
            BodyType::X_WWW_FROM_URLENCODED => {}
            BodyType::RAW => match self.request.body_raw_type {
                BodyRawType::TEXT => self.set_content_type("text/plain".to_string()),
                BodyRawType::JSON => self.set_content_type("application/json".to_string()),
                BodyRawType::HTML => self.set_content_type("text/html".to_string()),
                BodyRawType::XML => self.set_content_type("application/xml".to_string()),
                BodyRawType::JavaScript => {
                    self.set_content_type("application/javascript".to_string())
                }
            },
            BodyType::BINARY => {}
        }
    }

    pub fn set_content_type(&mut self, value: String) {
        let mut need_add = false;
        let mut find = false;
        for (index, header) in self.request.headers.clone().iter().enumerate() {
            if header.key == "content-type" {
                find = true;
                if !header.value.contains(value.as_str()) {
                    need_add = true;
                    self.request.headers.remove(index);
                }
            }
        }
        if !find || need_add {
            self.request.headers.push(Header {
                key: "content-type".to_string(),
                value,
                desc: "".to_string(),
                enable: true,
            });
        }
    }
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq, Eq, Debug)]
pub enum BodyRawType {
    TEXT,
    JSON,
    HTML,
    XML,
    JavaScript,
}

impl Default for BodyRawType {
    fn default() -> Self {
        BodyRawType::JSON
    }
}

#[derive(Clone, EnumIter, EnumString, Display, PartialEq, Eq, Debug)]
pub enum BodyType {
    NONE,
    FROM_DATA,
    X_WWW_FROM_URLENCODED,
    RAW,
    BINARY,
}

impl Default for BodyType {
    fn default() -> Self {
        BodyType::NONE
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct FormData {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
    pub desc: String,
    pub enable: bool,
}

impl Header {
    pub fn new_from_tuple(headers: Vec<(String, String)>) -> Vec<Header> {
        let mut result = vec![];
        for (key, value) in headers {
            result.push(Header {
                key,
                value,
                desc: "".to_string(),
                enable: true,
            })
        }
        result
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct Response {
    pub body: Vec<u8>,
    pub headers: Vec<Header>,
    pub url: String,
    pub ok: bool,
    pub status: u16,
    pub status_text: String,
}

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: String,
    pub max_age: String,
    pub http_only: bool,
    pub secure: bool,
}

impl Response {
    //BAIDUID=67147D03A8E2F75F66619A1CFADFAAF2:FG=1; expires=Thu, 31-Dec-37 23:55:55 GMT; max-age=2147483647; path=/; domain=.baidu.com
    pub fn get_cookies(&self) -> Vec<Cookie> {
        self.headers
            .iter()
            .filter(|h| h.key.starts_with("set-cookie"))
            .map(|h| {
                let mut cookie = Cookie {
                    name: "".to_string(),
                    value: "".to_string(),
                    domain: "".to_string(),
                    path: "".to_string(),
                    expires: "".to_string(),
                    max_age: "".to_string(),
                    http_only: false,
                    secure: false,
                };
                let s = h.value.split(";");
                for x in s {
                    let one: Vec<&str> = x.splitn(2, "=").collect();
                    match one[0].trim() {
                        "expires" => cookie.expires = one[1].to_string(),
                        "path" => cookie.path = one[1].to_string(),
                        "domain" => cookie.domain = one[1].to_string(),
                        "max-age" => cookie.max_age = one[1].to_string(),
                        "secure" => cookie.secure = true,
                        "httponly" => cookie.http_only = true,
                        _ => {
                            cookie.value = one[1].to_string();
                            cookie.name = one[0].to_string()
                        }
                    }
                }
                cookie
            })
            .collect()
    }
}

#[derive(Debug, Display, PartialEq, EnumString, EnumIter, Clone, Eq)]
pub enum Method {
    POST,
    GET,
    PUT,
    PATCH,
    DELETE,
    COPY,
    HEAD,
    OPTIONS,
    LINK,
    UNLINK,
    PURGE,
    LOCK,
    UNLOCK,
    PROPFIND,
    VIEW,
}

impl Default for Method {
    fn default() -> Self {
        Method::GET
    }
}
