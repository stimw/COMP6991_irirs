use crate::{
    channel_list::ChannelList,
    connect::ConnectionWrite,
    types::{self, ErrorType, Nick, NickMsg, PrivMsg, QuitReply, Reply, Target, WelcomeReply},
    user::UserList,
};
use anyhow::{anyhow, Error, Result};
use log::error;

pub fn error_msg_handler(err: Error, user_list: &UserList, sender_nick: Nick) {
    if let Some(err) = err.downcast_ref::<ErrorType>() {
        let users = user_list.get_users();
        let mut users = users.lock().expect("Failed to lock users");
        let user = users
            .iter_mut()
            .find(|user| user.get_nick() == sender_nick)
            .expect("Failed to find user");
        user.send_back_error(*err).expect("Failed to send back error");
    } else {
        error!("Server Error: {}", err);
    }
}

pub fn global_msg_handler(
    user_list: &mut UserList,
    channel_list: &mut ChannelList,
    parsed_msg: types::ParsedMessage,
) -> Result<()> {
    match parsed_msg.message {
        types::Message::Nick(nick_msg) => {
            nick_msg_handler(user_list, nick_msg, parsed_msg.sender_nick)
        }
        types::Message::User(user_msg) => {
            user_msg_handler(user_list, user_msg, parsed_msg.sender_nick)
        }
        types::Message::Ping(ping_msg) => {
            ping_msg_handler(user_list, ping_msg, parsed_msg.sender_nick)
        }
        types::Message::Quit(quit_msg) => {
            quit_msg_handler(user_list, channel_list, quit_msg, parsed_msg.sender_nick)
        }
        _ => Ok(()),
    }
}

fn nick_msg_handler(
    user_list: &mut UserList,
    nick_msg: NickMsg,
    user_id_as_nick: Nick,
) -> Result<()> {
    let nick = nick_msg.nick;

    let users = user_list.get_users();
    let mut users = users.lock().expect("Failed to lock users");

    // Check if nick exists and if it does, return an error
    if users.iter().any(|user| user.get_nick() == nick) {
        return Err(anyhow!(ErrorType::NickCollision));
    }

    // Find the user by user id
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == user_id_as_nick)
        .ok_or(anyhow!("User not found"))?;

    // Check if the nick is valid, if not, return an error
    let nick = Nick::try_from(nick.0)?;
    // Set the nick
    user.set_nick(nick.0);

    Ok(())
}

fn user_msg_handler(
    user_list: &mut UserList,
    user_msg: types::UserMsg,
    user_id_as_nick: Nick,
) -> Result<()> {
    let users = user_list.get_users();
    let mut users = users.lock().expect("Failed to lock users");
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == user_id_as_nick)
        .ok_or(anyhow!("User not found"))?;

    if user.is_set_nick() && !user.is_set_real_name() {
        user.set_real_name(user_msg.real_name);

        user.send(Reply::Welcome(WelcomeReply {
            target_nick: user.get_real_name(),
            message: format!("Welcome to the server, {}!", user.get_real_name()),
        }))?;
    }

    Ok(())
}

fn ping_msg_handler(user_list: &mut UserList, ping_msg: String, sender_nick: Nick) -> Result<()> {
    let users = user_list.get_users();
    let mut users = users.lock().expect("Failed to lock users");
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == sender_nick)
        .ok_or(anyhow!("User not found"))?;

    if user.is_set_nick() && user.is_set_real_name() {
        user.send(Reply::Pong(ping_msg))?;
    }

    Ok(())
}

fn quit_msg_handler(
    user_list: &mut UserList,
    channel_list: &mut ChannelList,
    quit_msg: types::QuitMsg,
    sender_nick: Nick,
) -> Result<()> {
    let users = user_list.get_users();
    let mut users = users.lock().expect("Failed to lock users");
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == sender_nick)
        .ok_or(anyhow!("User not found"))?;

    if !user.is_set_nick() || !user.is_set_real_name() {
        return Ok(());
    }

    let channels = user.get_joined_channels().clone();

    // send quit message to all channels
    for channel_str in channels {
        let channel_users = channel_list
            .get_users_mut(&channel_str)
            .ok_or(anyhow!("channel_users not found"))?;

        for other_user_nick in &mut *channel_users {
            let other_user = users
                .iter_mut()
                .find(|user| user.get_nick() == Nick(other_user_nick.clone()))
                .ok_or(anyhow!("User not found"))?;

            other_user.send(Reply::Quit(QuitReply {
                message: quit_msg.clone(),
                sender_nick: sender_nick.clone(),
            }))?;
        }

        // remove user from channel
        channel_users.retain(|user_nick| user_nick != &sender_nick.0);
    }

    // remove user from user list
    users.retain(|user| user.get_nick() != sender_nick);

    Ok(())
}

fn private_msg_handler(
    user_list: &mut UserList,
    channel_list: &mut ChannelList,
    priv_msg: PrivMsg,
    sender_nick: Nick,
) -> Result<()> {
    let users = user_list.get_users();
    let mut users = users.lock().expect("Failed to lock users");
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == sender_nick)
        .ok_or(anyhow!("User not found"))?;

    if !user.is_set_nick() || !user.is_set_real_name() {
        return Ok(());
    }

    match priv_msg.target {
        Target::Channel(channel) => {
            // error if channel does not exist
            if !channel_list.has_channel(&channel.0) {
                return Err(anyhow!(ErrorType::NoSuchChannel));
            }

            // ignore if user is not in channel
            if !channel_list.has_user(&channel.0, &sender_nick.0) {
                return Ok(());
            }
        }

        Target::User(user_nick) => {}
    }

    Ok(())
}
