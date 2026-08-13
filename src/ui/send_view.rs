use std::sync::Arc;

use egui::Widget;

use crate::{
    Controls,
    network::{connection_control::ConnectionControl, sender::Sender},
};

/// settings key under which the last successfully sent file/directory is persisted.
pub const LAST_SEND_PATH: &str = "last_send_path";

pub struct SendView {
    file_or_directory_path: String,
    sender: Arc<Sender>,
    connection_control: Arc<ConnectionControl>,
}

impl SendView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            file_or_directory_path: controls.settings.get(LAST_SEND_PATH).unwrap_or_default(),
            sender: controls.sender.clone(),
            connection_control: controls.connection_control.clone(),
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
                self.sender.send(self.file_or_directory_path.clone());
            }

            return edit_response | button_response;
        }

        edit_response
    }
}
