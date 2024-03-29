use anyhow::{anyhow, Result};
use clap::Parser;
use iris_lib::{
    channel_list::ChannelList,
    connect::{ConnectionError, ConnectionManager},
    massage_sender::{error_msg_sender, global_msg_sender},
    types::{ErrorType, Message, Nick, ParsedMessage, UnparsedMessage, SERVER_NAME},
    user::{User, UserList},
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
        .with_utc_timestamps()
        .init()
        .expect("Failed to initialize logger!");

    let arguments = Arguments::parse();
    info!(
        "Launching {} at {}:{}",
        SERVER_NAME, arguments.ip_address, arguments.port
    );

    let mut user_list = UserList::new();

    let mut connection_manager = ConnectionManager::launch(arguments.ip_address, arguments.port);

    // Channel
    let (sender, receiver) = std::sync::mpsc::channel::<Result<ParsedMessage, (ErrorType, Nick)>>();

    // Thread to send messages
    {
        // The clone is needed because the thread will take ownership of the variable
        let mut user_list = user_list.clone();

        std::thread::spawn(move || {
            let mut channel_list = ChannelList::new();

            for msg in receiver {
                if let Ok(parsed_msg) = msg {
                    let sender_nick = parsed_msg.sender_nick.clone();
                    if let Err(err) = global_msg_sender(&mut user_list, &mut channel_list, parsed_msg) {
                        error!("Error when handling message: {}", err);
                        error_msg_sender(err, &user_list, sender_nick);
                    } else {
                        debug!("Message handled successfully!");
                    }
                } else if let Err((err, nick)) = msg {
                    let err = anyhow!(err);
                    error!("Error when parsing message: {}", err);
                    error_msg_sender(err, &user_list, nick);
                }
            }
        });
    }

    loop {
        // This function call will block until a new client connects!
        let (mut conn_read, conn_write) = connection_manager.accept_new_connection();

        let user = User::new(conn_read.id(), conn_write);

        user_list.add_user(user);

        info!("New connection from {}", conn_read.id());

        {
            let user_list = user_list.clone();
            let sender = sender.clone();

            std::thread::spawn(move || {
                loop {
                    // debug!("Waiting for message...");
                    let message = match conn_read.read_message() {
                        Ok(message) => message,
                        Err(
                            ConnectionError::ConnectionLost | ConnectionError::ConnectionClosed,
                        ) => {
                            warn!("Lost connection.");
                            break;
                        }
                        Err(_) => {
                            debug!("Invalid message received... ignoring message.");
                            continue;
                        }
                    };

                    debug!("Received message: {message}");

                    // Get the user's nick by id
                    let users = user_list.get_users();
                    let users = users.lock().expect("Failed to lock users list!");
                    let user = users
                        .iter()
                        .find(|user| user.get_id() == conn_read.id())
                        .expect("Failed to find user!");
                    let user_nick = user.get_nick();

                    // Parse the message
                    let parsed_msg = match ParsedMessage::try_from(UnparsedMessage {
                        sender_nick: user_nick.clone(),
                        message: &message,
                    }) {
                        Ok(parsed_msg) => parsed_msg,
                        Err(err) => {
                            sender.send(Err((err, user_nick))).expect("The channel is closed!");
                            debug!("Invalid message received... ignoring message.");
                            continue;
                        }
                    };

                    debug!("Parsed message: {:?}", parsed_msg);

                    // Drop the lock before sending the message
                    let is_registered = user.is_set_nick() && user.is_set_real_name();
                    drop(users);

                    sender.send(Ok(parsed_msg.clone())).expect("The channel is closed!");

                    // Check if the user is quitting
                    // If so, quit the thread
                    if let Message::Quit(_) = parsed_msg.message {
                        // Check if the user has a nick and a real name
                        if is_registered {
                            info!("User {} has quit.", user_nick);
                            break;
                        }
                    }
                }
            });
        }
    }
}
