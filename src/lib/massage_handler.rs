use crate::{
    channel_list::ChannelList,
    connect::ConnectionWrite,
    types::{self, ErrorType, Nick, NickMsg, Reply, WelcomeReply},
    user::UserList,
};

pub fn error_msg_handler(err: ErrorType, user_list: &UserList, sender_nick: Nick) {
    let users = user_list.get_users();
    let mut users = users.lock().unwrap();
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == sender_nick)
        .unwrap();
    user.send_back_error(err);
}

pub fn global_msg_handler(
    user_list: &mut UserList,
    channel_list: &mut ChannelList,
    parsed_msg: types::ParsedMessage,
) -> Result<(), ErrorType> {
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
        _ => Ok(()),
    }
}

fn nick_msg_handler(
    user_list: &mut UserList,
    nick_msg: NickMsg,
    user_id_as_nick: Nick,
) -> Result<(), ErrorType> {
    let nick = nick_msg.nick;

    let users = user_list.get_users();
    let mut users = users.lock().unwrap();

    // Check if nick exists and if it does, return an error
    if users.iter().any(|user| user.get_nick() == nick) {
        return Err(ErrorType::NickCollision);
    }

    // Find the user by user id
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == user_id_as_nick)
        .unwrap();

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
) -> Result<(), ErrorType> {
    let users = user_list.get_users();
    let mut users = users.lock().unwrap();
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == user_id_as_nick)
        .unwrap();

    if user.is_set_nick() && !user.is_set_real_name() {
        user.set_real_name(user_msg.real_name);

        user.send(Reply::Welcome(WelcomeReply {
            target_nick: user.get_real_name(),
            message: format!("Welcome to the server, {}!", user.get_real_name()),
        }));
    }

    Ok(())
}

fn ping_msg_handler(
    user_list: &mut UserList,
    ping_msg: String,
    sender_nick: Nick,
) -> Result<(), ErrorType> {
    let users = user_list.get_users();
    let mut users = users.lock().unwrap();
    let user = users
        .iter_mut()
        .find(|user| user.get_nick() == sender_nick)
        .unwrap();

    if user.is_set_nick() && user.is_set_real_name() {
        user.send(Reply::Pong(ping_msg));
    }

    Ok(())
}
