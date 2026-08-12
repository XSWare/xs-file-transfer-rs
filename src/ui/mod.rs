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
                send_view: SendView::new(controls),
                receive_view: ReceiveView::new(controls),
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add(&mut self.connection_view);
                ui.separator();
                ui.add(&mut self.send_view);
                ui.separator();
                ui.add(&mut self.receive_view);
                if !self.error_log.is_empty() {
                    ui.separator();
                    self.show_error_output(ui);
                }
            });
        });
    }
}

impl MainWindow {
    fn show_error_output(&self, ui: &mut egui::Ui) {
        let collapsed_id = ui.make_persistent_id("error_output_collapsing");
        let is_open =
            egui::containers::collapsing_header::CollapsingState::load(ui.ctx(), collapsed_id)
                .is_some_and(|state| state.is_open());

        let header_text = if is_open {
            format!("Errors ({})", self.error_log.error_count())
        } else {
            // Show the most recent error while collapsed.
            self.error_log.last_error().unwrap_or_default()
        };

        egui::CollapsingHeader::new(header_text)
            .id_salt("error_output_collapsing")
            .default_open(false)
            .show(ui, |ui| {
                // Fill all the leftover space in the central panel below the views.
                ui.set_min_height(ui.available_height());
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        let mut first = true;
                        for error in &self.error_log.all_errors() {
                            if !first {
                                ui.separator();
                            }
                            first = false;
                            ui.label(error);
                        }
                    });
            });
    }
}
