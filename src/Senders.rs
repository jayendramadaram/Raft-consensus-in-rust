use std::io::{Read, ErrorKind}; // IO Read and Error handling 
use serde::{Serialize , Deserialize};
use serde_json::Value;
use serde_json::{json};
use reqwest::{Client, Error, Response, Body};
use tokio::time::{self,timeout, Duration};

// # macro attribute for calling async function recursively 
use async_recursion::async_recursion;

// use serde::Serialize;
// use reqwest::Client;

// l : 6 ✅


#[derive(Clone, Debug ,Serialize, Deserialize)]
pub struct LogEntry {
    term: u32,
    Operation: String,
}


#[derive(Deserialize)]
struct Ip {
    origin: String,
}

// ultimate root state of project SLIM SHADY
#[derive(Debug, Deserialize)]
pub struct state {
    // persistent state
    pub id : u32,
    pub currentterm : u32,
    pub voted_for: Option<u64>,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub next_index : Vec<usize>, // need to update after eveery AE
    pub match_index: Vec<u64>, // need to update after eveery AE
    pub vals : [i32 ; 4]

}
#[derive(Debug, Serialize , Deserialize)]
pub struct sendEntries {
    pub term : u32,
    pub leaderId : Option<u64>,
    pub prevLogIndex : usize,
    pub prevLogTerm : u32,
    pub entries : Vec<LogEntry>,
    pub leaderCommit : u64,
    pub vals : [i32 ; 4]
}



#[derive(Debug, Serialize , Deserialize , Clone)]
pub struct  askPayload {
    pub term : u32,
    pub candidateId : u32,
    pub lastLogIndex : usize,
    pub lastLogTerm : u32
}

#[derive(Debug ,Deserialize , Serialize)]
pub struct askVoteResp {
    pub term : u32,
    pub success : bool
}

impl  state {
    pub fn new(id : u32) -> Self {
        state {
            id,
            currentterm: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: vec![0; 5],
            match_index: vec![0; 5],
            vals : [0 ; 4]
        }
    }
    

    pub fn handle_exec(&mut self , command : String , args : Vec<i32>) -> String {
        let mut resp :String = String::new();
        let result = match command.as_str() {
            "set" => {
                if args.len() >= 2 {
                    let first = args[0] as usize;
                    let second = args[1];
                    if first < 4 {
                        self.vals[first] = second;
                        let logentry = LogEntry {
                            term : self.currentterm,
                            Operation : format!("{} {} {}" ,command.as_str() ,first , second )
                        };
                        self.log.push(logentry);
                        format!("Cool changes made")
                    }else {
                        format!("Invalid index passed")
                    }
                } else {
                    resp = String::from("Error: not enough arguments provided for Set command");
                    resp
                }
            },
            "add" => {
                if args.len() >= 3 {
                    let first = args[0] as usize;
                    let second = args[1];
                    let third = args[2];
                    if first < 4 {
                        self.vals[first] = second + third;
                        let logentry = LogEntry {
                            term : self.currentterm,
                            Operation : format!("{} {} {} {}" ,command.as_str() ,first , second , third )
                        };
                        self.log.push(logentry);
                        
                        format!("Cool changes made")
                    }else {
                        format!("Invalid index passed")
                    }
                } else {
                    resp = String::from("Error: not enough arguments provided for Set command");
                    resp
                }
            },
            "sub" => {
                if args.len() >= 3 {
                    let first = args[0] as usize;
                    let second = args[1];
                    let third = args[2];
                    if first < 4 {
                        self.vals[first] = second - third;
                        let logentry = LogEntry {
                            term : self.currentterm,
                            Operation : format!("{} {} {} {}" ,command.as_str() ,first , second , third )
                        };
                        self.log.push(logentry);
                        
                        format!("Cool changes made")
                    }else {
                        format!("Invalid index passed")
                    }
                } else {
                    resp = String::from("Error: not enough arguments provided for Set command");
                    resp
                }
            },
            "mul" => {
                if args.len() >= 3 {
                    let first = args[0] as usize;
                    let second = args[1];
                    let third = args[2];
                    if first < 4 {
                        self.vals[first] = second * third;
                        let logentry = LogEntry {
                            term : self.currentterm,
                            Operation : format!("{} {} {} {}" ,command.as_str() ,first , second , third )
                        };
                        self.log.push(logentry);
                        
                        format!("Cool changes made")
                    }else {
                        format!("Invalid index passed")
                    }
                } else {
                    resp = String::from("Error: not enough arguments provided for Set command");
                    resp
                }
            },
            "div" => {
                if args.len() >= 3 {
                    let first = args[0] as usize;
                    let second = args[1];
                    let third = args[2];
                    if third == 0 {
                        format!("Zero Division Err Dude")
                    }
                    else if first < 4 {
                        self.vals[first] = second / third;
                        let logentry = LogEntry {
                            term : self.currentterm,
                            Operation : format!("{} {} {} {}" ,command.as_str() ,first , second , third )
                        };
                        self.log.push(logentry);
                        
                        format!("Cool changes made")
                    }else {
                        format!("Invalid index passed")
                    }
                } else {
                    resp = String::from("Error: not enough arguments provided for Set command");
                    resp
                }
            },
            _ => {
                resp = String::from("Wrong command passed");
                resp
            },
        };
        result
    }

