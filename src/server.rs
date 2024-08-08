use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, Result};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// define game state structure

#[derive(Debug, Serialize, Deserialize)]
struct GameState {
    player_name: String,
    current_location: String,

    // TODO: add more game state fields as needed !! - PENDING
}

// handler to serve index.html file

async fn index() -> Result<NamedFile> {
    let path: PathBuf = "static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

// handler to get current game state

async fn get_game_state(data: web::Data<Arc<Mutex<GameState>>>) -> impl Responder {
    let game_state = data.lock().unwrap();
    HttpResponse::Ok().json(&*game_state)
}

// handler to update game state (example: move player)

async fn update_game_state(
    data: web::Data<Arc<Mutex<GameState>>>,
    new_state: web::Json<GameState>,
) -> impl Responder {
    let mut game_state = data.lock().unwrap();
    *game_state = new_state.into_inner();
    HttpResponse::Ok().json(&*game_state)
}

pub async fn run_server() -> std::io::Result<()> {

    // initialize shared game state
    
    let game_state = Arc::new(Mutex::new(GameState {
        player_name: "Adventurer".to_string(),
        current_location: "Home".to_string(),
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(game_state.clone()))
            .route("/", web::get().to(index))
            .route("/game", web::get().to(get_game_state))
            .route("/game", web::post().to(update_game_state))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


