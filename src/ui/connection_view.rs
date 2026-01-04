use std::sync::Arc;

use egui::Widget;

use crate::{Controls, connection_control::ConnectionControl, error_log::ErrorLog};

#[derive(Clone)]
pub struct ConnectionView {
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
}

impl ConnectionView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            connection_control: controls.connection_control.clone(),
            error_log: controls.error_log.clone(),
        }
    }

    fn is_connected(&self) -> bool {
        self.connection_control.is_connected()
    }

    pub fn connection_status_as_string(&self) -> &str {
        if self.is_connected() {
            "Connected"
        } else {
            "Disconnected"
        }
    }

    pub fn connect_button_label(&self) -> &str {
        if self.is_connected() {
            "Disconnect"
        } else {
            "Connect"
        }
    }
}

impl Widget for &mut ConnectionView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.heading("Connection");
            let response = ui
                .horizontal(|ui| {
                    let label_response = ui.label(format!(
                        "Connection status: {}",
                        self.connection_status_as_string()
                    ));
                    let connect_button_response = ui.button(self.connect_button_label());

                    if connect_button_response.clicked() {
                        let res = if self.is_connected() {
                            self.connection_control.disconnect()
                        } else {
                            self.connection_control
                                .connect("127.0.0.1:3648".parse().unwrap())
                        };

                        if let Err(error) = res {
                            self.error_log.log(error.to_string());
                        };
                    }

                    if self.is_connected() {
                        label_response | connect_button_response
                    } else {
                        let accept_button_response = ui.button("Accept");
                        if accept_button_response.clicked() {
                            if let Err(error) = self.connection_control.accept() {
                                self.error_log.log(error.to_string());
                            }
                        }
                        label_response | connect_button_response | accept_button_response
                    }
                })
                .inner;

            if let Some(error) = &self.error_log.last_error() {
                ui.label(error);
            };

            response
        })
        .inner
    }
}
