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

pub struct ConnectionControl {
    connection: Arc<Mutex<Option<PacketConnection>>>,
    receive_buffer_size: usize,
    port: u16,
    error_log: Arc<ErrorLog>,
}

impl ConnectionControl {
    pub fn new(error_log: Arc<ErrorLog>) -> Self {
        Self {
            connection: Default::default(),
            receive_buffer_size: 1024,
            port: 3648,
            error_log,
        }
    }

    pub fn connect(&self, addr: SocketAddr) {
        if self.is_connected() {
            self.error_log.log(Error::AlreadyConnected.to_string());
            return;
        }

        let cloned_connection = self.connection.clone();
        let receive_buffer_size = self.receive_buffer_size;

        self.execute_async(move || {
            let stream = TcpStream::connect(addr)?;
            let packet_connection = PacketConnection::new(stream, receive_buffer_size);
            let mut connection = cloned_connection.lock().unwrap();
            if connection.is_some() {
                return Err(Error::AlreadyConnected);
            }
            *connection = Some(packet_connection);
            Ok(())
        });
    }

    pub fn accept(&self) {
        if self.is_connected() {
            self.error_log.log(Error::AlreadyConnected.to_string());
            return;
        }

        let cloned_connection = self.connection.clone();
        let port = self.port;
        let receive_buffer_size = self.receive_buffer_size;

        self.execute_async(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
            let (stream, _) = listener.accept()?;
            let packet_connection = PacketConnection::new(stream, receive_buffer_size);
            let mut connection = cloned_connection.lock().unwrap();
            if connection.is_some() {
                return Err(Error::AlreadyConnected);
            }
            *connection = Some(packet_connection);
            Ok(())
        });
    }

    /// get the current connection. it might get replaced so always call this
    /// to get the most recently established connection.
    pub fn get_connection(&self) -> Arc<Mutex<Option<PacketConnection>>> {
        self.connection.clone()
    }

    pub fn disconnect(&self) {
        self.execute_logged(|| {
            let mut connection = self.connection.lock().unwrap();
            if let Some(packet_connection) = connection.as_ref() {
                packet_connection.shutdown(std::net::Shutdown::Both)?;
                *connection = None;
                Ok(())
            } else {
                Err(Error::AlreadyDisconnected)
            }
        });
    }

    pub fn is_connected(&self) -> bool {
        self.connection.lock().unwrap().is_some()
    }

    fn execute_logged(&self, f: impl FnOnce() -> Result<(), Error>) {
        let res = f();
        if let Err(error) = res {
            self.error_log.log(error.to_string());
        }
    }

    fn execute_async(&self, f: impl FnOnce() -> Result<(), Error> + Send + 'static) {
        let error_log = self.error_log.clone();
        thread::spawn(move || {
            let res = f();
            if let Err(error) = res {
                error_log.log(error.to_string());
            }
        });
    }
}
