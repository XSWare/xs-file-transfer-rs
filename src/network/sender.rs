use std::{
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use xs_rust_library::connection::Connection;

use crate::{
    error_log::ErrorLog,
    network::{connection_control::ConnectionControl, file_transmission::FileTransmission},
    settings::Settings,
    ui::LAST_SEND_PATH,
};

/// sends the file (or all files within the directory) at `file_or_directory_path`.
///
/// all heavy operations run on a separate thread so the UI stays responsive.
pub fn send(
    file_or_directory_path: String,
    connection_control: Arc<ConnectionControl>,
    settings: Arc<Settings>,
    error_log: Arc<ErrorLog>,
) {
    thread::spawn(move || {
        let path = Path::new(&file_or_directory_path);
        if !path.exists() {
            error_log.log("file path does not exist".to_string());
            return;
        }

        let file_paths = match get_all_file_paths_in_directory(path) {
            Ok(v) => v,
            Err(error) => {
                error_log.log(error.to_string());
                return;
            }
        };

        let parent_dir = path.parent().unwrap();

        for file_path in file_paths {
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
                Ok(()) => error_log.log(format!("sent file {:?}", file_path)),
                Err(error) => {
                    error_log.log(format!("error while sending file {:?}:\n{}", file_path, error));
                    return;
                }
            }
        }

        settings.set(LAST_SEND_PATH, file_or_directory_path);
    });
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
