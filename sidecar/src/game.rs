use crate::nbt;

use std::{
    process::Command,
    fs::File,
    io::{self, BufRead}
};


pub struct Message {
    pub username: String,
    pub command: String,
    pub args: String
}

struct GameData {
    kit_items: Vec<String>,
}
pub struct CommandRouter {
    tmux_session_name: String,
    game_cache: GameData,
}

//TODO: logging
impl CommandRouter {
    pub fn new(tmux_session_name: String) -> io::Result<Self> {
        let mut kit_items = Vec::new();

        let file = File::open("src/kit_items.txt").expect("kit_items.txt not found");

        for line in io::BufReader::new(file).lines() {
            let line = line.expect("failed while reading line");
            kit_items.push(line); 
        }

        //TODO: read usercache.json, create hashmap of Username to UUID

        Ok(CommandRouter {
            tmux_session_name,
            game_cache: GameData { kit_items },
        })
    }

    pub fn run_command(&self, message: &Message) {
        match &message.command[1..] {
            "kit" => self.run_kit(&message.username),
            "sethome" => self.run_sethome(&message.username),
            _ => ()
        } 
    }

    fn run_kit(&self, username: &str) {
        for item in &self.game_cache.kit_items {
            let minecraft_command = &format!("give {} {}", username, item);
            self._execute_in_server(minecraft_command);
        }
    }

    fn run_sethome(&self, username: &str) {
        // TODO: look up UUID from username
        let coords = nbt::get_player_coords(username);
    }

    fn run_home(&self, username: &str) {
        //TODO: 
    }

    fn _execute_in_server(&self, minecraft_command: &str) {
        let status = Command::new("tmux")
            .args(["send-keys", "-t", &self.tmux_session_name, minecraft_command, "C-m"])
            .status();

        match status {
            Ok(status) if status.success() => (),
            Err(e) => eprintln!("Error sending command to tmux: {}", e),
            _ => eprint!("Other non zero exit"),
        }
    }
}