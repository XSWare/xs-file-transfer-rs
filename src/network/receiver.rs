use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use displaydoc::Display;
use thiserror::Error;
use xs_rust_library::connection::Connection;

use crate::{
    error_log::ErrorLog,
    network::{connection_control::ConnectionControl, file_transmission::FileTransmission},
};

#[derive(Error, Display, Debug)]
pub enum Error {
    /// Receive loop was already started
    AlreadyReceiving,
}

#[allow(unused)]
pub enum ReceiveLoopStatus {
    NotStarted,
    Receiving(JoinHandle<()>),
    ShuttingDown,
}

pub struct Receiver {
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
    status: Mutex<ReceiveLoopStatus>,
    stop: Arc<AtomicBool>,
}

impl Receiver {
    pub fn new(connection_control: Arc<ConnectionControl>, error_log: Arc<ErrorLog>) -> Self {
        Self {
            connection_control,
            error_log,
            status: Mutex::new(ReceiveLoopStatus::NotStarted),
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_receiving(&self, receive_dir: &Path) -> Result<(), Error> {
        let mut status = self.status.lock().unwrap();
        match *status {
            ReceiveLoopStatus::NotStarted => {}
            _ => return Err(Error::AlreadyReceiving),
        }

        let stop = self.stop.clone();
        let connection_control = self.connection_control.clone();
        let error_log = self.error_log.clone();
        let receive_dir = receive_dir.to_path_buf();
        let join_handle = thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let connection_lock = connection_control.get_connection();
                let mut connection_guard = connection_lock.lock().unwrap();
                let Some(connection) = connection_guard.as_mut() else {
                    error_log.log("unable to start receiving: no active connection.".to_string());
                    return;
                };

                error_log.log("waiting to receive file...".to_string());
                let packet_data = connection.receive().unwrap();
                match FileTransmission::write_file_from_packet_data(&packet_data, &receive_dir) {
                    Ok(_) => error_log.log("received file".to_string()),
                    Err(error) => {
                        error_log.log(format!("error during receive: {}", error));
                        return;
                    }
                }
            }
        });

        *status = ReceiveLoopStatus::Receiving(join_handle);

        Ok(())
    }
}
