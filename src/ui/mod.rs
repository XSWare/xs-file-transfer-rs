mod connection_view;
mod receive_view;
mod send_view;

use std::sync::Arc;

use eframe::egui;

use crate::{
    Controls,
    error_log::ErrorLog,
    ui::{connection_view::ConnectionView, receive_view::ReceiveView, send_view::SendView},
};

pub fn show(controls: &Controls) -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        "xsFileTransfer",
        options,
        Box::new(|_cc| {
            Ok(Box::new(MainWindow {
                connection_view: ConnectionView::new(controls),
                send_view: SendView::default(),
                receive_view: ReceiveView::new(controls.connection_control.clone()),
                error_log: controls.error_log.clone(),
            }))
        }),
    )
}

struct MainWindow {
    connection_view: ConnectionView,
    send_view: SendView,
    receive_view: ReceiveView,
    error_log: Arc<ErrorLog>,
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let maybe_error = self.error_log.last_error();
        if let Some(error) = maybe_error {
            egui::TopBottomPanel::bottom("error_output").show(ctx, |ui| {
                ui.label(error);
            });
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add(&mut self.connection_view);
                ui.separator();
                ui.add(&mut self.send_view);
                ui.separator();
                ui.add(&mut self.receive_view);
            });
        });
    }
}
