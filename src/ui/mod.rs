mod connection_view;
mod error_view;
mod receive_view;
pub use receive_view::LAST_RECEIVE_PATH;
mod send_view;
pub use send_view::LAST_SEND_PATH;

use eframe::egui;

use crate::{
    Controls,
    ui::{
        connection_view::ConnectionView, error_view::ErrorView, receive_view::ReceiveView,
        send_view::SendView,
    },
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
        Box::new(|cc| {
            controls
                .connection_control
                .set_egui_context(cc.egui_ctx.clone());
            Ok(Box::new(MainWindow {
                connection_view: ConnectionView::new(controls),
                send_view: SendView::new(controls),
                receive_view: ReceiveView::new(controls),
                error_view: ErrorView::new(controls),
            }))
        }),
    )
}

struct MainWindow {
    connection_view: ConnectionView,
    send_view: SendView,
    receive_view: ReceiveView,
    error_view: ErrorView,
}

impl eframe::App for MainWindow {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.add(&mut self.connection_view);
                ui.separator();
                ui.add(&mut self.send_view);
                ui.separator();
                ui.add(&mut self.receive_view);
                ui.separator();
                ui.add(&mut self.error_view);
            });
        });
    }
}
