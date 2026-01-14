#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod error_log;
mod file_transmission;
mod network;
mod ui;

use std::sync::Arc;

use crate::error_log::ErrorLog;
use crate::network::connection_control::ConnectionControl;

pub struct Controls {
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
}

pub fn run() {
    let error_log = Arc::new(ErrorLog::default());
    let controls = Controls {
        connection_control: Arc::new(ConnectionControl::new(error_log.clone())),
        error_log,
    };
    ui::show(&controls).unwrap();
}
