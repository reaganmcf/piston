use std::{
    io::{BufReader, Write},
    os::unix::net::UnixStream,
    path::Path,
};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::{Deserialize, Serialize};
use tokio_serde::{formats::SymmetricalJson, SymmetricallyFramed};
use tokio_util::codec::{length_delimited, FramedWrite, LengthDelimitedCodec};

use crate::messages::IpcMessage;

pub mod messages;

pub struct IpcReader {
    listener: LocalSocketListener,
}

const SOCKET_PATH: &str = "/tmp/piston.sock";

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
    connection: UnixStream,
}

impl IpcWriter {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            connection: UnixStream::connect(SOCKET_PATH)?,
        })
    }

    pub fn send<T: Serialize + std::fmt::Debug>(&mut self, message: &T) -> std::io::Result<()> {
        println!("Sending: {:?}", message);
        let serialized = serde_json::to_vec(message)?;
        let length = serialized.len() as u32;
        let length_bytes = length.to_be_bytes();

        let mut frame = length_bytes.to_vec();
        frame.extend(serialized);

        self.connection.write_all(&frame)?;
        // self.connection.flush().expect("Failed to flush");
        // self.connection.shutdown(std::net::Shutdown::Write)?;

        Ok(())
    }
}
