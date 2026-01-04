use std::{path::Path, sync::Arc};

use egui::Widget;

use crate::{
    Controls, connection_control::ConnectionControl, error_log::ErrorLog,
    file_transmission::FileTransmission,
};

pub struct ReceiveView {
    receive_directory_path: String,
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
}

impl ReceiveView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            receive_directory_path: get_default_directory(),
            connection_control: controls.connection_control.clone(),
            error_log: controls.error_log.clone(),
        }
    }

    fn receive(&self) {
        let dir = Path::new(&self.receive_directory_path);
        if !dir.exists() {
            self.error_log
                .log("receive directory does not exist".to_string());
            return;
        }

        if let Some(connection) = self
            .connection_control
            .get_connection()
            .lock()
            .unwrap()
            .as_mut()
        {
            self.error_log.log("waiting to receive file...".to_string());
            FileTransmission::receive_file(connection, &self.receive_directory_path);
            self.error_log.log("received file.".to_string());
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
                self.receive();
            }

            return edit_response | button_response;
        }

        edit_response
    }
}
