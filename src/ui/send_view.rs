use std::{path::Path, sync::Arc};

use egui::Widget;

use crate::{
    Controls, connection_control::ConnectionControl, error_log::ErrorLog,
    file_transmission::FileTransmission,
};

pub struct SendView {
    file_or_directory_path: String,
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
}

impl SendView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            file_or_directory_path:
                "D:\\Projects\\Rust\\XSRustyFileTransfer\\target\\debug\\test.txt".to_string(),
            connection_control: controls.connection_control.clone(),
            error_log: controls.error_log.clone(),
        }
    }

    fn send(&self) {
        let path = Path::new(&self.file_or_directory_path);
        if !path.exists() || !path.is_file() {
            self.error_log.log("file does not exist".to_string());
            return;
        }

        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            self.error_log
                .log("unable to extract file name".to_string());
            return;
        };

        let Some(directory) = path
            .parent()
            .and_then(|parent| parent.to_str())
            .map(|dir| format!("{}\\", dir))
        else {
            self.error_log
                .log("unable to extract directory".to_string());
            return;
        };

        if let Some(connection) = self
            .connection_control
            .get_connection()
            .lock()
            .unwrap()
            .as_mut()
        {
            self.error_log
                .log(format!("sending file \"{}{}\"", directory, file_name));
            FileTransmission::send_file(connection, &directory, file_name);
        } else {
            self.error_log.log("failed to get connection".to_string());
        }
    }
}

impl Widget for &mut SendView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Send files");
        let edit_response = ui
            .horizontal(|ui| {
                ui.label("File or directory path: ");
                ui.text_edit_singleline(&mut self.file_or_directory_path)
            })
            .inner;

        if !self.file_or_directory_path.is_empty() && self.connection_control.is_connected() {
            let button_response = ui.button("Send files");
            if button_response.clicked() {
                self.send();
            }

            return edit_response | button_response;
        }

        edit_response
    }
}
