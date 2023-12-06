use eframe::emath::Align;
use egui::{Button, Checkbox, Layout, TextBuffer, TextEdit, Ui, Widget};
use egui_extras::{Column, TableBuilder};
use strum::IntoEnumIterator;

use crate::data::{AppData, BodyRawType, BodyType, FormData, QueryParam};
use crate::panels::{DataView, VERTICAL_GAP};

#[derive(Default)]
pub struct RequestBodyFormDataPanel {
    new_form: FormData,
}

impl DataView for RequestBodyFormDataPanel {
    type CursorType = String;
    fn set_and_render(&mut self, app_data: &mut AppData, cursor: Self::CursorType, ui: &mut Ui) {
        let data = app_data
            .central_request_data_list
            .data_map
            .get_mut(cursor.as_str())
            .unwrap();
        ui.label("Query Params");
        let mut delete_index = None;
        let table = TableBuilder::new(ui)
            .resizable(false)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto())
            .column(Column::exact(20.0))
            .column(Column::initial(200.0).range(40.0..=300.0))
            .column(Column::initial(200.0).range(40.0..=300.0))
            .column(Column::remainder())
            .min_scrolled_height(0.0);
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
                for (index, param) in data.rest.request.body_form_data.iter_mut().enumerate() {
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
                            ui.text_edit_singleline(&mut param.key);
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut param.value);
                        });
                        row.col(|ui| {
                            TextEdit::singleline(&mut param.desc)
                                .desired_width(f32::INFINITY)
                                .ui(ui);
                        });
                    });
                }
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.add_enabled(false, Checkbox::new(&mut self.new_form.enable, ""));
                    });
                    row.col(|ui| {
                        ui.add_enabled(false, Button::new("x"));
                    });
                    row.col(|ui| {
                        ui.text_edit_singleline(&mut self.new_form.key);
                    });
                    row.col(|ui| {
                        ui.text_edit_singleline(&mut self.new_form.value);
                    });
                    row.col(|ui| {
                        TextEdit::singleline(&mut self.new_form.desc)
                            .desired_width(f32::INFINITY)
                            .ui(ui);
                    });
                });
            });
        if delete_index.is_some() {
            data.rest
                .request
                .body_form_data
                .remove(delete_index.unwrap());
        }
        if self.new_form.key != "" || self.new_form.value != "" || self.new_form.desc != "" {
            self.new_form.enable = true;
            data.rest.request.body_form_data.push(self.new_form.clone());
            self.new_form.key = "".to_string();
            self.new_form.value = "".to_string();
            self.new_form.desc = "".to_string();
            self.new_form.enable = false;
        }
    }
}
