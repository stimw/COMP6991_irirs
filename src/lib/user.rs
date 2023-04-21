use crate::{
    connect::ConnectionWrite,
    types::{ErrorType, Nick, Reply},
};
use anyhow::Result;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

pub struct User {
    id: String,
    connection_write: ConnectionWrite,
    nick: Option<String>,
    real_name: Option<String>,
    joined_channels: Vec<String>,
}

impl Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("nick", &self.nick)
            .field("real_name", &self.real_name)
            .finish()
    }
}

impl User {
    pub fn new(id: String, connection_write: ConnectionWrite) -> Self {
        Self {
            id,
            connection_write,
            nick: None,
            real_name: None,
            joined_channels: Vec::new(),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_nick(&self) -> Nick {
        match &self.nick {
            Some(nick) => Nick(nick.clone()),
            None => Nick(self.get_id()),
        }
    }

    pub fn get_real_name(&self) -> Nick {
        match &self.real_name {
            Some(real_name) => Nick(real_name.clone()),
            None => Nick(self.get_id()),
        }
    }

    pub fn set_nick(&mut self, nick: String) {
        println!("{} set nick to {}", self.get_id(), nick);
        self.nick = Some(nick);
    }

    pub fn set_real_name(&mut self, real_name: String) {
        println!("{} set real name to {}", self.get_id(), real_name);
        self.real_name = Some(real_name);
    }

    pub fn is_set_nick(&self) -> bool {
        self.nick.is_some()
    }

    pub fn is_set_real_name(&self) -> bool {
        self.real_name.is_some()
    }

    pub fn send(&mut self, reply: Reply) -> Result<()> {
        self.connection_write.write_message(&format!("{}", reply))?;

        Ok(())
    }

    pub fn send_back_error(&mut self, err: ErrorType) -> Result<()> {
        self.connection_write
            .write_message(&format!("{}\r\n", err))?;

        Ok(())
    }

    pub fn join_channel(&mut self, channel_name: &str) {
        self.joined_channels.push(channel_name.to_owned());
    }

    pub fn part_channel(&mut self, channel_name: &str) {
        self.joined_channels.retain(|name| name != channel_name);
    }

    pub fn get_joined_channels(&self) -> &Vec<String> {
        &self.joined_channels
    }
}

pub struct UserList {
    users: Arc<Mutex<Vec<User>>>,
}

impl Clone for UserList {
    fn clone(&self) -> Self {
        Self {
            users: self.users.clone(),
        }
    }
}

impl Default for UserList {
    fn default() -> Self {
        Self::new()
    }
}

impl UserList {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.lock().expect("Failed to lock users").push(user);
    }

    pub fn get_users(&self) -> Arc<Mutex<Vec<User>>> {
        self.users.clone()
    }
}
