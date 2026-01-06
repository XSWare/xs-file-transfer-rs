use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use displaydoc::Display;
use thiserror::Error;

use xs_rust_library::packet_connection::{self, PacketConnection};

use crate::error_log::ErrorLog;

#[derive(Error, Display, Debug)]
pub enum Error {
    /// IO error: {0}
    IO(#[from] std::io::Error),
    /// Connection error: {0}
    Connection(#[from] packet_connection::Error),
    /// Could not connect since connection is already established.
    AlreadyConnected,
    /// Could not disconnect since no connection is established.
    AlreadyDisconnected,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Accepting,
}

pub struct ConnectionControl {
    connection: Arc<Mutex<Option<PacketConnection>>>,
    status: Arc<Mutex<ConnectionStatus>>,
    receive_buffer_size: usize,
    error_log: Arc<ErrorLog>,
}

impl ConnectionControl {
    pub fn new(error_log: Arc<ErrorLog>) -> Self {
        Self {
            connection: Default::default(),
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            receive_buffer_size: 1024,
            error_log,
        }
    }

    pub fn connect(&self, addr: SocketAddr) {
        if !self.is_disconnected() {
            self.error_log.log(Error::AlreadyConnected.to_string());
            return;
        }

        self.set_status(ConnectionStatus::Connecting);

        let connection = self.connection.clone();
        let status = self.status.clone();
        let receive_buffer_size = self.receive_buffer_size;

        self.execute_connect_routine_async(move || {
            let stream = TcpStream::connect(addr)?;
            let packet_connection = PacketConnection::new(stream, receive_buffer_size);
            *connection.lock().unwrap() = Some(packet_connection);
            set_status(&status, ConnectionStatus::Connected);
            Ok(())
        });
    }

    pub fn accept(&self, port: String) {
        if self.is_connected() {
            self.error_log.log(Error::AlreadyConnected.to_string());
            return;
        }

        self.set_status(ConnectionStatus::Accepting);

        let cloned_connection = self.connection.clone();
        let status = self.status.clone();
        let receive_buffer_size = self.receive_buffer_size;

        self.execute_connect_routine_async(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
            let (stream, _) = listener.accept()?;
            let packet_connection = PacketConnection::new(stream, receive_buffer_size);
            *cloned_connection.lock().unwrap() = Some(packet_connection);
            set_status(&status, ConnectionStatus::Connected);
            Ok(())
        });
    }

    /// get the current connection. it might get replaced so always call this
    /// to get the most recently established connection.
    pub fn get_connection(&self) -> Arc<Mutex<Option<PacketConnection>>> {
        self.connection.clone()
    }

    pub fn get_status(&self) -> ConnectionStatus {
        get_status(&self.status)
    }

    fn set_status(&self, new_status: ConnectionStatus) {
        set_status(&self.status, new_status);
    }

    pub fn disconnect(&self) {
        self.execute_logged(|| {
            if self.is_disconnected() {
                return Err(Error::AlreadyDisconnected);
            }

            let mut connection = self.connection.lock().unwrap();
            if let Some(packet_connection) = connection.as_ref() {
                packet_connection.shutdown(std::net::Shutdown::Both)?;
                *connection = None;
                self.set_status(ConnectionStatus::Disconnected);
                Ok(())
            } else {
                Err(Error::AlreadyDisconnected)
            }
        });
    }

    pub fn is_connected(&self) -> bool {
        self.get_status() == ConnectionStatus::Connected
    }

    pub fn is_disconnected(&self) -> bool {
        self.get_status() == ConnectionStatus::Disconnected
    }

    fn execute_logged(&self, f: impl FnOnce() -> Result<(), Error>) {
        let res = f();
        if let Err(error) = res {
            self.error_log.log(error.to_string());
        }
    }

    /// executes a connect routine in a separate thread and sets status and error log accordingly
    fn execute_connect_routine_async(
        &self,
        f: impl FnOnce() -> Result<(), Error> + Send + 'static,
    ) {
        let status = self.status.clone();
        let error_log = self.error_log.clone();
        thread::spawn(move || {
            let res = f();
            if let Err(error) = res {
                set_status(&status, ConnectionStatus::Disconnected);
                error_log.log(error.to_string());
            }
        });
    }
}

fn get_status(status: &Arc<Mutex<ConnectionStatus>>) -> ConnectionStatus {
    *status.lock().unwrap()
}

fn set_status(status: &Arc<Mutex<ConnectionStatus>>, new_status: ConnectionStatus) {
    *status.lock().unwrap() = new_status
}
