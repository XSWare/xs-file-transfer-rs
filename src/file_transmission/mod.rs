mod header;

use std::{
    fs::{File, create_dir_all},
    io::{Read, Write},
    path::Path,
};

use bytemuck::{bytes_of, from_bytes};
use xs_rust_library::{connection::Connection, packet_connection::PacketConnection};

use crate::file_transmission::header::Header;

pub struct FileTransmission;

impl FileTransmission {
    /// send a file. `sub_path` can be the file name or a subdirectory with the filename.
    /// the subdirectoy is used to restore the same file hierarchy at the destination.
    pub fn send_file(connection: &mut PacketConnection, directory: &str, sub_path: &str) {
        let file_path = format!("{directory}{sub_path}");
        let mut file = File::open(file_path).unwrap();
        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content).unwrap();
        let header = Header::new(file_content.len(), sub_path.len());
        let mut data = Vec::with_capacity(size_of::<Header>() + file_content.len());
        data.extend_from_slice(bytes_of(&header));
        data.extend_from_slice(sub_path.as_bytes());
        data.extend_from_slice(&file_content);
        connection.send(&data).unwrap();
    }

    pub fn receive_file(connection: &mut PacketConnection, directory: &str) {
        let data = connection.receive().unwrap();
        let header: Header = *from_bytes(&data[..size_of::<Header>()]);
        let mut cursor = size_of::<Header>();
        let sub_path: &str =
            str::from_utf8(&data[cursor..cursor + header.sub_path_length()]).unwrap();
        cursor += header.sub_path_length();
        let file_content = &data[cursor..cursor + header.file_content_length()];
        let file_path = format!("{directory}{sub_path}");
        if let Some(sub_directory) = Path::new(&file_path).parent() {
            create_dir_all(sub_directory).unwrap();
        };
        let mut file = File::create(file_path).unwrap();
        file.write_all(file_content).unwrap();
        file.flush().unwrap();
    }
}
