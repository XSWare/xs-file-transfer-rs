use std::fs::{File, create_dir_all};
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str::from_utf8;
use std::{u8, thread};

use directories::ProjectDirs;
use xs_rust_library::network::packet_connection::PacketConnection;
use xs_rust_library::network::packet_receive_event::PacketReceiveEvent;

fn main() {
  let listener = TcpListener::bind("0.0.0.0:3648").unwrap();
  loop {
    let stream = listener.accept().unwrap().0;
    thread::spawn(|| {
      initialize_accepted_connection(stream);
    }); 
  }
}

fn initialize_accepted_connection(stream: TcpStream) {
  let connection = PacketConnection::new(stream, 1024);
  let mut receive_loop = PacketReceiveEvent::new(connection);

  let handler = Box::new(|data: &Vec<u8>| {
    println!("received {} bytes", data.len());
    read_file_from_xs(data);
  });

  let _sub = receive_loop.subscribe(handler);
  receive_loop.start();
}

fn read_file_from_xs(data: &Vec<u8>) {
  let mut cursor: usize = 0;

  let file_name_length = data[0] as usize;
  cursor += 1;

  let file_name = from_utf8(&data[cursor..cursor + file_name_length]).unwrap();
  cursor += file_name_length;

  println!("file name: {}", file_name);

  cursor += 8; // data size
  cursor += 1; // create file
  cursor += 1; // last chunk
  cursor += 1; // last file

  let mut chunk_size_array = [0 as u8; 4];
  chunk_size_array.clone_from_slice(&data[cursor..cursor + 4]);
  let chunk_size = i32::from_le_bytes(chunk_size_array) as usize;
  cursor += 4;

  let file_content = &data[cursor..cursor + chunk_size];
  println!("file content: {}", from_utf8(file_content).unwrap());

  let project_dirs = ProjectDirs::from("", "", "XSFileTransfer").unwrap();
  let directory = Path::new(project_dirs.config_dir().parent().unwrap());
  let filepath = directory.join(file_name);

  create_dir_all(directory).unwrap();
  let mut file = File::create(filepath).unwrap();
  file.write_all(file_content).unwrap();
}