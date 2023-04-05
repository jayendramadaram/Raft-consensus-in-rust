#[macro_use]
extern crate lazy_static;


use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref TIMER_VALUE: RwLock<u16> = RwLock::new(15);
}

#[derive(Serialize, Deserialize)]
struct AppendEntryStruct {
    term: u32,
    leader_id: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct AskPayload {
    term: u32,
    candidate_id: u32,
}

#[derive(Serialize, Deserialize)]
struct ExecutePayload {
    command: String,
    args: Vec<u32>,
}

async fn append_entries(
    data: web::Json<AppendEntryStruct>,
    // timer_value: web::Data<&Arc<RwLock<u16>>>,
) -> impl Responder {
    // Reset the countdown timer when appendentries route is called
    print!("Term: {}, Leader ID: {:?}",
    data.term, data.leader_id);
    let mut value = TIMER_VALUE.write().unwrap();
    *value = 15;
    HttpResponse::Ok().body(format!(
        "Term: {}, Leader ID: {:?}",
        data.term, data.leader_id
    ))
}

async fn request_vote(
    data: web::Json<AskPayload>,
    timer_value: web::Data<&Arc<RwLock<u16>>>,
) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "Term: {}, Candidate ID: {}",
        data.term, data.candidate_id
    ))
}

async fn execute(
    data: web::Json<ExecutePayload>,
    timer_value: web::Data<&Arc<RwLock<u16>>>,
) -> impl Responder {
    HttpResponse::Ok().body(format!("Command: {}, Args: {:?}", data.command, data.args))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let timer_value: Arc<RwLock<u16>> = Arc::new(RwLock::new(15));
    // let timer_value_clone = timer_value.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut value = TIMER_VALUE.write().unwrap();
            *value -= 1;
            if *value == 0 {
                println!("Requested for Leader");
                // Node.askvotes();
                *value = 5;
            }
            println!("Timer value: {}", *value);
        }
    });

    HttpServer::new(move || {
        App::new()
            .route("/appendentries", web::post().to(append_entries))
            .route("/requestvote", web::post().to(request_vote))
            .route("/execute", web::post().to(execute))
            .default_service(web::get().to(|| HttpResponse::NotFound()))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
