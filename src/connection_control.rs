use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Mutex,
};

use displaydoc::Display;
use thiserror::Error;

use xs_rust_library::packet_connection::{self, PacketConnection};

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
    connection: Mutex<Option<PacketConnection>>,
    receive_buffer_size: usize,
    port: u16,
}

impl ConnectionControl {
    pub fn connect(&self, addr: SocketAddr) -> Result<(), Error> {
        let mut connection = self.connection.lock().unwrap();
        if connection.is_some() {
            return Err(Error::AlreadyConnected);
        }

        let stream = TcpStream::connect(addr)?;
        let packet_connection = PacketConnection::new(stream, self.receive_buffer_size);
        *connection = Some(packet_connection);
        Ok(())
    }

    pub fn accept(&self) -> Result<(), Error> {
        let mut connection = self.connection.lock().unwrap();
        if connection.is_some() {
            return Err(Error::AlreadyConnected);
        }

        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port))?;
        let (stream, _) = listener.accept()?;
        let packet_connection = PacketConnection::new(stream, self.receive_buffer_size);
        *connection = Some(packet_connection);
        Ok(())
    }

    pub fn disconnect(&self) -> Result<(), Error> {
        let mut connection = self.connection.lock().unwrap();
        if let Some(packet_connection) = connection.as_ref() {
            packet_connection.shutdown(std::net::Shutdown::Both)?;
            *connection = None;
            Ok(())
        } else {
            Err(Error::AlreadyDisconnected)
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.lock().unwrap().is_some()
    }
}

impl Default for ConnectionControl {
    fn default() -> Self {
        Self {
            connection: Default::default(),
            receive_buffer_size: 1024,
            port: 3648,
        }
    }
}
