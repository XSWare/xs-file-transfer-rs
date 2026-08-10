use std::{sync::Arc, thread};

use egui::Widget;
use xs_rust_library::connection::Connection;

use crate::{
    Controls, error_log::ErrorLog, file_transmission::FileTransmission,
    network::connection_control::ConnectionControl,
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
        let receive_directory_path = self.receive_directory_path.clone();
        let connection_control = self.connection_control.clone();
        let error_log = self.error_log.clone();

        thread::spawn(move || {
            loop {
                error_log.log("waiting to receive file...".to_string());
                let packet_data = connection_control
                    .get_connection()
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .receive()
                    .unwrap();
                match FileTransmission::write_file_from_packet_data(
                    &packet_data,
                    &receive_directory_path,
                ) {
                    Ok(_) => error_log.log("received file.".to_string()),
                    Err(error) => error_log.log(error.to_string()),
                }
            }
        });
    }
}

fn get_default_directory() -> String {
    directories::UserDirs::new()
        .and_then(|user_dir| {
            user_dir
                .download_dir()
                .and_then(|download_dir| download_dir.to_str().map(str::to_string))
        })
        .unwrap_or_default()
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
