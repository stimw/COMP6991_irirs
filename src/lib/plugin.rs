//! Plugin system
//! 
//! This module is used to handle plugins.
//! 
//! Please refer to the `plugin_handler` function for more information.
use crate::{
    channel_list::ChannelList,
    types::{ErrorType, Nick, PrivMsg, PrivReply, Reply, Target},
    user::UserList,
};
use anyhow::{anyhow, Result};

/// Plugin handler
/// 
/// If you want to add a new plugin, add a new match statement.
/// 
/// You can see an example of a plugin in the `use_plugin_sample` function.
/// 
/// `"use_plugin_sample" => use_plugin_sample(user_list, message_str, receiver_nick)`
/// 
/// The `"use_plugin_sample"` is the command that will trigger the plugin.
/// You can change it to whatever you want.
/// 
/// **But make sure that the command prefix is `"use_plugin_"`.**
/// 
/// Because the plugin system will check if the command starts with `"use_plugin_"`.
/// And it will not conflict with the IRC commands due to the prefix is longer than a user's nick.
pub fn plugin_handler(
    user_list: &mut UserList,
    _channel_list: &mut ChannelList,
    target_nick: Nick,
    receiver_nick: Nick,
    message_str: &str,
) -> Result<()> {
    match target_nick.0.as_str() {
        "use_plugin_sample" => use_plugin_sample(user_list, message_str, receiver_nick),
        "use_plugin_reminder" => use_plugin_reminder(user_list, message_str, receiver_nick),
        _ => Err(anyhow!(ErrorType::PluginCommandError)),
    }?;

    Ok(())
}

/// # Plugin sample
/// 
/// This plugin just repeats the message.
/// 
/// For example, if you send "PRIVMSG use_plugin_sample :test",
/// the plugin will send back "test" to the user.
/// 
/// You can use this plugin as a template to create your own plugins.
pub fn use_plugin_sample(
    user_list: &mut UserList,
    message_str: &str,
    receiver_nick: Nick,
) -> Result<()> {
    // Set the plugin nick, which will show up in the message
    let plugin_nick = Nick("plugin_sample".to_owned());

    // Clone the user list and the message string for the thread
    let user_list = user_list.clone();
    let message_str = message_str.to_owned();
    // Spawn a new thread to send the message
    std::thread::spawn(move || {
        // Get the user list
        let users = user_list.get_users();
        // Lock the user list
        let mut users = users.lock().expect("Failed to lock users");
        // Find the user by nick
        let user = users
            .iter_mut()
            .find(|user| user.get_nick() == receiver_nick.clone())
            .expect("Failed to find user");

        // Send the message
        user.send(Reply::PrivMsg(PrivReply {
            sender_nick: plugin_nick.clone(),
            message: PrivMsg {
                target: Target::User(receiver_nick.clone()),
                message: message_str.to_owned(),
            },
        }))
        .expect("Plugin Sample: Failed to send the message");
    });
    Ok(())
}

// PRIVMSG plugin_reminder :1 test
fn use_plugin_reminder(
    user_list: &mut UserList,
    message_str: &str,
    receiver_nick: Nick,
) -> Result<()> {
    let plugin_nick = Nick("plugin_reminder".to_owned());

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
