use std::sync::Arc;

use egui::Widget;

use crate::{connection_control::ConnectionControl, file_transmission::FileTransmission};

pub struct ReceiveView {
    receive_directory_path: String,
    connection_control: Arc<ConnectionControl>,
}

impl ReceiveView {
    pub fn new(connection_control: Arc<ConnectionControl>) -> Self {
        Self {
            receive_directory_path: get_default_directory(),
            connection_control,
        }
    }
}

fn get_default_directory() -> String {
    if let Some(dir) = directories::UserDirs::new().and_then(|user_dir| {
        user_dir
            .download_dir()
            .and_then(|download_dir| download_dir.to_str().map(str::to_string))
    }) {
        dir
    } else {
        String::new()
    }
}

impl Widget for &mut ReceiveView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Receive files");
        let edit_response = ui
            .horizontal(|ui| {
                ui.label("Receive directory: ");
                ui.text_edit_singleline(&mut self.receive_directory_path)
            })
            .inner;

        if !self.receive_directory_path.is_empty() && self.connection_control.is_connected() {
            let button_response = ui.button("Receive files");
            if button_response.clicked() {
                if let Some(connection) = self
                    .connection_control
                    .get_connection()
                    .lock()
                    .unwrap()
                    .as_mut()
                {
                    FileTransmission::receive_file(connection, &self.receive_directory_path);
                }
            }

            return edit_response | button_response;
        }

        edit_response
    }
}
