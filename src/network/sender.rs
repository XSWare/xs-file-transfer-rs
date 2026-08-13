use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

use egui::Context as EguiContext;
use xs_rust_library::connection::Connection;

use crate::{
    error_log::ErrorLog,
    network::{connection_control::ConnectionControl, file_transmission::FileTransmission},
    settings::Settings,
    ui::{LAST_SEND_PATH, request_repaint},
};

/// redraw UI when this is dropped
struct RedrawUi<'a> {
    ctx: &'a Option<EguiContext>,
}

impl<'a> RedrawUi<'a> {
    fn new(ctx: &'a Option<EguiContext>) -> Self {
        Self { ctx }
    }
}

impl Drop for RedrawUi<'_> {
    fn drop(&mut self) {
        request_repaint(&self.ctx);
    }
}

pub struct Sender {
    connection_control: Arc<ConnectionControl>,
    settings: Arc<Settings>,
    error_log: Arc<ErrorLog>,
    egui_context: Mutex<Option<EguiContext>>,
}

impl Sender {
    pub fn new(
        connection_control: Arc<ConnectionControl>,
        settings: Arc<Settings>,
        error_log: Arc<ErrorLog>,
    ) -> Self {
        Self {
            connection_control,
            settings,
            error_log,
            egui_context: Mutex::new(None),
        }
    }

    pub fn set_egui_context(&self, egui_context: EguiContext) {
        *self.egui_context.lock().unwrap() = Some(egui_context);
    }

    /// sends the file (or all files within the directory) at `file_or_directory_path`.
    /// spawns a new thread so the UI stays responsive.
    pub fn send(&self, file_or_directory_path: String) {
        let connection_control = self.connection_control.clone();
        let settings = self.settings.clone();
        let error_log = self.error_log.clone();
        let egui_context = self.egui_context.lock().unwrap().clone();

        thread::spawn(move || {
            let path = Path::new(&file_or_directory_path);
            if !path.exists() {
                error_log.log("file path does not exist".to_string());
                request_repaint(&egui_context);
                return;
            }

            let file_paths = match get_all_file_paths_in_directory(path) {
                Ok(v) => v,
                Err(error) => {
                    error_log.log(error.to_string());
                    request_repaint(&egui_context);
                    return;
                }
            };

            let parent_dir = path.parent().unwrap();

            for file_path in file_paths {
                // redraw UI when this is dropped
                let _redraw_ui = RedrawUi::new(&egui_context);
                if !file_path.starts_with(parent_dir) {
                    error_log.log(format!(
                        "file path {:?} does not contain passed path {:?}",
                        file_path, parent_dir
                    ));
                    return;
                }

                let sub_path = file_path.strip_prefix(parent_dir).unwrap();

                let directory = if path.is_file() {
                    match file_path.parent() {
                        Some(v) => v,
                        None => {
                            error_log.log("unable to extract directory".to_string());
                            return;
                        }
                    }
                } else {
                    parent_dir
                };

                error_log.log(format!("sending file {:?}", file_path));
                let packet_data =
                    match FileTransmission::create_packet_data_from_path(directory, sub_path) {
                        Ok(v) => v,
                        Err(error) => {
                            error_log.log(error.to_string());
                            return;
                        }
                    };

                let res = connection_control
                    .get_connection()
                    .lock()
                    .unwrap()
                    .as_mut()
                    .unwrap()
                    .send(&packet_data);

                match res {
                    Ok(()) => {
                        error_log.log(format!("sent file {:?}", file_path));
                    }
                    Err(error) => {
                        error_log.log(format!(
                            "error while sending file {:?}:\n{}",
                            file_path, error
                        ));
                        return;
                    }
                }
            }

            settings.set(LAST_SEND_PATH, file_or_directory_path);
        });
    }
}

fn get_all_file_paths_in_directory(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    if dir.is_dir() {
        let mut paths = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                paths.extend_from_slice(&get_all_file_paths_in_directory(&path)?);
            } else {
                paths.push(path);
            }
        }

        Ok(paths)
    } else {
        Ok(vec![dir.to_path_buf()])
    }
}
