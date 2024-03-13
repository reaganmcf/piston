use std::{
    io::{BufReader, Write},
    path::Path,
};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::{Deserialize, Serialize};

pub mod messages;

pub struct IpcReader {
    listener: LocalSocketListener,
}

const SOCKET_PATH: &str = "/tmp/piston-ipc.sock";

impl IpcReader {
    pub fn new() -> std::io::Result<Self> {
        let path = Path::new(SOCKET_PATH);

        Ok(Self {
            listener: LocalSocketListener::bind(path.to_path_buf())?,
        })
    }

    pub fn receive<T: for<'de> Deserialize<'de>>(&mut self) -> std::io::Result<T> {
        let stream = self.listener.accept()?;
        let reader = BufReader::new(stream);
        let message = serde_json::from_reader(reader)?;
        Ok(message)
    }
}

impl Drop for IpcReader {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

#[derive(Debug)]
pub struct IpcWriter {
    connection: LocalSocketStream,
}

impl IpcWriter {
    pub fn new() -> std::io::Result<Self> {
        let path = Path::new(SOCKET_PATH);

        Ok(Self {
            connection: LocalSocketStream::connect(path)?,
        })
    }

    pub fn send<T: Serialize>(&mut self, message: &T) -> std::io::Result<()> {
        let message = serde_json::to_string(message)?;
        println!("Sending: {}", message);
        self.connection.write_all(message.as_bytes())?;
        self.connection.flush().expect("Failed to flush");

        Ok(())
    }
}
