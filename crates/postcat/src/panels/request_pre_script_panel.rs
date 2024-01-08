use std::collections::BTreeMap;
use std::ops::Add;

use egui::Ui;

use crate::data::environment::EnvironmentItemValue;
use crate::data::http::Request;
use crate::data::workspace_data::WorkspaceData;
use crate::operation::Operation;
use crate::panels::{DataView, HORIZONTAL_GAP};
use crate::script::script::{Context, ScriptScope};
use crate::windows::test_script_windows::TestScriptWindows;

#[derive(Default)]
pub struct RequestPreScriptPanel {
    test_script_windows: TestScriptWindows,
}

impl RequestPreScriptPanel {
    pub(crate) fn set_and_render(
        &mut self,
        ui: &mut Ui,
        operation: &mut Operation,
        workspace_data: &mut WorkspaceData,
        mut script: String,
        parent_script: Option<ScriptScope>,
        request: Request,
        envs: BTreeMap<String, EnvironmentItemValue>,
        id: String,
    ) -> String {
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                egui_extras::syntax_highlighting::highlight(ui.ctx(), &theme, string, "js");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        ui.horizontal(|ui| {
            egui::SidePanel::right("pre_request_right_".to_string() + id.as_str())
                .resizable(true)
                .min_width(300.0)
                .show_separator_line(false)
                .show_inside(ui, |ui| {
                    ui.label("Pre-request scripts are written in JavaScript， and are run before the request is sent.");
                    if ui.link("Test").clicked() {
                        let script_scope = ScriptScope {
                            script: script.clone(),
                            scope: "request".to_string(),
                        };
                        let mut script_scopes = Vec::new();
                        if let Some(parent_script_scope) = parent_script {
                            script_scopes.push(parent_script_scope);
                        }
                        script_scopes.push(script_scope);
                        let context = Context {
                            scope_name: "request".to_string(),
                            request: request.clone(),
                            envs,
                            ..Default::default()
                        };
                        self.test_script_windows.open(script_scopes, context);
                    }
                    ui.separator();
                    ui.strong("SNIPPETS");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if ui.link("Log info message").clicked() {
                            script = script.clone().add("\nconsole.log(\"info1\",\"info2\");");
                        }
                        if ui.link("Log warn message").clicked() {
                            script = script.clone().add("\nconsole.warn(\"info1\",\"info2\");");
                        }
                        if ui.link("Log error message").clicked() {
                            script = script.clone().add("\nconsole.error(\"error1\",\"error2\");");
                        }
                        if ui.link("Get a variable").clicked() {
                            script = script.clone().add("\npostcat.get_env(\"variable_key\");");
                        }
                        if ui.link("Set a variable").clicked() {
                            script = script.clone().add("\npostcat.set_env(\"variable_key\",\"variable_value\");");
                        }
                        if ui.link("Add a header").clicked() {
                            script = script.clone().add("\npostcat.add_header(\"header_key\",\"header_value\");");
                        }
                        if ui.link("Add a params").clicked() {
                            script = script.clone().add("\npostcat.add_params(\"params_key\",\"params_value\");");
                        }
                        if ui.link("Get a shared").clicked() {
                            script = script.clone().add("\npostcat.get_shared(\"shared_key\");");
                        }
                        if ui.link("Set a shared").clicked() {
                            script = script.clone().add("\npostcat.set_shared(\"shared_key\",\"shared_value\");");
                        }
                        if ui.link("Fetch a http request").clicked() {
                            script = script.clone().add(
                                r#"let request = {
    "method":"post",
    "url":"http://www.httpbin.org/post",
    "headers":[{
        "name":"name",
        "value":"value"
    }],
    "body":"body"
}
let response = await fetch(request);
console.log(response)"#)
                        }
                    });
                });
            egui::SidePanel::left("pre_request_left_".to_string() + id.as_str())
                .resizable(true)
                .min_width(ui.available_width() - HORIZONTAL_GAP * 2.0)
                .show_inside(ui, |ui| {
                    ui.push_id("pre_request_script", |ui| {
                        egui::ScrollArea::vertical()
                            .min_scrolled_height(300.0)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut script)
                                        .font(egui::TextStyle::Monospace) // for cursor height
                                        .code_editor()
                                        .desired_rows(10)
                                        .lock_focus(true)
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter),
                                );
                            });
                    });
                });
        });
        self.test_script_windows.set_and_render(ui, operation);
        script
    }
}
