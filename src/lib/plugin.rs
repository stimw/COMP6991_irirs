use crate::{
    channel_list::ChannelList,
    types::{ErrorType, Nick, PrivMsg, PrivReply, Reply, Target},
    user::UserList,
};
use anyhow::{anyhow, Result};

pub fn plugin_handler(
    user_list: &mut UserList,
    _channel_list: &mut ChannelList,
    target_nick: Nick,
    receiver_nick: Nick,
    message_str: &str,
) -> Result<()> {
    match target_nick.0.as_str() {
        "use_plugin_reminder" => use_plugin_reminder(user_list, message_str, receiver_nick),
        _ => Err(anyhow!(ErrorType::PluginCommandError)),
    }?;

    Ok(())
}

// PRIVMSG plugin_reminder :1 test
fn use_plugin_reminder(
    user_list: &mut UserList,
    message_str: &str,
    receiver_nick: Nick,
) -> Result<()> {
    let plugin_nick = Nick("reminder".to_owned());

    let mut iter = message_str.splitn(2, ' ');

    let seconds = iter
        .next()
        .ok_or(anyhow!(ErrorType::PluginCommandError))?
        .parse::<u64>()
        .map_err(|_| anyhow!(ErrorType::PluginCommandError))?;

    let sentence = iter.next().ok_or(anyhow!(ErrorType::PluginCommandError))?;

    let user_list = user_list.clone();
    let sentence = sentence.to_owned();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(seconds));

        let users = user_list.get_users();
        let mut users = users.lock().expect("Failed to lock users");
        let user = users
            .iter_mut()
            .find(|user| user.get_nick() == receiver_nick.clone())
            .expect("Failed to find user");

        user.send(Reply::PrivMsg(PrivReply {
            sender_nick: plugin_nick.clone(),
            message: PrivMsg {
                target: Target::User(receiver_nick.clone()),
                message: sentence.to_owned(),
            },
        }))
        .expect("Plugin Reminder: Failed to send the message");
    });

    Ok(())
}
