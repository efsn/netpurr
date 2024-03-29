use std::collections::BTreeMap;

use eframe::emath::Align;
use egui::{Button, Checkbox, Layout, TextBuffer, TextEdit, Widget};
use egui_extras::{Column, TableBody, TableBuilder};

use netpurr_core::data::central_request_data::CentralRequestItem;
use netpurr_core::data::environment::EnvironmentItemValue;
use netpurr_core::data::http::MultipartData;
use netpurr_core::data::workspace_data::WorkspaceData;

use crate::widgets::highlight_template::HighlightTemplateSinglelineBuilder;

#[derive(Default)]
pub struct RequestBodyXXXFormPanel {
    new_form: MultipartData,
}

impl RequestBodyXXXFormPanel {
    pub fn set_and_render(
        &mut self,
        ui: &mut egui::Ui,
        workspace_data: &mut WorkspaceData,
        crt_id: String,
    ) {
        let envs = workspace_data.get_crt_envs(crt_id.clone());
        workspace_data.must_get_mut_crt(crt_id.clone(), |crt| {
            let mut delete_index = None;
            let table = TableBuilder::new(ui)
                .resizable(false)
                .cell_layout(Layout::left_to_right(Align::Center))
                .column(Column::auto())
                .column(Column::exact(20.0))
                .column(Column::initial(200.0).range(40.0..=300.0))
                .column(Column::initial(200.0).range(40.0..=300.0))
                .column(Column::remainder())
                .max_scroll_height(100.0);
            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("KEY");
                    });
                    header.col(|ui| {
                        ui.strong("VALUE");
                    });
                    header.col(|ui| {
                        ui.strong("DESCRIPTION");
                    });
                })
                .body(|mut body| {
                    delete_index = self.build_body(crt, &envs, &mut body);
                    self.build_new_body(envs, body);
                });
            if delete_index.is_some() {
                crt.record
                    .must_get_mut_rest()
                    .request
                    .body
                    .body_xxx_form
                    .remove(delete_index.unwrap());
            }
            if self.new_form.key != "" || self.new_form.value != "" || self.new_form.desc != "" {
                self.new_form.enable = true;
                crt.record
                    .must_get_mut_rest()
                    .request
                    .body
                    .body_xxx_form
                    .push(self.new_form.clone());
                self.new_form.key = "".to_string();
                self.new_form.value = "".to_string();
                self.new_form.desc = "".to_string();
                self.new_form.enable = false;
            }
        });
    }
}

impl RequestBodyXXXFormPanel {
    fn build_body(
        &self,
        data: &mut CentralRequestItem,
        envs: &BTreeMap<String, EnvironmentItemValue>,
        mut body: &mut TableBody,
    ) -> Option<usize> {
        let mut delete_index: Option<usize> = None;
        for (index, param) in data
            .record
            .must_get_mut_rest()
            .request
            .body
            .body_xxx_form
            .iter_mut()
            .enumerate()
        {
            body.row(18.0, |mut row| {
                row.col(|ui| {
                    ui.checkbox(&mut param.enable, "");
                });
                row.col(|ui| {
                    if ui.button("x").clicked() {
                        delete_index = Some(index)
                    }
                });
                row.col(|ui| {
                    HighlightTemplateSinglelineBuilder::default()
                        .envs(envs.clone())
                        .all_space(false)
                        .build(
                            "request_body_key_".to_string() + index.to_string().as_str(),
                            &mut param.key,
                        )
                        .ui(ui);
                });
                row.col(|ui| {
                    HighlightTemplateSinglelineBuilder::default()
                        .envs(envs.clone())
                        .all_space(false)
                        .build(
                            "request_body_value_".to_string() + index.to_string().as_str(),
                            &mut param.value,
                        )
                        .ui(ui);
                });
                row.col(|ui| {
                    TextEdit::singleline(&mut param.desc)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
            });
        }
        delete_index
    }

    fn build_new_body(
        &mut self,
        envs: BTreeMap<String, EnvironmentItemValue>,
        mut body: TableBody,
    ) {
        body.row(18.0, |mut row| {
            row.col(|ui| {
                ui.add_enabled(false, Checkbox::new(&mut self.new_form.enable, ""));
            });
            row.col(|ui| {
                ui.add_enabled(false, Button::new("x"));
            });
            row.col(|ui| {
                HighlightTemplateSinglelineBuilder::default()
                    .envs(envs.clone())
                    .all_space(false)
                    .build("request_body_key_new".to_string(), &mut self.new_form.key)
                    .ui(ui);
            });
            row.col(|ui| {
                HighlightTemplateSinglelineBuilder::default()
                    .envs(envs)
                    .all_space(false)
                    .build(
                        "request_body_value_new".to_string(),
                        &mut self.new_form.value,
                    )
                    .ui(ui);
            });
            row.col(|ui| {
                TextEdit::singleline(&mut self.new_form.desc)
                    .desired_width(f32::INFINITY)
                    .ui(ui);
            });
        });
    }
}
