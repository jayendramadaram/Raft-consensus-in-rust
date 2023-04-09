// to maintain a global timer object
#[macro_use]
extern crate lazy_static;

// actix server
use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use rand::Rng; // to generate random numbers
use serde::{Deserialize, Serialize}; // to serialize structs to json and deser
// use std::sync::atomic::{AtomicBool, Ordering}; // to safely do operations and lock on threaded vars
use std::sync::{Arc, Mutex, RwLock}; // Atomically Reference Counted , lock access , multiple read writes
use std::{thread, env}; // threading and cature port number
use std::time::Duration; // time ig
use my_lib::*; // Library file

// Timer to be a global 
lazy_static! {
    static ref TIMER_VALUE: RwLock<u16> = RwLock::new(15);
    // static ref  change_val :u32 = 15;
}


// struct used while asking votes
#[derive(Serialize, Deserialize)]
struct AskPayload {
    term: u32,
    candidate_id: u32,
}

// struct which incoming execute route data payload has
#[derive(Debug,Serialize, Deserialize)]
struct ExecutePayload {
    command: String,
    args : Vec<i32>
}

// append Entries Function which resets Timeout time for individual nodes and also recives logs from node
async fn append_entries(
    data: web::Json<sendEntries>,
    stateobj: web::Data<Arc<Mutex<state>>>,
    // timer_value: web::Data<&Arc<RwLock<u16>>>,
) -> impl Responder {

    // unlock state struct  
    let mut locked_data = match stateobj.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    // call recive_entries impl 
    let (term , success) = locked_data.recive_entries(data.term, data.leaderId, data.prevLogIndex, data.prevLogTerm, data.entries.clone(), data.leaderCommit , data.vals);

    // Reset the countdown timer when appendentries route is called
    let mut rng = rand::thread_rng(); // rand timeout generate
    let mut value = TIMER_VALUE.write().unwrap();
    *value = rng.gen_range(15..=30);

    // send response
    let response_data = askVoteResp {
        term: locked_data.currentterm,
        success: success,
    };
    HttpResponse::Ok().json(response_data)
}



// execute route which adds new entry to logs obj and makes operation on VM
async fn execute(
    data: web::Json<ExecutePayload>,
    stateobj: web::Data<Arc<Mutex<state>>>,
) -> impl Responder {

    // unlock state
    let mut locked_data = stateobj.lock().unwrap();
    
    // call handle execute and return data 
    let respponse = locked_data.handle_exec(data.command.clone(), data.args.clone());
    // println!("{:?}" , locked_data);
    HttpResponse::Ok().body(format!("Command: {}", respponse))
}

// grant votes when a leaders asks for votes
async fn requestvotes(
    data: web::Json<askPayload>,
    stateobj: web::Data<Arc<Mutex<state>>>,
) -> impl Responder {

    // unlock and reset the timer
    let mut value = TIMER_VALUE.write().unwrap();
    *value = 15;

    // unlock state object
    let mut locked_data = stateobj.lock().unwrap();
    
    // call grant vote
    let (term , success) = locked_data.grantVote(data.term, data.candidateId, data.lastLogIndex, data.lastLogTerm);
    
    // respond
    let response_data = askVoteResp {
        term:locked_data.currentterm,
        success: success,
    };
    HttpResponse::Ok().json(response_data)
}


//actix server init
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // collect env arguments passed through commandline
    let args: Vec<String> = env::args().collect();
    
    // error handling
    if args.len() != 2 {
        println!("Usage: {} <id>", args[0]);
        return Ok(()); 
    }

    // error handling
    let id: u32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Invalid id");
            return Ok(()); // <-- return a Result value here
        }
    };

    // create a new state OBJECT
    let stateobj = Arc::new(Mutex::new(state::new(id)));

    // let timer_value: Arc<RwLock<u16>> = Arc::new(RwLock::new(15));
    // let timer_value_clone = timer_value.clone();

    // clone the object and pass it to seperate timer thread to act upon
    let clone_state = stateobj.clone();

    // thread::spawn --> create a new thread
    // move --> transfer clone_state ownership to different thread 

    thread::spawn( move || {

        // tokio runtime used for parallel tasks [meatubale rt because block_on needs muateble reference]
        let mut rt = tokio::runtime::Runtime::new().unwrap();

            // block working of this thread and run this async function
            rt.block_on(async move {
                loop {
                    // sleep for a sec and countdown
                    thread::sleep(Duration::from_secs(1));

                    // unlock timer to decrement it and locked-data as well 
                    let mut value = TIMER_VALUE.write().unwrap();
                    let mut locked_data = clone_state.lock().unwrap();
                    // unlock state to call operation on it
                    *value -= 1;
                    if *value == 0 {

                        
                        // if i am already a leader i call append entries
                        if locked_data.voted_for != None && locked_data.voted_for.unwrap() == locked_data.id as u64 {
                            locked_data.call_append().await;
                            println!("call append entries");
                            *value = 5;
                        }
                        // else i am not a leader and my timeout happened i will become leader
                        else if locked_data.askvotes().await {
                            // println!("Requested for Leader");
                            *value = 5;
                        }
                    }
            println!("Timer value: {}", *value);
        }
    });

    });

    // start a httpServer with actix in main Thread
    // cargo run 1 starts server at http://localhost:800{1}
    HttpServer::new(move || {
        App::new()
        // pass stateObj clone to Main thread 
        .app_data(web::Data::new(stateobj.clone()))
            .route("/appendentries", web::post().to(append_entries))
            .route("/execute", web::post().to(execute))
            .route("/requestvotes", web::post().to(requestvotes))
            .default_service(web::get().to(|| HttpResponse::NotFound()))
    })
    .bind(format!("127.0.0.1:800{}" , id))?
    .run()
    .await
}

