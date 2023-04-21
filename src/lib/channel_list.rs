//! This module contains the ChannelList struct which is used to 
//! keep track of which users are in which channels.
use std::collections::HashMap;

/// This struct is used to keep track of which users are in which channels.
pub struct ChannelList {
    channels: HashMap<String, Vec<String>>,
}

impl Default for ChannelList {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelList {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn has_channel(&self, channel_name: &str) -> bool {
        self.channels.contains_key(channel_name)
    }

    pub fn has_user(&self, channel_name: &str, user_id: &str) -> bool {
        if !self.has_channel(channel_name) {
            return false;
        }
        
        // it's ok to use unwrap here because we already checked that the channel exists
        let channel = self.channels.get(channel_name).unwrap();
        channel.contains(&user_id.to_owned())
    }

    pub fn add_channel(&mut self, channel_name: String) {
        if self.has_channel(&channel_name) {
            return;
        }

        self.channels.insert(channel_name, Vec::new());
    }

    pub fn get_users_mut(&mut self, channel_name: &str) -> Option<&mut Vec<String>> {
        self.channels.get_mut(channel_name)
    }

    pub fn join_channel(&mut self, channel_name: &str, user_id: &str) {
        if !self.has_channel(channel_name) {
            self.add_channel(channel_name.to_owned());
        }

        // it's ok to use unwrap here because we already checked that the channel exists
        let channel_users = self.get_users_mut(channel_name).unwrap();
        channel_users.push(user_id.to_owned());
    }

    pub fn part_channel(&mut self, channel_name: &str, user_id: &str) {
        if !self.has_channel(channel_name) {
            return;
        }

        // it's ok to use unwrap here because we already checked that the channel exists
        let channel_users = self.get_users_mut(channel_name).unwrap();
        channel_users.retain(|id| id != user_id);
    }

    pub fn remove_user(&mut self, user_id: &str) {
        for (_, channel_users) in self.channels.iter_mut() {
            channel_users.retain(|id| id != user_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_channel() {
        let mut channel_list = ChannelList::new();
        channel_list.add_channel("channel1".to_string());

        assert!(channel_list.has_channel("channel1"));
        assert!(!channel_list.has_channel("channel2"));
    }

    #[test]
    fn test_join_channel() {
        let mut channel_list = ChannelList::new();
        channel_list.join_channel("channel1", "user1");

        assert!(channel_list.has_channel("channel1"));
        assert!(channel_list.has_user("channel1", "user1"));
    }

    #[test]
    fn test_part_channel() {
        let mut channel_list = ChannelList::new();
        channel_list.join_channel("channel1", "user1");
        channel_list.join_channel("channel1", "user2");
        channel_list.part_channel("channel1", "user1");

        assert!(channel_list.has_channel("channel1"));
        assert!(!channel_list.has_user("channel1", "user1"));
        assert!(channel_list.has_user("channel1", "user2"));
    }

    #[test]
    fn test_remove_user() {
        let mut channel_list = ChannelList::new();
        channel_list.join_channel("channel1", "user1");
        channel_list.join_channel("channel2", "user1");
        channel_list.remove_user("user1");

        assert!(!channel_list.has_user("channel1", "user1"));
        assert!(!channel_list.has_user("channel2", "user1"));
    }

    #[test]
    fn test_has_channel() {
        let mut channel_list = ChannelList::new();
        channel_list.add_channel("channel1".to_string());

        assert!(channel_list.has_channel("channel1"));
        assert!(!channel_list.has_channel("channel2"));
    }

    #[test]
    fn test_has_user() {
        let mut channel_list = ChannelList::new();
        channel_list.join_channel("channel1", "user1");

        assert!(channel_list.has_user("channel1", "user1"));
        assert!(!channel_list.has_user("channel1", "user2"));
    }
}
