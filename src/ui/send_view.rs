use std::sync::Arc;

use egui::Widget;

use crate::{
    Controls,
    error_log::ErrorLog,
    network::{connection_control::ConnectionControl, sender},
    settings::Settings,
};

/// settings key under which the last successfully sent file/directory is persisted.
pub const LAST_SEND_PATH: &str = "last_send_path";

pub struct SendView {
    file_or_directory_path: String,
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
    settings: Arc<Settings>,
}

impl SendView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            file_or_directory_path: controls.settings.get(LAST_SEND_PATH).unwrap_or_default(),
            connection_control: controls.connection_control.clone(),
            error_log: controls.error_log.clone(),
            settings: controls.settings.clone(),
        }
    }
}

impl Widget for &mut SendView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Send files");
        let edit_response = ui
            .horizontal(|ui| {
                ui.label("File or directory path: ");
                let edit_response = ui.text_edit_singleline(&mut self.file_or_directory_path);
                if ui.button("Browse file...").clicked() {
                    if let Some(file) = rfd::FileDialog::new().pick_file() {
                        self.file_or_directory_path = file.to_string_lossy().to_string();
                    }
                }
                if ui.button("Browse folder...").clicked() {
                    if let Some(directory) = rfd::FileDialog::new().pick_folder() {
                        self.file_or_directory_path = directory.to_string_lossy().to_string();
                    }
                }
                edit_response
            })
            .inner;

        if !self.file_or_directory_path.is_empty() && self.connection_control.is_connected() {
            let button_response = ui.button("Send files");
            if button_response.clicked() {
                sender::send(
                    self.file_or_directory_path.clone(),
                    self.connection_control.clone(),
                    self.settings.clone(),
                    self.error_log.clone(),
                );
            }

            return edit_response | button_response;
        }

        edit_response
    }
}
