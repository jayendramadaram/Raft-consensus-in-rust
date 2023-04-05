use std::io::Read;
use serde::{Serialize , Deserialize};
use serde_json::Value;
use serde_json::{json};
use reqwest::{Client, Error, Response};

#[derive(Clone, Debug)]
pub struct LogEntry {
    term: u32,
    Operation: String,
}

pub struct Vm {
    vals : [u32 ; 4]
}
#[derive(Deserialize)]
struct Ip {
    origin: String,
}
#[derive(Debug, Deserialize)]
 struct askVoteResp {
    data : String
}

pub struct state {
    // persistent state
    pub id : u32,
    pub currentterm : u32,
    pub voted_for: Option<u64>,
    pub log: Vec<LogEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub next_index : Vec<usize>,
    pub match_index: Vec<u64>,

}

#[derive(Debug, Serialize)]
struct  askPayload {
    term : u32,
    candidateId : u32,
    lastLogIndex : usize,
    lastLogTerm : u32
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
        }
    }
    
    
    
    
    // Regenerate

    fn send_entries(&mut self , to :usize) -> (u64 , bool) {
        println!("Leaders Term {:?} \nLeaders Id {:?} \nprevLogIndex {} \nprevLogTerm {} \nEntries {:?} leaderCommit {}" , &self.currentterm ,  &self.voted_for , self.next_index[to] ,  self.log[self.next_index[to]].term,&self.log[self.next_index[to]..],&self.commit_index);
        // (1 , true)
        let (current_term, success) = self.recive_entries(self.currentterm, self.voted_for.clone(), self.next_index[to], self.log[self.next_index[to]].term, self.log[self.next_index[to]..].to_vec(), self.commit_index);

        if !success {
            self.next_index[to] -= 1;
            return self.send_entries(to);
        }
    
        (1 , true)
    }

    fn recive_entries(
        &mut self,
        term : u32,
        leaderId : Option<u64>,
        prevLogIndex : usize,
        prevLogTerm : u32,
        entries : Vec<LogEntry>,
        leaderCommit : u64
    ) -> (u32 , bool) {
        let mut success = false;
        let mut current_term = self.currentterm;

        if term < current_term {
            return (current_term, success);
        }
       
        
        if self.log.len() < prevLogIndex-1 && self.log[prevLogIndex].term != prevLogTerm {
            return (current_term, success);
        }
        
        let mut entries_to_append: Vec<LogEntry> = vec![];
        if !entries.is_empty() {
            if self.log[prevLogIndex].term != entries[0].term {
                return (self.currentterm, false);
            }

            entries_to_append.extend_from_slice(&entries);
        }

        self.log.truncate(prevLogIndex + 1);
        self.log.append(&mut entries_to_append);
        self.commit_index = std::cmp::min(leaderCommit, self.log.len() as u64 - 1);
        self.currentterm = term;
        success = true;
        (current_term, success)

    }

    pub async fn askvotes(&mut self) -> (u32, bool) {
        let client = Client::new();

        for i in 0..=4 {
            let url = format!("http://localhost:800{}/askvotes" , i);
            
            let payload = askPayload {
                term : self.currentterm,
                candidateId : self.id,
                lastLogIndex : self.log.len() - 1,
                lastLogTerm : self.log[&self.log.len() - 1].term,
            };
            let payload_json = serde_json::to_string(&payload).unwrap();
            let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(payload_json)
            .send().await.unwrap();
            print!("{:?}" , response)
            // let out = response.json::<Ip>().await.unwrap();
    
    
            
                   
        }
        
        for i in 1..=4  {
            println!(
                "Current term {:?} \nmy Id {:?}  \n My last Log {:?}  , \nand my last term {:?} Requesting to {}",
                &self.currentterm,
                &self.id,
                &self.log.len() - 1,
                &self.log[&self.log.len() - 1].term,
                i
            )}
        self.voted_for = Some(self.id as u64);

        (1, true)
    }

    pub fn grantVote(&mut self,term : u32,
        candidateId : u32,
        lastLogIndex : usize,
        lastLogTerm : u32
    ) -> (u32, bool) {
        let mut success = false;
        let mut current_term = self.currentterm;

        if term < current_term {
            return (current_term, success);
        }

        let last_log = self.log.last();

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