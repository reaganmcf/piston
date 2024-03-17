use lazy_static::lazy_static;
use messages::IpcMessage;
use std::{
    io::{BufReader, Write},
    path::Path,
    sync::RwLock,
};

use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::Serialize;
use serde_json::Deserializer;

pub mod messages;

pub struct IpcReader {
    listener: LocalSocketListener,
}

const SOCKET_PATH: &str = "/tmp/piston-ipc.sock";

/* Every single writer writes to the same connection - not multiple connections at once */
lazy_static! {
    static ref SOCKET_WRITER_CONNECTION: RwLock<LocalSocketStream> = {
        let path = Path::new(SOCKET_PATH);

        let connection = LocalSocketStream::connect(path).expect("Failed to open socket");
        RwLock::new(connection)
    };
}

impl IpcReader {
    pub fn new() -> std::io::Result<Self> {
        let path = Path::new(SOCKET_PATH);

        Ok(Self {
            listener: LocalSocketListener::bind(path.to_path_buf())?,
        })
    }

    pub fn receive(&self) -> std::io::Result<()> {
        let stream = self.listener.accept()?;
        let reader = BufReader::new(stream);
        let deserializer = Deserializer::from_reader(reader).into_iter::<IpcMessage>();

        for message in deserializer {
            match message {
                Ok(msg) => println!("{:?}", msg),
                Err(e) => println!("Failed to deserialize, {:?}", e),
            }
        }

        Ok(())
    }
}

impl Drop for IpcReader {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

#[derive(Debug)]
pub struct IpcWriter {
    connection: &'static RwLock<LocalSocketStream>,
}

impl IpcWriter {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            connection: &SOCKET_WRITER_CONNECTION,
        })
    }

    pub fn send<T: Serialize>(&mut self, message: &T) -> std::io::Result<()> {
        let message = serde_json::to_string(message)?;
        println!("Sending: {}", message);
        self.connection
            .write()
            .expect("Failed to get writer")
            .write_all(message.as_bytes())?;
        self.connection
            .write()
            .expect("Failed to get writer")
            .flush()
            .expect("Failed to flush");

        Ok(())
    }
}
