use std::collections::HashMap;

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

        let channel_users = self.get_users_mut(channel_name).unwrap();
        channel_users.push(user_id.to_owned());
    }

    pub fn part_channel(&mut self, channel_name: &str, user_id: &str) {
        if !self.has_channel(channel_name) {
            return;
        }

        let channel_users = self.get_users_mut(channel_name).unwrap();
        channel_users.retain(|id| id != user_id);
    }

    pub fn remove_user(&mut self, user_id: &str) {
        for (_, channel_users) in self.channels.iter_mut() {
            channel_users.retain(|id| id != user_id);
        }
    }
}
