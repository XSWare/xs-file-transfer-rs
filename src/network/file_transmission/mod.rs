pub mod header;

use std::{
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::Path,
};

use bytemuck::{bytes_of, from_bytes};
use displaydoc::Display;
use thiserror::Error;

use header::Header;

#[derive(Error, Display, Debug)]
pub enum Error {
    /// File IO error: {0}
    FileIO(#[from] std::io::Error),
    /// Invalid file path: {0}
    InvalidFilePath(#[from] std::str::Utf8Error),
    /// File name not in a recognized format
    FilenameDecoding,
}

pub struct FileTransmission;

impl FileTransmission {
    /// send a file. `sub_path` can be the file name or a subdirectory with the filename.
    /// the subdirectoy is used to restore the same file hierarchy at the destination.
    pub fn create_packet_data_from_path(
        directory: &Path,
        sub_path: &Path,
    ) -> Result<Vec<u8>, Error> {
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
        Ok(data)
    }

    pub fn write_file_from_packet_data(packet_data: &[u8], directory: &Path) -> Result<(), Error> {
        let header: Header = *from_bytes(&packet_data[..size_of::<Header>()]);
        let mut cursor = size_of::<Header>();
        let sub_path = Path::new(str::from_utf8(
            &packet_data[cursor..cursor + header.sub_path_length()],
        )?);
        cursor += header.sub_path_length();
        let file_content = &packet_data[cursor..cursor + header.file_content_length()];
        let file_path = Path::join(directory, sub_path);
        if let Some(sub_directory) = file_path.parent() {
            create_dir_all(sub_directory)?;
        };
        let mut file = File::create(file_path)?;
        file.write_all(file_content)?;
        file.flush()?;
        Ok(())
    }
}
