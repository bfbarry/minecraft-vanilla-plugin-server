use crate::nbt;

use std::{
    process::Command,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Write},
    // env,
    collections::HashMap,
    sync::RwLock,
};
use serde::Deserialize;

const USER_DATA_PATH: &str = "data/user_state.json";
const DEBUG_NO_SERVER: bool = true;
pub struct Message {
    pub username: String,
    pub command: String,
    pub args: String
}

type Username = String;
type Coordinates = Vec<f64>;
type UserState = HashMap<Username, Coordinates>;
struct CustomGameData {
    kit_items: Vec<String>,
    user_state: UserState, //for now just Username: coordinate
}
pub struct CommandRouter {
    tmux_session_name: String,
    game_cache: RwLock<CustomGameData>,
    name_to_uuid: HashMap<String, String>,
    // file_mutex: Mutex<()>,
}

#[derive(Debug, Deserialize)]
struct UserCacheEntry {
    name: String,
    uuid: String,
    expiresOn: String,
}

//TODO: logging
impl CommandRouter {
    pub fn new(tmux_session_name: String) -> io::Result<Self> {
        let mut kit_items = Vec::new();

        let file = File::open("data/kit_items.txt").expect("kit_items.txt not found");

        for line in io::BufReader::new(file).lines() {
            let line = line.expect("failed while reading line");
            kit_items.push(line); 
        }
        
        // let file_mutex = Mutex::new(()); // to lock cache json file reading
        let user_state_str = fs::read_to_string(USER_DATA_PATH).expect("couldn't find user_state.json");
        let user_state: UserState = serde_json::from_str(&user_state_str).expect("err deserializing UserState");

        let usercache_str = fs::read_to_string("../usercache.json").expect("couldn't find usercache.json");
        let players: Vec<UserCacheEntry> = serde_json::from_str(&usercache_str).expect("could not deserialize UserCacheEntry");
        let name_to_uuid: HashMap<String, String> = players
            .into_iter()
            .map(|p| (p.name, p.uuid))
            .collect();

        Ok(CommandRouter {
            tmux_session_name,
            game_cache: RwLock::new(CustomGameData { kit_items, user_state }),
            name_to_uuid,
            // file_mutex,
        })
    }

    pub fn run_command(&self, message: &Message) {
        match &message.command[1..] {
            "kit" => self.run_kit(&message.username),
            "sethome" => self.run_sethome(&message.username),
            "home" => self.run_home(&message.username),
            _ => ()
        } 
    }

    fn run_kit(&self, username: &str) {
        for item in &self.game_cache.read().unwrap().kit_items {
            let minecraft_command = &format!("give {} {}", username, item);
            self._execute_in_server(minecraft_command);
        }
    }

    fn run_sethome(&self, username: &str) {
        // TODO: look up UUID from username
        let uuid = match self.name_to_uuid.get(username) {
            Some(u) => u,
            None => {
                eprintln!("run_sethome Error: username not found in usercache '{}'", username);
                return;
            }
        };
        let coords = match nbt::get_player_coords(uuid) {
            Ok(coords) => coords,
            Err(e) => {
                eprintln!("run_sethome Error while reading player coords for player {} {}", username, e);
                return;
            }
        };

        let mut cache = self.game_cache.write().unwrap();
        cache.user_state.insert(username.to_string(), coords);

        let json_string = serde_json::to_string(&cache.user_state).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(USER_DATA_PATH)
            .expect("Failed to open file");
    
        file.write_all(json_string.as_bytes()).expect("Failed to write to file");

        drop(cache);

        self._execute_in_server(&format!("tell {} home coordinates updated", username));
        
    }

    fn run_home(&self, username: &str) {
        //TODO:
        let cache = self.game_cache.read().unwrap();
        let coords = match cache.user_state.get(username)  {
            Some(c) => c,
            None => {
                eprintln!("run_home Error: username not found in usercache '{}'", username);
                return;
            }
        };
        self._execute_in_server(&format!("tp {} {} {} {}", username, coords[0], coords[1], coords[2]));
    }

    fn _execute_in_server(&self, minecraft_command: &str) {
        if DEBUG_NO_SERVER {
            println!("{}", minecraft_command);
            return;
        }
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