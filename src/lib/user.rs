use crate::{
    connect::ConnectionWrite,
    types::{ErrorType, Nick, Reply},
};
use std::fmt::Debug;

pub struct User {
    id: String,
    connection_write: ConnectionWrite,
    nick: Option<String>,
    real_name: Option<String>,
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
        self.nick = Some(nick);
    }

    pub fn set_real_name(&mut self, real_name: String) {
        self.real_name = Some(real_name);
    }

    pub fn is_set_nick(&self) -> bool {
        self.nick.is_some()
    }

    pub fn is_set_real_name(&self) -> bool {
        self.real_name.is_some()
    }

    pub fn is_registered(&self) -> bool {
        self.is_set_nick() && self.is_set_real_name()
    }

    pub fn send(&mut self, reply: Reply) {
        self.connection_write
            .write_message(&format!("{}", reply))
            .unwrap();
    }

    pub fn send_error(&mut self, error_type: ErrorType) {
        self.connection_write
            .write_message(&format!("{}", error_type))
            .unwrap();
    }
}
