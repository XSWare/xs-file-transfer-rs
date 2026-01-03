#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod command_resolver;
mod connection_control;
mod header;
mod ui;

use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use bytemuck::{bytes_of, from_bytes};
use xs_rust_library::connection::Connection;
use xs_rust_library::network::packet_connection::PacketConnection;

use crate::command_resolver::CommandResolver;
use crate::connection_control::ConnectionControl;
use crate::header::Header;

struct Controls {
    connection_control: ConnectionControl,
}

pub fn run() {
    let controls = Controls {
        connection_control: ConnectionControl::default(),
    };
    ui::show(controls).unwrap();
    // let mut command_resolver = CommandResolver::new();
    // match command_resolver.read_next_command().unwrap() {
    //     command_resolver::Command::Send(args) => {
    //         let stream = TcpStream::connect("127.0.0.1:3648").unwrap();
    //         let connection = PacketConnection::new(stream, 1024);
    //         send_file(connection, &args[0], &args[1]);
    //     }
    //     command_resolver::Command::Receive(directory) => {
    //         let listener = TcpListener::bind("0.0.0.0:3648").unwrap();
    //         let stream = listener.accept().unwrap().0;
    //         let connection = PacketConnection::new(stream, 1024);
    //         receive_file(connection, &directory);
    //     }
    // }
}

fn send_file(mut connection: PacketConnection, directory: &str, sub_path: &str) {
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

fn receive_file(mut connection: PacketConnection, directory: &str) {
    let data = connection.receive().unwrap();
    let header: Header = *from_bytes(&data[..size_of::<Header>()]);
    let mut cursor = size_of::<Header>();
    let sub_path: &str = str::from_utf8(&data[cursor..cursor + header.sub_path_length()]).unwrap();
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
