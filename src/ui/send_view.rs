use egui::Widget;

pub struct SendView {
    file_or_directory_path: String,
}

impl Default for SendView {
    fn default() -> Self {
        Self {
            file_or_directory_path: Default::default(),
        }
    }
}

impl Widget for &mut SendView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Send files");
        ui.horizontal(|ui| {
            ui.label("File or directory path: ");
            ui.text_edit_singleline(&mut self.file_or_directory_path)
        })
        .inner
    }
}
