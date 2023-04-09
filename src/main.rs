#[macro_use]
extern crate lazy_static;


use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, MutexGuard};
use std::{thread, env};
use std::time::Duration;
use my_lib::*;

lazy_static! {
    static ref TIMER_VALUE: RwLock<u16> = RwLock::new(15);
    static ref  change_val :u32 = 15;
}



#[derive(Serialize, Deserialize)]
struct AskPayload {
    term: u32,
    candidate_id: u32,
}

#[derive(Debug,Serialize, Deserialize)]
struct ExecutePayload {
    command: String,
    args : Vec<i32>
}

async fn append_entries(
    data: web::Json<sendEntries>,
    stateobj: web::Data<Arc<Mutex<state>>>,
    // timer_value: web::Data<&Arc<RwLock<u16>>>,
) -> impl Responder {
    let mut locked_data = match stateobj.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let (term , success) = locked_data.recive_entries(data.term, data.leaderId, data.prevLogIndex, data.prevLogTerm, data.entries.clone(), data.leaderCommit , data.vals);
    // Reset the countdown timer when appendentries route is called
    let mut rng = rand::thread_rng();
    let mut value = TIMER_VALUE.write().unwrap();
    *value = rng.gen_range(15..=30);
    let response_data = askVoteResp {
        term: "some term".to_string(),
        success: success,
    };
    HttpResponse::Ok().json(response_data)
}


async fn request_vote(
    data: web::Json<AskPayload>,
) -> impl Responder {
    let mut rng = rand::thread_rng();
    let mut value = TIMER_VALUE.write().unwrap();
    *value = rng.gen_range(15..=30);
    HttpResponse::Ok().body(format!(
        "Term: {}, Candidate ID: {}",
        data.term, data.candidate_id
    ))
}

async fn execute(
    data: web::Json<ExecutePayload>,
    stateobj: web::Data<Arc<Mutex<state>>>,
) -> impl Responder {
    let mut locked_data = stateobj.lock().unwrap();
    // println!("{:?}" , data);
    let respponse = locked_data.handle_exec(data.command.clone(), data.args.clone());
    println!("{:?}" , locked_data);

    HttpResponse::Ok().body(format!("Command: {}", respponse))
}

async fn requestvotes(
    data: web::Json<askPayload>,
    stateobj: web::Data<Arc<Mutex<state>>>,
) -> impl Responder {
    let mut value = TIMER_VALUE.write().unwrap();
    *value = 15;

    let mut locked_data = stateobj.lock().unwrap();
    let (term , success) = locked_data.grantVote(data.term, data.candidateId, data.lastLogIndex, data.lastLogTerm);
    // println!("check {:?}" , success);
    let response_data = askVoteResp {
        term: "some term".to_string(),
        success: success,
    };
    HttpResponse::Ok().json(response_data)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    // println!("{:?}" , args);
    if args.len() != 2 {
        println!("Usage: {} <id>", args[0]);
        return Ok(()); // <-- return a Result value here
    }
    let id: u32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid id");
            return Ok(()); // <-- return a Result value here
        }
    };
    let stateobj = Arc::new(Mutex::new(state::new(id)));

    // let timer_value: Arc<RwLock<u16>> = Arc::new(RwLock::new(15));
    // let timer_value_clone = timer_value.clone();
    let clone_state = stateobj.clone();
    thread::spawn( move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let mut value = TIMER_VALUE.write().unwrap();
                    let mut locked_data = clone_state.lock().unwrap();
                    *value -= 1;
                    if *value == 0 {
                        if locked_data.voted_for != None && locked_data.voted_for.unwrap() == locked_data.id as u64 {
                            locked_data.call_append().await;
                            println!("call append entries");
                            *value = 5;
                        }
                        else if locked_data.askvotes().await {
                            // println!("Requested for Leader");
                            *value = 5;
                        }
                    }
            println!("Timer value: {}", *value);
        }
    });

    });

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(stateobj.clone()))
            .route("/appendentries", web::post().to(append_entries))
            .route("/requestvote", web::post().to(request_vote))
            .route("/execute", web::post().to(execute))
            .route("/requestvotes", web::post().to(requestvotes))
            .default_service(web::get().to(|| HttpResponse::NotFound()))
    })
    .bind(format!("127.0.0.1:800{}" , id))?
    .run()
    .await
}

