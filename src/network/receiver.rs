use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use displaydoc::Display;
use egui::Context as EguiContext;
use thiserror::Error;
use xs_rust_library::{
    connection::Connection as ConnectionInterface, encrypted_connection::TransmissionError,
    receive_loop::ReceiveLoop,
};

use crate::{
    error_log::ErrorLog, network::{connection_control::ConnectionControl, file_transmission::FileTransmission}, settings::Settings, ui::{LAST_RECEIVE_PATH, request_repaint},
};

#[derive(Error, Display, Debug)]
pub enum Error {
    /// Receive loop was already started
    AlreadyReceiving,
    /// Unable to clone connection: {0}
    ConnectionClone(#[from] TransmissionError),
    /// Unable to start receiving without an established connection
    NoConnection,
}

#[allow(unused)]
pub enum ReceiveLoopStatus {
    NotStarted,
    Receiving,
    ShuttingDown,
}

pub struct Receiver {
    connection_control: Arc<ConnectionControl>,
    settings: Arc<Settings>,
    error_log: Arc<ErrorLog>,
    receive_loop: Mutex<Option<ReceiveLoop>>,
    egui_context: Mutex<Option<EguiContext>>,
}

impl Receiver {
    pub fn new(
        connection_control: Arc<ConnectionControl>,
        settings: Arc<Settings>,
        error_log: Arc<ErrorLog>,
    ) -> Self {
        Self {
            connection_control,
            settings,
            error_log,
            receive_loop: Mutex::new(None),
            egui_context: Mutex::new(None),
        }
    }

    pub fn set_egui_context(&self, egui_context: EguiContext) {
        *self.egui_context.lock().unwrap() = Some(egui_context);
    }

    pub fn start_receiving(&self, receive_dir: &Path) -> Result<(), Error> {
        match self.receive_status() {
            ReceiveLoopStatus::NotStarted => {}
            _ => return Err(Error::AlreadyReceiving),
        }

        self.error_log.log("waiting to receive file...".to_string());
        let send_connection = self.connection_control.get_connection().clone();
        let connection = send_connection
            .lock()
            .unwrap()
            .as_mut()
            .ok_or(Error::NoConnection)?
            .try_clone()?;

        let settings = self.settings.clone();
        let error_log = self.error_log.clone();
        let egui_context = self.egui_context.lock().unwrap().clone();
        let receive_dir = receive_dir.to_path_buf();
        let packet_handler = move |packet: &Vec<u8>| {
            match FileTransmission::write_file_from_packet_data(&packet, &receive_dir) {
                Ok(file_path) => {
                    settings.set(LAST_RECEIVE_PATH, receive_dir.to_string_lossy().to_string());
                    error_log.log(format!(
                        "received file: \"{}\"",
                        file_path.to_str().unwrap()
                    ));
                }
                Err(error) => {
                    error_log.log(format!("error during receive: {}", error));
                }
            };

            // egui ignores changes from a background thread so force a redraw
            request_repaint(&egui_context);
        };

        *self.receive_loop.lock().unwrap() = Some(ReceiveLoop::start(
            connection,
            vec![Box::new(packet_handler)],
        ));

        Ok(())
    }

    pub fn receive_status(&self) -> ReceiveLoopStatus {
        let receive_loop_guard = self.receive_loop.lock().unwrap();
        if let Some(receive_loop) = &*receive_loop_guard {
            if receive_loop.stopped() {
                ReceiveLoopStatus::ShuttingDown
            } else {
                ReceiveLoopStatus::Receiving
            }
        } else {
            ReceiveLoopStatus::NotStarted
        }
    }

    #[allow(unused)]
    pub fn stop(&self) {
        let mut receive_loop_guard = self.receive_loop.lock().unwrap();
        let Some(receive_loop) = &*receive_loop_guard else {
            self.error_log
                .log("receiving has not yet started: unable to stop".to_string());
            return;
        };
        receive_loop.stop();
        *receive_loop_guard = None;
    }
}