    // Regenerate

    #[async_recursion]
    pub async fn send_entries(&mut self , to :usize) -> (u64 , bool) {
        // println!("hereee");
        let prevlogterm = if self.log.len() > 0 {self.log[self.next_index[to]].term} else {0};
        let payload = sendEntries {
            term : self.currentterm ,   // to verify term shouldnt not conflict 
            leaderId :  self.voted_for , // Leader id shouldnt not conflict too
            prevLogIndex :  self.next_index[to] ,  // this must be read and updated after every AE
            prevLogTerm : prevlogterm, // 
            entries : self.log[self.next_index[to]..].to_vec(),
            leaderCommit : self.commit_index,
            vals : self.vals
        };
        let client = reqwest::Client::new();
        let resp_result = client
                    .post(format!("http://127.0.0.1:800{}/appendentries" , to))
                    .json(&payload)
                    .send()
                    .await;
        
                    match resp_result {
                        Ok(resp) => {
                            let body_result = resp.text().await;
                            match body_result {
                                Ok(body) => {
                                    let response_data: askVoteResp = serde_json::from_str(&body).unwrap();
                                    // println!("Response AE {:?}" , response_data);
                                    if !response_data.success {
                                        self.next_index[to] -= 1;
                                        return self.send_entries(to).await;
                                    } else{
                                        self.next_index[to] = if self.log.len() == 0 {0} else {self.log.len() - 1};
                                        return (self.currentterm as u64 , true);
                                    }
                                },
                                Err(e) => {},
                            }
                        }
                        Err(e) => {
                            self.next_index[to] = 0;
                        },
                    }
        
        // println!("Leaders Term {:?} \nLeaders Id {:?} \nprevLogIndex {} \nprevLogTerm {} \nEntries {:?} leaderCommit {}" , &self.currentterm ,  &self.voted_for , self.next_index[to] ,  self.log[self.next_index[to]].term,&self.log[self.next_index[to]..],&self.commit_index);

        // (1 , true)
        // let (current_term, success) = self.recive_entries(self.currentterm, self.voted_for.clone(), self.next_index[to], self.log[self.next_index[to]].term, self.log[self.next_index[to]..].to_vec(), self.commit_index);

        // if !success {
        //     self.next_index[to] -= 1;
        //     return self.send_entries(to);
        // }
    
        (self.currentterm as u64, false)
    }

    pub async fn call_append(&mut self) -> bool {
        
        for player in  0..=4 {
            if player == self.id {
                continue;
            }
            &self.send_entries(player as usize).await;
        }
        true
    }

