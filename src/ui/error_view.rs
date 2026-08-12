use std::sync::Arc;

use egui::{Align, Widget, collapsing_header::CollapsingState};

use crate::{Controls, error_log::ErrorLog};

#[derive(Clone)]
pub struct ErrorView {
    error_log: Arc<ErrorLog>,
    was_expanded: bool,
}

impl ErrorView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            error_log: controls.error_log.clone(),
            was_expanded: false,
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
        let expanded = state.is_open();
        let just_toggled = expanded != self.was_expanded;
        self.was_expanded = expanded;

        let header_response = egui::Panel::bottom("error view header")
            .show(ui, |ui| {
                let header_text = if state.is_open() {
                    "Logs".to_string()
                } else {
                    self.error_log.last_error().unwrap_or_default()
                };

                state
                    .show_header(ui, |ui| ui.label(header_text))
                    .body(|_ui| {})
            })
            .inner;

        if expanded {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let mut first = true;
                    for error in &self.error_log.all_errors() {
                        if !first {
                            ui.separator();
                        }
                        first = false;
                        ui.label(error);
                    }

                    if just_toggled {
                        ui.scroll_to_cursor(Some(Align::BOTTOM));
                    }
                });
        }

        header_response.0
    }
}
