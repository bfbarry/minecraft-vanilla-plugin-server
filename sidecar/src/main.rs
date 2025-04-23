use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
    process::Command,
};
use regex::Regex;

fn main() -> io::Result<()> {
    let log_path = "/Users/brianbarry/Desktop/computing/minecraft-vanilla-plugin-server/logs/dummy_log.log";
    let file = File::open(log_path)?;
    let mut reader = BufReader::new(file);

    let (sender, receiver) = mpsc::channel(); // communication w/ worker threads
    let receiver = Arc::new(Mutex::new(receiver));

    let file_monitor_handle = thread::spawn(move || {
        // let mut prev_line_count = 0; 

        loop {
            let mut buffer = String::new();
            let _ = reader.read_line(&mut buffer).expect("read line failed");

            if !buffer.is_empty() {
                sender.send(buffer).expect("failed to send message");
                // prev_line_count += 1;
            } else {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });
    let command_router = CommandRouter::new("mcserver".to_string()).unwrap(); //TODO unsafe
    let command_router = Arc::new(command_router);
    let mut worker_handles = Vec::new();

    for _ in 0..5 {
        let receiver = Arc::clone(&receiver);
        let command_router = Arc::clone(&command_router);
        let handle = thread::spawn(move || loop {
            let receiver = receiver.lock().unwrap();
            if let Ok(line) = receiver.recv() {
                let result = process_line(&line);
                if let Some(data) = result {
                    command_router.run_command(&data);
                }
                println!("processing line: {}", line);
            }
            drop(receiver);
            // thread::sleep() simulate work
        });
        worker_handles.push(handle);
    }

    file_monitor_handle.join().expect("File monitor panicked");

    for h in worker_handles {
        h.join().expect("Worker thread panicked");
    }

    Ok(())
}


struct Message {
    username: String,
    command: String,
    args: String
}

struct GameData {
    kit_items: Vec<String>,
}
struct CommandRouter {
    tmux_session_name: String,
    game_cache: GameData,
}

impl CommandRouter {
    fn new(tmux_session_name: String) -> io::Result<Self> {
        let mut kit_items = Vec::new();

        // Open the file items.txt
        let file = File::open("src/items.txt")?;

        // Read the file line by line
        for line in io::BufReader::new(file).lines() {
            let line = line?; // Handle any errors reading the line
            kit_items.push(line); // Store each line in `kit_items`
        }

        Ok(CommandRouter {
            tmux_session_name,
            game_cache: GameData { kit_items },
        })
    }

    fn run_command(&self, message: &Message) {
        match &message.command[1..] {
            "kit" => self.run_kit(message),
            _ => ()
        } 
    }

    fn run_kit(&self, message: &Message) {
        for item in &self.game_cache.kit_items {
            let minecraft_command = &format!("give {} {}", &message.username, item);
            self._execute_in_server(minecraft_command);
        }
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


fn process_line(line: &str) -> Option<Message> {
    let re = Regex::new(r"\[\d{2}:\d{2}:\d{2}\] \[Server thread\/INFO\]: <(.*?)> (\.\w+)(?:\s+(.*))?").unwrap();
    
    if let Some(caps) = re.captures(line) {
        Some(Message { 
            username : caps[1].to_string(),
            command : caps[2].to_string(),
            args : caps.get(3).map_or("", |m| m.as_str()).to_string(),
         })
    } else {
      None
    }
}