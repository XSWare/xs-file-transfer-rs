#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod error_log;
mod network;
mod ui;

use std::sync::Arc;

use crate::error_log::ErrorLog;
use crate::network::connection_control::ConnectionControl;
use crate::network::receiver::Receiver;

pub struct Controls {
    connection_control: Arc<ConnectionControl>,
    receiver: Arc<Receiver>,
    error_log: Arc<ErrorLog>,
}

pub fn run() {
    let error_log = Arc::new(ErrorLog::default());
    let connection_control = Arc::new(ConnectionControl::new(error_log.clone()));
    let controls = Controls {
        receiver: Arc::new(Receiver::new(connection_control.clone(), error_log.clone())),
        connection_control,
        error_log,
    };
    ui::show(&controls).unwrap();
}
