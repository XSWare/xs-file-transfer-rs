use egui::Widget;

pub struct SendView;

impl Widget for SendView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Send files");
        ui.horizontal(|ui| {
            ui.label("File or directory path: ");
            ui.text_edit_singleline(&mut "".to_string())
        })
        .inner
    }
}
