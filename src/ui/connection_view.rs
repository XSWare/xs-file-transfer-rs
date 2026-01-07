use std::sync::Arc;

use egui::{Grid, Response, TextEdit, Ui, Widget};

use crate::{
    Controls,
    connection_control::{ConnectionControl, ConnectionStatus},
    error_log::ErrorLog,
};

#[derive(Clone)]
pub struct ConnectionView {
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
    remote_address: String,
    accept_port: String,
}

impl ConnectionView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            connection_control: controls.connection_control.clone(),
            error_log: controls.error_log.clone(),
            remote_address: if cfg!(debug_assertions) {
                "127.0.0.1:3648".to_string()
            } else {
                String::new()
            },
            accept_port: "3648".to_string(),
        }
    }

    fn get_status(&self) -> ConnectionStatus {
        self.connection_control.get_status()
    }

    fn connection_status_as_str(&self) -> &'static str {
        match self.get_status() {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Connecting => "Connecting",
            ConnectionStatus::Accepting => "Waiting for remote to connect",
        }
    }

    fn add_connection_management_widget(&mut self, ui: &mut Ui) -> Response {
        Grid::new("connect_grid")
            .show(ui, |ui| {
                let connect_response = self.add_connect_widget(ui);
                ui.end_row();
                let accept_response = self.add_accept_widget(ui);
                ui.end_row();

                connect_response | accept_response
            })
            .inner
    }

    fn add_connect_widget(&mut self, ui: &mut Ui) -> Response {
        let address_label_response = ui.label("Remote address: ");

        let address_edit_response = ui.add_sized(
            [140., ui.available_height()],
            TextEdit::singleline(&mut self.remote_address),
        );

        let connect_response = ui.button("Connect");
        if connect_response.clicked() {
            self.on_connect_button_clicked();
        }

        address_label_response | address_edit_response | connect_response
    }

    fn add_accept_widget(&mut self, ui: &mut Ui) -> Response {
        let port_label_response = ui.label("Port: ");

        let port_edit_response = ui.add_sized(
            [50., ui.available_height()],
            TextEdit::singleline(&mut self.accept_port),
        );

        let accept_response = ui.button("Accept");
        if accept_response.clicked() {
            self.on_accept_button_clicked();
        }

        port_label_response | port_edit_response | accept_response
    }

    fn on_connect_button_clicked(&self) {
        match self.remote_address.parse() {
            Ok(address) => self.connection_control.connect(address),
            Err(error) => self.error_log.log(error.to_string()),
        };
    }

    fn on_accept_button_clicked(&self) {
        self.connection_control.accept(self.accept_port.clone());
    }

    fn on_disconnect_button_clicked(&self) {
        self.connection_control.disconnect();
    }
}

impl Widget for &mut ConnectionView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.heading("Connection");
            let status_label_response = ui.label(format!(
                "Connection status: {}",
                self.connection_status_as_str()
            ));

            let connect_management_response = add_conditional_widget(
                self.get_status() == ConnectionStatus::Disconnected,
                status_label_response,
                || self.add_connection_management_widget(ui),
            );

            let disconnect_response = add_conditional_widget(
                self.get_status() == ConnectionStatus::Connected,
                connect_management_response,
                || {
                    let response = ui.button("Disconnect");
                    if response.clicked() {
                        self.on_disconnect_button_clicked();
                    }
                    response
                },
            );
            disconnect_response
        })
        .inner
    }
}

/// add a widget only if the condition is true and combine their responses
fn add_conditional_widget(
    condition: bool,
    current_response: Response,
    add_widget: impl FnOnce() -> Response,
) -> Response {
    if condition {
        current_response | add_widget()
    } else {
        current_response
    }
}
