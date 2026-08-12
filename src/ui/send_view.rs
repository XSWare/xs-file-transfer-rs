use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use egui::Widget;
use xs_rust_library::connection::Connection;

use crate::{
    Controls,
    error_log::ErrorLog,
    network::{connection_control::ConnectionControl, file_transmission::FileTransmission},
};

pub struct SendView {
    file_or_directory_path: String,
    connection_control: Arc<ConnectionControl>,
    error_log: Arc<ErrorLog>,
}

impl SendView {
    pub fn new(controls: &Controls) -> Self {
        Self {
            file_or_directory_path: if cfg!(debug_assertions) {
                "C:\\Games\\World of Warcraft\\_classic_era_\\Interface\\AddOns\\Auctionator"
                    .to_string()
            } else {
                String::new()
            },
            connection_control: controls.connection_control.clone(),
            error_log: controls.error_log.clone(),
        }
    }

    fn send(&self) {
        let path = Path::new(&self.file_or_directory_path);
        if !path.exists() {
            self.error_log.log("file path does not exist".to_string());
            return;
        }

        let file_paths = match get_all_file_paths_in_directory(path) {
            Ok(v) => v,
            Err(error) => {
                self.error_log.log(error.to_string());
                return;
            }
        };

        let parent_dir = path.parent().unwrap();

        for file_path in file_paths {
            if !file_path.starts_with(parent_dir) {
                self.error_log.log(format!(
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
                        self.error_log
                            .log("unable to extract directory".to_string());
                        return;
                    }
                }
            } else {
                parent_dir
            };

            self.error_log.log(format!(
                "sending file {:?}",
                file_path
            ));
            let packet_data =
                match FileTransmission::create_packet_data_from_path(directory, sub_path) {
                    Ok(v) => v,
                    Err(error) => {
                        self.error_log.log(error.to_string());
                        return;
                    }
                };

            let res = self.connection_control
                .get_connection()
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .send(&packet_data);

                match res {
                    Ok(()) => self.error_log
                .log(format!("sent file {:?}", file_path)),
                    Err(error) => self.error_log
                .log(format!("error while sending file {:?}:\n{}", file_path, error)),
                }
            
        }
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

impl Widget for &mut SendView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.heading("Send files");
        let edit_response = ui
            .horizontal(|ui| {
                ui.label("File or directory path: ");
                ui.text_edit_singleline(&mut self.file_or_directory_path)
            })
            .inner;

        if !self.file_or_directory_path.is_empty()
        /*&& self.connection_control.is_connected()*/
        {
            let button_response = ui.button("Send files");
            if button_response.clicked() {
                self.send();
            }

            return edit_response | button_response;
        }

        edit_response
    }
}
