use std::sync::Arc;

use egui::{Response, Ui, Widget};

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

    fn add_error_log(&mut self, ui: &mut Ui) -> Response {
        if self.error_log.is_empty() {
            return ui.response();
        }

        let collapsed_id = ui.make_persistent_id("error_output_collapsing");
        let is_open =
            egui::containers::collapsing_header::CollapsingState::load(ui.ctx(), collapsed_id)
                .is_some_and(|state| state.is_open());

        let header_text = if is_open {
            format!("Errors ({})", self.error_log.error_count())
        } else {
            // Show the most recent error while collapsed.
            self.error_log.last_error().unwrap_or_default()
        };

        egui::CollapsingHeader::new(header_text)
            .id_salt(collapsed_id)
            .default_open(false)
            .show(ui, |ui| {
                // Fill all the leftover space in the docked bottom panel.
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

impl Widget for &mut ErrorView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.add_error_log(ui)
    }
}
