use std::sync::Arc;

use egui::{Widget, collapsing_header::CollapsingState};

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

        let id = ui.make_persistent_id("error view expander");
        let state = CollapsingState::load_with_default_open(ui.ctx(), id.into(), false);

        let header_text = if state.is_open() {
            "Logs".to_string()
        } else {
            self.error_log.last_error().unwrap_or_default()
        };

        state
            .show_header(ui, |ui| ui.label(header_text))
            .body(|ui| {
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
            .0
    }
}
