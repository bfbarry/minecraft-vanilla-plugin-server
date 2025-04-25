pub mod nbt;
pub mod game;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};
use regex::Regex;

fn main() -> io::Result<()> {
    // TODO: create constant root path to use across source
    let log_path = "/Users/brianbarry/Desktop/computing/minecraft-vanilla-plugin-server/logs/dummy_log.log";
    let file = File::open(log_path).expect("log file not found");
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
    let command_router = game::CommandRouter::new("mcserver".to_string()).unwrap(); //TODO unsafe
    let command_router = Arc::new(command_router);
    let mut worker_handles = Vec::new();

    for _ in 0..5 {
        let receiver = Arc::clone(&receiver);
        let command_router = Arc::clone(&command_router);
        let handle = thread::spawn(move || loop {
            let line = {
                let receiver = receiver.lock().unwrap();
                match receiver.recv() {
                    Ok(line) => line,
                    Err(_) => return, // TODO: more informative handling?
                }
            };

            //     println!("processing line: {}", line);
            if let Some(data) = process_line(&line) {
                command_router.run_command(&data);
            }
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

fn process_line(line: &str) -> Option<game::Message> {
    let re = Regex::new(r"\[\d{2}:\d{2}:\d{2}\] \[Server thread\/INFO\]: <(.*?)> (\.\w+)(?:\s+(.*))?").unwrap();
    
    if let Some(caps) = re.captures(line) {
        Some(game::Message { 
            username : caps[1].to_string(),
            command : caps[2].to_string(),
            args : caps.get(3).map_or("", |m| m.as_str()).to_string(),
         })
    } else {
      None
    }
}