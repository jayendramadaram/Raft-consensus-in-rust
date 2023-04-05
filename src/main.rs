use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    countdown_timer: web::Data<Arc<CountdownTimer>>,
) -> impl Responder {
    // Reset the countdown timer when appendentries route is called
    countdown_timer.reset();
    HttpResponse::Ok().body(format!(
        "Term: {}, Leader ID: {:?}",
        data.term, data.leader_id
    ))
}

async fn request_vote(
    data: web::Json<AskPayload>,
    countdown_timer: web::Data<Arc<CountdownTimer>>,
) -> impl Responder {
    HttpResponse::Ok().body(format!(
        "Term: {}, Candidate ID: {}",
        data.term, data.candidate_id
    ))
}

async fn execute(
    data: web::Json<ExecutePayload>,
    countdown_timer: web::Data<Arc<CountdownTimer>>,
) -> impl Responder {
    HttpResponse::Ok().body(format!("Command: {}, Args: {:?}", data.command, data.args))
}

struct CountdownTimer {
    timer: Option<thread::JoinHandle<()>>,
    stopped: Arc<AtomicBool>,
    countdown: Arc<Mutex<u32>>,
}

impl CountdownTimer {
    fn new() -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let countdown = Arc::new(Mutex::new(15));
        let stopped_clone = stopped.clone();
        let countdown_clone = countdown.clone();

        // Start a new thread for the countdown timer
        let timer = Some(thread::spawn(move || {
            loop {
                let stopped = stopped_clone.load(Ordering::SeqCst);
                if stopped {
                    break;
                }

                let mut countdown = countdown_clone.lock().unwrap();
                println!("{}", *countdown);
                *countdown -= 1;

                // Reset the countdown timer when it reaches 0
                if *countdown == 0 {
                    *countdown = 15;
                }

                thread::sleep(Duration::from_secs(1));
            }
        }));

        CountdownTimer {
            timer,
            stopped,
            countdown,
        }
    }

    fn reset(&self) {
        *self.countdown.lock().unwrap() = 15;
    }

   
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let countdown_timer = web::Data::new(Arc::new(CountdownTimer::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(countdown_timer.clone())
            .route("/appendentries", web::post().to(append_entries))
            .route("/requestvote", web::post().to(request_vote))
            .route("/execute", web::post().to(execute))
            .default_service(web::get().to(|| HttpResponse::NotFound()))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
