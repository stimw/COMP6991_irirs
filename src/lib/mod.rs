//! This is a simple IRC server written in Rust.
//! 
//! ## Plugin System
//! 
//! This IRC server supports plugins. You can create your own plugins and use them in the server.
//! 
//! ### How to use the built-in reminder plugin
//! 
//! To use the built-in reminder plugin, you need to send a message to the server with the following format:
//! 
//! ```text
//! PRIVMSG use_plugin_reminder :<seconds> <message>
//! ```
//! 
//! For example, if you send the following message to the server:
//! 
//! ```text
//! PRIVMSG use_plugin_reminder :5 test, test, test
//! ```
//! 
//! The server will send the message "test, test, test" to the user in 5 seconds.
//! 
//! ### How to create your own plugin
//! 
//! Please refer to the plugin documentation or check out the comments in the `plugin.rs` file.
pub mod connect;
pub mod types;
pub mod user;
pub mod channel_list;
pub mod massage_sender;
pub mod plugin;