    pub fn recive_entries(
        &mut self,
        term : u32,
        leaderId : Option<u64>,
        prevLogIndex : usize,
        prevLogTerm : u32,
        entries : Vec<LogEntry>,
        leaderCommit : u64,
        _vals : [i32 ; 4]
    ) -> (u32 , bool) {
        // println!("1");
        let mut success = false;
        let mut current_term = self.currentterm;
        
        if term < current_term {
            // println!("2");
            return (current_term, success);
        }
        let mut entries_to_append: Vec<LogEntry> = vec![];
        if prevLogIndex == 0 {
            entries_to_append.extend_from_slice(&entries);
            // self.log.truncate(prevLogIndex + 1);
            self.log = entries_to_append;
            self.vals = _vals;
            self.commit_index = 0;
            self.currentterm = term;
            println!("{:?} {}", self.log.clone(), "LOGSSS after APPEND");
            return (self.currentterm, true);
        }
       
        
        if self.log.len() < prevLogIndex-1 && self.log[prevLogIndex].term != prevLogTerm {
            return (current_term, success);
        }
        
        if !entries.is_empty() {
            if self.log.len() == 0  {
                return (self.currentterm, false);
            }
            if self.log[prevLogIndex].term != entries[0].term {
                return (self.currentterm, false);
            }

            entries_to_append.extend_from_slice(&entries);
        }

        self.log.truncate(prevLogIndex + 1);
        self.log.append(&mut entries_to_append);
        self.vals = _vals;
        self.commit_index = std::cmp::min(leaderCommit, self.log.len() as u64 - 1);
        self.currentterm = term;
        success = true;
        println!("{:?} {}", self.log.clone(), "LOGSSS after APPEND");
        (current_term, success)

    }

    // asynchronous call multiple servers for votes 
    pub async fn askvotes(&mut self) -> bool {
        let length = self.log.len().saturating_sub(1);
        let payload = askPayload {
            term: self.currentterm,
            candidateId: self.id,
            lastLogIndex: length,
            lastLogTerm: if length > 0 { self.log[length].term } else { 0 },
        };
        let mut totCount = 0;
        let mut total_positve = 0;
        let mut tasks = vec![];
        for player in 0..=4 {
            if player == self.id {
                continue;
            }
            let client = reqwest::Client::new();
            let payload_clone = payload.clone();
            let task = async move {
                let resp_result = timeout(Duration::from_secs(1), client
                    .post(format!("http://127.0.0.1:800{}/requestvotes", player))
                    .json(&payload_clone)
                    .send()
                ).await;
                match resp_result {
                    Ok(resp) => {
                        totCount += 1;
                        let body_result = resp.unwrap().text().await;
                        match body_result {
                            Ok(body) => {
                                let response_data: askVoteResp = serde_json::from_str(&body).unwrap();
                                if response_data.success {
                                    total_positve += 1;
                                }
                            }
                            Err(e) => {}
                        }
                    }
                    Err(e) => {}
                }
            };
            tasks.push(task);
        }
        futures::future::join_all(tasks).await;
        if totCount == 0 || totCount / 2 <= total_positve {
            println!("Declaring myself as leader");
            self.voted_for = Some(self.id as u64);
            return true;
        }
        false
    }
    
    // function to grant vote when a leader asks for a vote
    pub fn grantVote(&mut self,term : u32,
        candidateId : u32,
        lastLogIndex : usize,
        lastLogTerm : u32
    ) -> (u32, bool) {
        println!("Granting Vote");
        let mut success = false;
        let mut current_term = self.currentterm;

        // validate term
        if term < current_term {
            return (current_term, success);
        }

        let last_log = self.log.last();
    
    // validate logs
    if last_log.is_none() {
        success = true;
    } else {
        let last_term = last_log.unwrap().term;

        if lastLogIndex > last_term as usize || (lastLogIndex == last_term as usize && lastLogIndex >= self.log.len() - 1) {
            self.voted_for = Some(candidateId as u64);
            success = true;
        }
    }

    if success {
        self.currentterm = term;
    }

        (current_term, success)
    }
    
}

// pub struct RequestMod {
//     pub RequestVote : String,
// } 

// impl RequestMod {
//     pub fn requestVote(&self) {
//         // for 0..=
//     }
// }