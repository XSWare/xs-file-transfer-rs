mod connection_view;

use eframe::egui;

use crate::ui::connection_view::ConnectionView;

pub fn show() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "xs file transfer",
        options,
        Box::new(|_cc| Ok(Box::<MainWindow>::default())),
    )
}

struct MainWindow {}

impl Default for MainWindow {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(ConnectionView::new());
            });
        });
    }
}
