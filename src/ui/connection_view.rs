use egui::Widget;

pub struct ConnectionView {
    is_connected: bool,
}

impl ConnectionView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connection_status_as_string(&self) -> &str {
        if self.is_connected {
            "Connected"
        } else {
            "Disconnected"
        }
    }

    pub fn connect_button_label(&self) -> &str {
        if self.is_connected {
            "Disconnect"
        } else {
            "Connect"
        }
    }
}

impl Default for ConnectionView {
    fn default() -> Self {
        Self {
            is_connected: false,
        }
    }
}

impl Widget for ConnectionView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let label_response = ui.label(format!(
                "Connection status: {}",
                self.connection_status_as_string()
            ));
            let button_response = ui.button(self.connect_button_label());
            label_response | button_response
        })
        .inner
    }
}
