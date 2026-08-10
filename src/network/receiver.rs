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
use xs_rust_library::{connection::Connection, packet_connection::PacketConnection};

use crate::{
    error_log::ErrorLog,
    network::{connection_control::ConnectionControl, file_transmission::FileTransmission},
};

#[derive(Error, Display, Debug)]
pub enum Error {
    /// Receive loop was already started
    AlreadyReceiving,
    /// Tcp stream error: {0}
    TcpStreamError(#[from] std::io::Error),
    /// Unable to start receiving without an established connection
    NoConnection,
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

        self.error_log.log("waiting to receive file...".to_string());
        let stop = self.stop.clone();
        let send_connection = self.connection_control.get_connection().clone();
        let tcp_stream = send_connection
            .lock()
            .unwrap()
            .as_mut()
            .ok_or(Error::NoConnection)?
            .tcp_stream()
            .try_clone()?;
        let mut connection = PacketConnection::new(
            tcp_stream,
            self.connection_control.get_receive_buffer_size(),
        );
        let error_log = self.error_log.clone();
        let receive_dir = receive_dir.to_path_buf();
        let join_handle = thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let packet_data = connection.receive().unwrap();
                match FileTransmission::write_file_from_packet_data(&packet_data, &receive_dir) {
                    Ok(file_path) => error_log.log(format!(
                        "received file: \"{}\"",
                        file_path.to_str().unwrap()
                    )),
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
