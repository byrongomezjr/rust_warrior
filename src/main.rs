use eframe::egui;
use egui::ViewportBuilder;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

mod server;

// define game state structure

struct GameState {
    game_log: Vec<String>,
    current_input: String,
    current_location: Option<Location>,
    player: Option<Player>,
    player_health: i32,
}

impl GameState {
    fn new() -> Self {
        let mut starting_location = Location::new("Home", "You are in your cozy home.");
        starting_location.exits.insert("north".to_string(), "Forest".to_string());
        
        GameState {
            game_log: Vec::new(),
            current_input: String::new(),
            current_location: Some(starting_location),
            player: Some(Player::new("Adventurer", "Home")),
            player_health: 100,
        }
    }

    fn handle_input(&mut self) {
        let input = self.current_input.clone();
        self.game_log.push(format!("> {}", input));
        
        match input.trim().to_lowercase().as_str() {
            "quit" | "exit" => {
                self.game_log.push("Exiting the game. Thank you for playing!".to_string());
                std::process::exit(0);
            }
            "help" => {
                self.game_log.push("\nAvailable commands:".to_string());
                self.game_log.push("- help: Show this help message".to_string());
                self.game_log.push("- look: Look around your current location".to_string());
                self.game_log.push("- inventory: Check your inventory".to_string());
                self.game_log.push("- go <direction>: Move in a direction (north, south, etc.)".to_string());
                self.game_log.push("- save: Save your game".to_string());
                self.game_log.push("- load: Load your game".to_string());
                self.game_log.push("- quit/exit: Exit the game\n".to_string());
            }
            "look" => {
                if let Some(location) = &self.current_location {
                    self.game_log.push(format!("\nLocation: {}", location.name));
                    self.game_log.push(format!("{}\n", location.description));
                    
                    // display available exits

                    if !location.exits.is_empty() {
                        let exits: Vec<String> = location.exits.keys()
                            .map(|s| s.to_string())
                            .collect();
                        self.game_log.push(format!("Exits: {}\n", exits.join(", ")));
                    }
                } else {
                    self.game_log.push("\nYou are at Home.\n".to_string());
                }
            }
            "inventory" => {
                if let Some(player) = &self.player {
                    if player.inventory.is_empty() {
                        self.game_log.push("\nYour inventory is empty.\n".to_string());
                    } else {
                        self.game_log.push("\nInventory:".to_string());
                        for item in &player.inventory {
                            self.game_log.push(format!("- {}", item));
                        }
                        self.game_log.push("".to_string());
                    }
                } else {
                    self.game_log.push("\nYour inventory is empty.\n".to_string());
                }
            }
            cmd if cmd.starts_with("go ") => {
                let direction = cmd.trim_start_matches("go ").trim();
                if let Some(location) = &self.current_location {
                    if let Some(next_location) = location.exits.get(direction) {
                        self.current_location = Some(Location::new(
                            next_location,
                            "You have entered a new area."
                        ));
                        self.game_log.push(format!("\nYou move {}.\n", direction));
                    } else {
                        self.game_log.push("\nYou can't go that way!\n".to_string());
                    }
                }
            }
            "save" => {
                self.game_log.push("\nGame saved.\n".to_string());
                // implement save functionality
            }
            "load" => {
                self.game_log.push("\nGame loaded.\n".to_string());
                // implement load functionality
            }
            _ => self.game_log.push("\nUnknown command. Type 'help' for available commands.\n".to_string()),
        }
        
        self.current_input.clear();
    }
}

// implement the eframe::App trait for GameState
impl eframe::App for GameState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // game log area
            egui::ScrollArea::vertical().show(ui, |ui| {
                for message in &self.game_log {
                    ui.label(message);
                }
            });

            // input area
            ui.horizontal(|ui| {
                let text_edit = ui.text_edit_singleline(&mut self.current_input);
                if (ui.button("Submit").clicked() || text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) 
                    && !self.current_input.is_empty() 
                {
                    self.handle_input();
                }
            });

            // status area
            ui.separator();
            ui.label(format!("Health: {}", self.player_health));
            if let Some(location) = &self.current_location {
                ui.label(format!("Location: {}", location.name));
            }
        });
    }
}

