use egui::Widget;

pub struct ReceiveView {
    receive_directory_path: String,
}

impl Default for ReceiveView {
    fn default() -> Self {
        Self {
            receive_directory_path: Default::default(),
        }
    }
}

impl Widget for &mut ReceiveView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Receive files");
        ui.horizontal(|ui| {
            ui.label("Receive directory: ");
            ui.text_edit_singleline(&mut self.receive_directory_path)
        })
        .inner
    }
}
