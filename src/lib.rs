#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod error_log;
mod network;
mod settings;
mod ui;

use std::sync::Arc;

use crate::error_log::ErrorLog;
use crate::network::connection_control::ConnectionControl;
use crate::network::receiver::Receiver;
use crate::settings::Settings;

pub struct Controls {
    connection_control: Arc<ConnectionControl>,
    receiver: Arc<Receiver>,
    error_log: Arc<ErrorLog>,
    settings: Arc<Settings>,
}

pub fn run() {
    let error_log = Arc::new(ErrorLog::default());
    let settings = Arc::new(Settings::load());
    let connection_control = Arc::new(ConnectionControl::new(error_log.clone(), settings.clone()));
    let controls = Controls {
        receiver: Arc::new(Receiver::new(
            connection_control.clone(),
            settings.clone(),
            error_log.clone(),
        )),
        connection_control,
        error_log,
        settings,
    };
    ui::show(&controls).unwrap();
}
