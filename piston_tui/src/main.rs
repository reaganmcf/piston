use tokio::net::UnixListener;
use tokio_serde::formats::*;
use futures::prelude::*;

use piston_ipc::messages::IpcMessage;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};

#[tokio::main]
async fn main() {
    if let Ok(_) = std::fs::remove_file("/tmp/piston.sock") {
        println!("Removed existing socket file");
    }

    let listener = UnixListener::bind("/tmp/piston.sock").unwrap();
    let stream = listener.accept().await.unwrap().0;

    let length_delimited = FramedRead::new(stream, LengthDelimitedCodec::new());
    let mut deserialized = tokio_serde::SymmetricallyFramed::new(
        length_delimited,
        SymmetricalJson::<IpcMessage>::default(),
    );

    while let Some(msg) = deserialized.try_next().await.unwrap() {
        println!("Received message: {:?}", msg);
    }
}
