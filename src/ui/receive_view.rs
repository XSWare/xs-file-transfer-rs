use std::{path::PathBuf, sync::Arc};

use egui::Widget;

use crate::{
    Controls,
    error_log::ErrorLog,
    network::{connection_control::ConnectionControl, receiver::Receiver},
};

pub struct ReceiveView {
    receive_directory_path: PathBuf,
    receiver: Arc<Receiver>,
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
}

impl ReceiveView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            receive_directory_path: get_default_directory(),
            connection_control: controls.connection_control.clone(),
            receiver: controls.receiver.clone(),
            error_log: controls.error_log.clone(),
        }
    }

    fn receive(&self) {
        if let Err(error) = self.receiver.start_receiving(&self.receive_directory_path) {
            self.error_log.log(error.to_string())
        };
    }
}

fn get_default_directory() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|user_dir| {
            user_dir
                .download_dir()
                .map(|download_dir| download_dir.to_path_buf())
        })
        .unwrap_or_default()
}

impl Widget for &mut ReceiveView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Receive files");
        let edit_response = ui
            .horizontal(|ui| {
                ui.label("Receive directory: ");
                ui.text_edit_singleline(&mut self.receive_directory_path.to_str().unwrap())
            })
            .inner;

        if !self.receive_directory_path.iter().count() > 0 && self.connection_control.is_connected()
        {
            let button_response = ui.button("Receive files");
            if button_response.clicked() {
                self.receive();
            }

            return edit_response | button_response;
        }

        edit_response
    }
}
