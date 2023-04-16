use clap::Parser;
use iris_lib::{
    connect::{ConnectionError, ConnectionManager},
    types::SERVER_NAME,
};
use simple_logger::SimpleLogger;
use std::net::IpAddr;

#[macro_use]
extern crate log;

#[derive(Parser)]
struct Arguments {
    #[clap(default_value = "127.0.0.1")]
    ip_address: IpAddr,

    #[clap(default_value = "6991")]
    port: u16,
}

fn main() {
    SimpleLogger::new()
        .env()
        .with_local_timestamps()
        .init()
        .unwrap();

    let arguments = Arguments::parse();
    info!(
        "Launching {} at {}:{}",
        SERVER_NAME, arguments.ip_address, arguments.port
    );

    let mut connection_manager = ConnectionManager::launch(arguments.ip_address, arguments.port);
    loop {
        // This function call will block until a new client connects!
        let (mut conn_read, mut conn_write) = connection_manager.accept_new_connection();

        info!("New connection from {}", conn_read.id());

        loop {
            debug!("Waiting for message...");
            let message = match conn_read.read_message() {
                Ok(message) => message,
                Err(ConnectionError::ConnectionLost | ConnectionError::ConnectionClosed) => {
                    warn!("Lost connection.");
                    break;
                }
                Err(_) => {
                    debug!("Invalid message received... ignoring message.");
                    continue;
                }
            };

            debug!("Received message: {message}");

            let _ = conn_write.write_message("Hello, World!\r\n");
            debug!("Sent hello-world message back!");
        }
    }
}
