use crate::{
    channel_list::ChannelList,
    types::{self, ErrorType, Nick, NickMsg, PrivMsg, QuitReply, Reply, Target, WelcomeReply, PrivReply, JoinMsg, JoinReply, Channel, PartMsg, PartReply},
    user::UserList,
};
use anyhow::{anyhow, Error, Result};
use log::error;

pub fn global_msg_sender(
    user_list: &mut UserList,
    channel_list: &mut ChannelList,
    parsed_msg: types::ParsedMessage,
) -> Result<()> {
    match parsed_msg.message {
        types::Message::Nick(nick_msg) => {
            nick_msg_sender(user_list, nick_msg, parsed_msg.sender_nick)
        }
        types::Message::User(user_msg) => {
            user_msg_sender(user_list, user_msg, parsed_msg.sender_nick)
        }
        types::Message::Ping(ping_msg) => {
            ping_msg_sender(user_list, ping_msg, parsed_msg.sender_nick)
        }
        types::Message::Quit(quit_msg) => {
            quit_msg_sender(user_list, channel_list, quit_msg, parsed_msg.sender_nick)
        }
        types::Message::PrivMsg(priv_msg) => {
            priv_msg_sender(user_list, channel_list, priv_msg, parsed_msg.sender_nick)
        }
        types::Message::Join(join_msg) => {
            join_msg_sender(user_list, channel_list, join_msg, parsed_msg.sender_nick)
        }
        types::Message::Part(part_msg) => {
            part_msg_sender(user_list, channel_list, part_msg, parsed_msg.sender_nick)
        }
    }
}

pub fn error_msg_sender(err: Error, user_list: &UserList, sender_nick: Nick) {
    if let Some(err) = err.downcast_ref::<ErrorType>() {
        let users = user_list.get_users();
        let mut users = users.lock().expect("Failed to lock users");
        let user = users
            .iter_mut()
            .find(|user| user.get_nick() == sender_nick)
            .expect("Failed to find user");
        user.send_back_error(*err)
            .expect("Failed to send back error");
    } else {
        error!("Server Error: {}", err);
    }
}

fn nick_msg_sender(
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

fn user_msg_sender(
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

fn ping_msg_sender(user_list: &mut UserList, ping_msg: String, sender_nick: Nick) -> Result<()> {
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

fn quit_msg_sender(
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

fn priv_msg_sender(
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

            let channel_users = channel_list
                .get_users_mut(&channel.0)
                .ok_or(anyhow!("channel_users not found"))?;

            for other_user_nick in &mut *channel_users {
                let other_user = users
                    .iter_mut()
                    .find(|user| user.get_nick() == Nick(other_user_nick.clone()))
                    .ok_or(anyhow!("User not found"))?;

                other_user.send(Reply::PrivMsg(PrivReply {
                    message: PrivMsg {
                        target: Target::Channel(channel.clone()),
                        message: priv_msg.message.clone(),
                    },
                    sender_nick: sender_nick.clone(),
                }))?;
            }
        }

        Target::User(user_nick) => {
            let other_user_option = users
                .iter_mut()
                .find(|user| user.get_nick() == user_nick);

            if let Some(other_user) = other_user_option {
                other_user.send(Reply::PrivMsg(PrivReply {
                    message: PrivMsg {
                        target: Target::User(user_nick),
                        message: priv_msg.message.clone(),
                    },
                    sender_nick: sender_nick.clone(),
                }))?;
            } else {
                return Err(anyhow!(ErrorType::NoSuchNick));
            }
        }
    }

    Ok(())
}

fn join_msg_sender(
    user_list: &mut UserList,
    channel_list: &mut ChannelList,
    join_msg: JoinMsg,
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

    let channel = join_msg.channel.0;

    // create channel if it does not exist
    if !channel_list.has_channel(&channel) {
        channel_list.add_channel(channel.clone());
    }

    // add user to channel
    channel_list.join_channel(&channel, &sender_nick.0);

    // add channel to user
    user.join_channel(&channel);

    // send join message to all users in channel
    let channel_users = channel_list
        .get_users_mut(&channel)
        .ok_or(anyhow!("channel_users not found"))?;

    for other_user_nick in &mut *channel_users {
        let other_user = users
            .iter_mut()
            .find(|user| user.get_nick() == Nick(other_user_nick.clone()))
            .ok_or(anyhow!("User not found"))?;

        other_user.send(Reply::Join(JoinReply {
            message: JoinMsg {
                channel: Channel(channel.clone()),
            },
            sender_nick: sender_nick.clone(),
        }))?;
    }

    Ok(())
}

fn part_msg_sender(
    user_list: &mut UserList,
    channel_list: &mut ChannelList,
    part_msg: PartMsg,
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

    let channel = part_msg.channel.0;

    // error if channel does not exist
    if !channel_list.has_channel(&channel) {
        return Err(anyhow!(ErrorType::NoSuchChannel));
    }

    // error if user is not in channel, do nothing
    if !channel_list.has_user(&channel, &sender_nick.0) {
        return Ok(());
    }

    // remove user from channel
    channel_list.part_channel(&channel, &sender_nick.0);

    // remove channel from user
    user.part_channel(&channel);

    // send part message to all users in channel
    let channel_users = channel_list
        .get_users_mut(&channel)
        .ok_or(anyhow!("channel_users not found"))?;

    for other_user_nick in &mut *channel_users {
        let other_user = users
            .iter_mut()
            .find(|user| user.get_nick() == Nick(other_user_nick.clone()))
            .ok_or(anyhow!("User not found"))?;

        other_user.send(Reply::Part(PartReply {
            message: PartMsg {
                channel: Channel(channel.clone()),
            },
            sender_nick: sender_nick.clone(),
        }))?;
    }

    Ok(())
}