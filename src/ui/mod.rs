mod connection_view;
mod receive_view;
mod send_view;

use eframe::egui;

use crate::{
    Controls,
    ui::{connection_view::ConnectionView, receive_view::ReceiveView, send_view::SendView},
};

pub fn show(controls: &Controls) -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "xs file transfer",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MainWindow {
                connection_view: ConnectionView::new(controls),
            }))
        }),
    )
}

struct MainWindow {
    connection_view: ConnectionView,
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add(&mut self.connection_view);
                ui.separator();
                ui.add(SendView);
                ui.separator();
                ui.add(ReceiveView);
            });
        });
    }
}