// modify your main function

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    // launch the GUI application ** Important !!

    eframe::run_native(
        "Rust Warrior",
        options,
        Box::new(|_cc| Ok(Box::new(GameState::new()))),
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct Location {
    name: String,
    description: String,
    quests: Vec<Quest>,
    exits: HashMap<String, String>,
}

impl Location {
    fn new(name: &str, description: &str) -> Self {
        Location {
            name: name.to_string(),
            description: description.to_string(),
            quests: Vec::new(),
            exits: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Quest {
    description: String,
    completed: bool,
    rust_concept: String,
}

impl Quest {
    fn new(description: &str, rust_concept: &str) -> Self {
        Quest {
            description: description.to_string(),
            completed: false,
            rust_concept: rust_concept.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    name: String,
    current_location: String,
    inventory: Vec<String>,
    completed_quests: Vec<String>,
}

impl Player {
    fn new(name: &str, start_location: &str) -> Self {
        Player {
            name: name.to_string(),
            current_location: start_location.to_string(),
            inventory: Vec::new(),
            completed_quests: Vec::new(),
        }
    }

    fn save(&self, filename: &str) -> io::Result<()> {
        let serialized = serde_json::to_string(self)?;
        fs::write(filename, serialized)?;
        Ok(())
    }

    fn load(filename: &str) -> io::Result<Self> {
        let data = fs::read_to_string(filename)?;
        let player = serde_json::from_str(&data)?;
        Ok(player)
    }
}

fn game_loop() {
    let locations = setup_locations();
    let mut player = setup_player();

    // start combat with a new enemy

    let mut enemy = Enemy::new("Goblin", 30, 10);
    combat(&mut player, &mut enemy);

    loop {
        // display current location and available actions

        display_location(&player, &locations);

        // player input

        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim().to_lowercase();

        // process commands

        match input.as_str() {
            "quit" | "exit" => {
                println!("Exiting the game. Thank you for playing!");
                break; // exit loop to end the game !!
            }
            "save" => {
                if let Err(err) = player.save("save_game.json") {
                    println!("Failed to save game: {}", err);
                } else {
                    println!("Game saved.");
                }
            }
            "load" => {
                match Player::load("save_game.json") {
                    Ok(saved_player) => {
                        player = saved_player;
                        println!("Game loaded.");
                    }
                    Err(err) => println!("Failed to load game: {}", err),
                }
            }
            "inventory" => display_inventory(&player),
            _ if input.starts_with("go ") => {
                let direction = input.split_whitespace().nth(1).unwrap_or("");
                if let Some(next_location) = locations[&player.current_location].exits.get(direction) {
                    player.current_location = next_location.clone();
                } else {
                    println!("You can't go that way!");
                }
            }
            _ => println!("I don't understand that command."),
        }
    }

    println!("Game session ended.");
}

fn setup_locations() -> HashMap<String, Location> {
    let mut locations = HashMap::new();

    let mut home = Location::new("Home", "You are in your cozy home.");
    home.exits.insert("north".to_string(), "Forest".to_string());

    let mut forest = Location::new("Forest", "You are in a dark, spooky forest.");
    forest.quests.push(Quest::new("Find a stick to defend yourself.", "variables"));
    forest.exits.insert("south".to_string(), "Home".to_string());

    locations.insert("Home".to_string(), home);
    locations.insert("Forest".to_string(), forest);

    locations
}

fn setup_player() -> Player {
    
    // prompts player for their name if desired

    Player::new("Adventurer", "Home")
}

fn display_location(player: &Player, locations: &HashMap<String, Location>) {
    let location = locations.get(&player.current_location).unwrap();
    println!("Location: {}", location.name);
    println!("{}", location.description);

    // displays quests

    for quest in &location.quests {
        if !player.completed_quests.contains(&quest.description) {
            println!("Quest: {}", quest.description);
        }
    }

    // displays exits

    if !location.exits.is_empty() {
        print!("Exits: ");
        for exit in location.exits.keys() {
            print!("{} ", exit);
        }
        println!();
    }
}

fn display_inventory(player: &Player) {
    if player.inventory.is_empty() {
        println!("Your inventory is empty.");
    } else {
        print!("Inventory: ");
        for item in &player.inventory {
            print!("{} ", item);
        }
        println!();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Enemy {
    name: String,
    health: i32,
    damage: i32,
}

impl Enemy {
    fn new(name: &str, health: i32, damage: i32) -> Self {
        Enemy {
            name: name.to_string(),
            health,
            damage,
        }
    }
}

fn combat(_player: &mut Player, enemy: &mut Enemy) {
    println!("You encounter a {}!", enemy.name);

    loop {
        println!("Enemy health: {}", enemy.health);
        println!("Player health: 100"); // TODO: add player health logic !!

        // combat logic

        println!("Do you want to attack or run?");
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "attack" => {
                println!("You attack the {}!", enemy.name);
                enemy.health -= 10; // TODO: adjust combat damage !!
                if enemy.health <= 0 {
                    println!("You defeated the {}!", enemy.name);
                    return;
                }
            }
            "run" => {
                println!("You run away from the {}.", enemy.name);
                return;
            }
            _ => println!("I don't understand that command."),
        }

        // enemy attacks

        println!("The {} attacks you!", enemy.name);

        // TODO: add player health logic here also !!
    }
}