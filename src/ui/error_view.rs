use std::sync::Arc;

use egui::Widget;

use crate::{Controls, error_log::ErrorLog};

#[derive(Clone)]
pub struct ErrorView {
    error_log: Arc<ErrorLog>,
}

impl ErrorView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            error_log: controls.error_log.clone(),
        }
    }
}

impl Widget for &mut ErrorView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if self.error_log.is_empty() {
            return ui.response();
        }

        let header_text = self.error_log.last_error().unwrap_or_default();

        egui::CollapsingHeader::new(header_text)
            .id_salt("error view expander")
            .default_open(false)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut first = true;
                    for error in &self.error_log.all_errors() {
                        if !first {
                            ui.separator();
                        }
                        first = false;
                        ui.label(error);
                    }
                });
            })
            .header_response
    }
}
