use egui::Widget;

pub struct ReceiveView;

impl Widget for ReceiveView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Receive files");
        ui.horizontal(|ui| {
            ui.label("Receive directory: ");
            ui.text_edit_singleline(&mut "".to_string())
        })
        .inner
    }
}
