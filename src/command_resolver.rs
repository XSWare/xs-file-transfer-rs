use std::io::stdin;

pub enum Command {
    Send([String; 2]),
    Receive(String),
    // Server,
    // Client,
    // Exit,
}

pub struct CommandResolver {
    cmd_buffer: String,
}

impl CommandResolver {
    pub fn new() -> Self {
        Self {
            cmd_buffer: String::new(),
        }
    }
    pub fn read_next_command(&mut self) -> std::io::Result<Command> {
        stdin().read_line(&mut self.cmd_buffer)?;
        let args: Box<[_]> = self.cmd_buffer.split(' ').map(|s| s.trim()).collect();
        let cmd = match args[0] {
            "send" => Command::Send([args[1].to_string(), args[2].to_string()]),
            "receive" => Command::Receive(args[1].to_string()),
            _ => panic!("not a valid command"),
        };

        Ok(cmd)
    }
}
