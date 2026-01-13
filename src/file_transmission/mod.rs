mod header;

use std::{
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use bytemuck::{bytes_of, from_bytes};
use displaydoc::Display;
use thiserror::Error;
use xs_rust_library::{
    connection::Connection,
    packet_connection::{self, PacketConnection},
};

use crate::file_transmission::header::Header;

#[derive(Error, Display, Debug)]
pub enum Error {
    /// File IO error: {0}
    FileIO(#[from] std::io::Error),
    ///
    InvalidFilePath(#[from] std::str::Utf8Error),
    /// No connection available
    NoConnection,
    /// Error during transmission: {0}
    Transmission(#[from] packet_connection::Error),
    /// File name not in a recognized format
    FilenameDecoding,
}

pub struct FileTransmission;

impl FileTransmission {
    /// send a file. `sub_path` can be the file name or a subdirectory with the filename.
    /// the subdirectoy is used to restore the same file hierarchy at the destination.
    pub fn send_file(
        connection: Arc<Mutex<Option<PacketConnection>>>,
        directory: &Path,
        sub_path: &Path,
    ) -> Result<(), Error> {
        let file_path = directory.join(sub_path);
        let mut file = File::open(file_path)?;
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content)?;
        let sub_path_as_string = sub_path.to_str().ok_or(Error::FilenameDecoding)?;
        let header = Header::new(file_content.len(), sub_path_as_string.len());
        let mut data = Vec::with_capacity(size_of::<Header>() + file_content.len());
        data.extend_from_slice(bytes_of(&header));
        data.extend_from_slice(sub_path_as_string.as_bytes());
        data.extend_from_slice(&file_content);
        connection
            .lock()
            .unwrap()
            .as_mut()
            .ok_or(Error::NoConnection)?
            .send(&data)?;
        Ok(())
    }

    pub fn receive_file(
        connection: Arc<Mutex<Option<PacketConnection>>>,
        directory: &str,
    ) -> Result<(), Error> {
        let data = connection
            .lock()
            .unwrap()
            .as_mut()
            .ok_or(Error::NoConnection)?
            .receive()?;
        let header: Header = *from_bytes(&data[..size_of::<Header>()]);
        let mut cursor = size_of::<Header>();
        let sub_path: &str = str::from_utf8(&data[cursor..cursor + header.sub_path_length()])?;
        cursor += header.sub_path_length();
        let file_content = &data[cursor..cursor + header.file_content_length()];
        let file_path = format!("{directory}\\{sub_path}");
        if let Some(sub_directory) = Path::new(&file_path).parent() {
            create_dir_all(sub_directory)?;
        };
        let mut file = File::create(file_path)?;
        file.write_all(file_content)?;
        file.flush()?;
        Ok(())
    }
}
