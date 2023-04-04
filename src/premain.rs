use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpListener;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{Read, Write};
use std::net::{ TcpStream};
use std::sync::{Arc, Mutex};
use my_lib::*;
use std::time::{Duration, Instant};

extern crate serde_json;
use serde::{Serialize , Deserialize};
use rand::Rng;
// use serde::Serialize;

// Define a struct to hold the JSON data
#[derive(Serialize, Deserialize)]
struct Member {
    id: i32,
    port: i32,
}

#[derive(Serialize, Deserialize)]
struct Members {
    members: Vec<Member>,
}

fn main() {

    let mut rng = rand::thread_rng();
    // Parse the command-line argument
    let args: Vec<String> = env::args().collect();
    println!("{:?}" , args);
    if args.len() != 2 {
        println!("Usage: {} <id>", args[0]);
        return;
    }
    let id: i32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid id");
            return;
        }
    };

    // Read the JSON file
    let mut file = match File::open("assets/playerConfig.json") {
        Ok(file) => file,
        Err(_) => {
            println!("Error opening members.json");
            return;
        }
    };
    let mut contents = String::new();
    if let Err(_) = file.read_to_string(&mut contents) {
        println!("Error reading members.json");
        return;
    }

    // Parse the JSON data
    let members: Members = match serde_json::from_str(&contents) {
        Ok(data) => data,
        Err(_) => {
            println!("Error parsing members.json");
            return;
        }
    };

    // Find the member with the matching ID
    let mut Node = my_lib::state::new(id.try_into().unwrap());

    let member = members.members.iter().find(|m| m.id == id);
    let port = match member {
        Some(m) => m.port,
        None => {
            println!("Member not found for id={}", id);
            return;
        }
    };

    let timer_value: Arc<Mutex<u16>> = Arc::new(Mutex::new(rng.gen_range(20..=30)));

    // Start the webserver
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    println!("Server started on port {}", port);

    let timer_value_clone = timer_value.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut value = timer_value_clone.lock().unwrap();
            *value -= 1;
            if *value == 0 {
                println!("Requested for Leader");
                Node.askvotes();
                *value == 5;
            }
            println!("Timer value: {}", *value);
        }
    });


    for stream in listener.incoming() {
        let timer_value_clone = timer_value.clone();

        let stream = stream.unwrap();
        thread::spawn(move || {
            handle_connection(stream , &timer_value_clone , Node);
        });
    }
}

// Handle a connection
fn handle_connection(mut stream: std::net::TcpStream , timer_value: &Arc<Mutex<u16>> , Node :my_lib::state) {
    let mut rng = rand::thread_rng();

    // Read the request
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Parse the request
    let request = String::from_utf8_lossy(&buffer[..]);
    let lines: Vec<&str> = request.split("\r\n").collect();
    let tokens: Vec<&str> = lines[0].split(" ").collect();
    let method = tokens[0];
    let path = tokens[1];

    // Dispatch the request
    match (method, path) {
        ("GET", "/foo") => {
            let response = "HTTP/1.1 200 OK\r\n\r\nFoo!";
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        ("GET", "/bar") => {
            let response = "HTTP/1.1 200 OK\r\n\r\nBar!";
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        ("GET", "/time") => {
            let now = SystemTime::now();
            let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
            // let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", since_epoch
            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", since_epoch.as_secs());
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
            
        }
        ("POST", "/askvotes") => {
            // Node.grantVote(term, candidateId, lastLogIndex, lastLogTerm)
        }
        ("GET", "/accept") => {
            let mut value = timer_value.lock().unwrap();
            let newTime = rng.gen_range(20..=30);
            println!("New time alloted {}" , newTime.clone());
            *value = newTime;
            let response = "HTTP/1.1 200 OK\r\n\r\nTimer reset";
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        (_, _) => {
            let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n404 Not Found";
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}







// fn main() {
//     let mut prev_hash: String = "0x0000000000000".to_string();
//     for i in 0..=10 {
//         let currTime = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
//         let mut block1 = my_lib::Block::new(i ,  "0x00000000000".to_owned(), prev_hash.to_owned() , currTime , format!("msg of id {}", i).to_owned());
//         block1.hash = block1.hash();
//         prev_hash = block1.hash.to_string();

//     }
//     // print!("{:?}" , SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis())
// }
