use piston_ipc::{
    messages::{IpcMessage, Ping, Pong},
    IpcReader,
};

fn main() {
    let reader = IpcReader::new().unwrap();
    let message = reader.receive().unwrap();
    println!("Received: {:?}", message);
}
