use piston_ipc::{IpcReader, messages::{Ping, Pong, IpcMessage}};

fn main() {
    let mut reader = IpcReader::new().unwrap();
    let message = reader.receive::<IpcMessage>().unwrap();
    println!("Received: {:?}", message);
}
