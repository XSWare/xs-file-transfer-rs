use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use displaydoc::Display;
use thiserror::Error;

use xs_rust_library::{
    connection::Connection as ConnectionInterface,
    encrypted_connection::{self, EncryptedConnection, HandshakeError},
    encryption::aes256_crypto::Aes256Crypto,
    key_exchange::{HandshakeMode, curve25519::Curve25519},
    packet_connection::PacketConnection,
};

use crate::error_log::ErrorLog;
use crate::settings::Settings;

#[derive(Error, Display, Debug)]
pub enum Error {
    /// IO error: {0}
    IO(#[from] std::io::Error),
    /// Connection error: {0}
    Connection(#[from] encrypted_connection::TransmissionError),
    /// Handshake error: {0}
    HandshakeError(#[from] HandshakeError),
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

/// settings key under which the last successfully used remote endpoint is persisted.
pub const LAST_ENDPOINT_KEY: &str = "last_endpoint";
pub const LAST_ACCEPTED_PORT: &str = "last_port";

pub type Connection = EncryptedConnection<Aes256Crypto, PacketConnection>;

pub struct ConnectionControl {
    connection: Arc<Mutex<Option<Connection>>>,
    status: Arc<Mutex<ConnectionStatus>>,
    error_log: Arc<ErrorLog>,
    settings: Arc<Settings>,
}

impl ConnectionControl {
    pub fn new(error_log: Arc<ErrorLog>, settings: Arc<Settings>) -> Self {
        Self {
            connection: Default::default(),
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            error_log,
            settings,
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
        let settings = self.settings.clone();

        self.execute_connect_routine_async(move || {
            let stream = TcpStream::connect(addr)?;
            let packet_connection = PacketConnection::new(stream);
            let encrypted_connection = EncryptedConnection::with_handshake(
                packet_connection,
                Curve25519,
                HandshakeMode::Client,
            )?;
            *connection.lock().unwrap() = Some(encrypted_connection);
            set_status(&status, ConnectionStatus::Connected);
            settings.set(LAST_ENDPOINT_KEY, addr.to_string());
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
        let settings = self.settings.clone();

        self.execute_connect_routine_async(move || {
            let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
            let (stream, _) = listener.accept()?;
            let packet_connection = PacketConnection::new(stream);
            let encrypted_connection = EncryptedConnection::with_handshake(
                packet_connection,
                Curve25519,
                HandshakeMode::Server,
            )?;
            *cloned_connection.lock().unwrap() = Some(encrypted_connection);
            set_status(&status, ConnectionStatus::Connected);
            settings.set(LAST_ACCEPTED_PORT, port);
            Ok(())
        });
    }

    /// get the current connection. it might get replaced so always call this
    /// to get the most recently established connection.
    pub fn get_connection(&self) -> Arc<Mutex<Option<Connection>>> {
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
            if let Some(con) = connection.as_mut() {
                con.shutdown(std::net::Shutdown::Both)?;
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
