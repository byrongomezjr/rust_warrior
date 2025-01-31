use actix_files::NamedFile;
use actix_web::{get, post, web, App, HttpServer, Result, Responder};
use std::sync::{Arc, Mutex};
use serde_json::json;
use crate::GameState;

// Serve the static HTML file
#[get("/")]
async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}

// Get current game state
#[get("/game-state")]
async fn get_game_state(data: web::Data<Arc<Mutex<GameState>>>) -> impl Responder {
    let game_state = data.lock().unwrap();
    web::Json(json!({
        "player": &game_state.player,
        "current_location": &game_state.current_location,
        "game_log": &game_state.game_log,
        "quests": game_state.current_location.as_ref().map(|loc| &loc.quests),
        "code_challenge": game_state.current_location.as_ref()
            .and_then(|loc| loc.quests.first())
            .and_then(|quest| quest.code_challenge.as_ref())
    }))
}

// Update game state
#[post("/game-state")]
async fn update_game_state(
    data: web::Data<Arc<Mutex<GameState>>>,
    input: web::Json<String>
) -> impl Responder {
    let mut game_state = data.lock().unwrap();
    game_state.current_input = input.into_inner();
    game_state.handle_input();
    web::Json(json!({"status": "success"}))
}

// Main server function
pub async fn run_server() -> std::io::Result<()> {
    let game_state = web::Data::new(Arc::new(Mutex::new(GameState::new())));
    
    println!("Starting server at http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(game_state.clone())
            .service(index)
            .service(get_game_state)
            .service(update_game_state)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


