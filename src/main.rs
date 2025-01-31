use eframe::egui;
use egui::ViewportBuilder;
use std::collections::HashMap;
use std::fs;
use std::io;
use serde::{Deserialize, Serialize};

mod server;

// define game state structure

struct GameState {
    game_log: Vec<String>,
    current_input: String,
    current_location: Option<Location>,
    player: Option<Player>,
    player_health: i32,
    locations: HashMap<String, Location>,
    current_enemy: Option<Enemy>,
    current_battle: Option<BattleState>,
}

impl GameState {
    fn new() -> Self {
        let mut locations = HashMap::new();
        
        // Create Home (starting location)
        let mut home = Location::new(
            "Home",
            "Your cozy starting point. A path leads forward to the training grounds."
        );
        home.exits.insert("forward".to_string(), "Training Grounds".to_string());
        locations.insert("Home".to_string(), home);

        // Create Training Grounds
        let mut training = Location::new(
            "Training Grounds",
            "A spacious area with training dummies. Paths lead backward to home and right to Level 1."
        );
        training.exits.insert("backward".to_string(), "Home".to_string());
        training.exits.insert("right".to_string(), "Level 1".to_string());
        
        let training_challenge = CodeChallenge::new(
            "Training: Basic Combat\n\n\
            Learn the basics of combat by defeating the training dummy.\n\
            Use 'attack' command to deal damage and 'rest' to heal.",
            "// Training ground code here"
        );
        training.quests.push(Quest::with_challenge(
            "Basic Combat Training",
            "Learn basic combat mechanics",
            training_challenge
        ));
        locations.insert("Training Grounds".to_string(), training);

        // Create Level 1
        let mut level1 = Location::new(
            "Level 1",
            "A long hallway with stairs at the end. There's nothing in the way."
        );
        level1.exits.insert("backward".to_string(), "Training Grounds".to_string());
        
        let level1_challenge = CodeChallenge::new(
            "Level 1: Baby Steps\n\n\
            ╔════════╗\n\
            ║@      >║\n\
            ╚════════╝\n\n\
            You see before yourself a long hallway with stairs at the end. There's nothing in the way.\n\n\
            TIP: Call warrior.walk() to walk forward in the Player's playTurn method.",
            
            "fn play_turn(warrior: &mut Warrior) {\n    // Cool code goes here\n    \n}"
        );
        level1.quests.push(Quest::with_challenge(
            "First Steps",
            "Learn to navigate using basic movement",
            level1_challenge
        ));
        locations.insert("Level 1".to_string(), level1);

        GameState {
            game_log: vec![
                "Welcome to Rust Warrior!".to_string(),
                "Type 'go forward' to reach the Training Grounds and begin your first challenge.".to_string(),
                "\nType 'help' for available commands.".to_string(),
            ],
            current_input: String::new(),
            current_location: locations.get("Home").cloned(),
            player: Some(Player::new("Warrior", "Home")),
            player_health: 100,
            locations,
            current_enemy: None,
            current_battle: None,
        }
    }

    fn complete_current_quest(&mut self) {
        if let Some(location) = &mut self.current_location {
            for quest in &mut location.quests {
                if !quest.completed {
                    quest.completed = true;
                    self.game_log.push(format!("\n✓ Completed: {}", quest.name));
                    self.game_log.push("You can now proceed to the next area.".to_string());
                    break;
                }
            }
        }
    }

    fn handle_input(&mut self) {
        let input = self.current_input.clone();
        self.game_log.push(format!("> {}", input));
        
        match input.trim().to_lowercase().as_str() {
            "help" => {
                self.game_log.push("\nAvailable Commands:".to_string());
                self.game_log.push("Movement:".to_string());
                self.game_log.push("- go forward  : Move forward".to_string());
                self.game_log.push("- go backward : Move backward".to_string());
                self.game_log.push("- go left     : Move to the left".to_string());
                self.game_log.push("- go right    : Move to the right".to_string());
                self.game_log.push("\nOther Commands:".to_string());
                self.game_log.push("- look      : Examine your surroundings".to_string());
                self.game_log.push("- inventory : Check your items".to_string());
                self.game_log.push("- help      : Show this help message".to_string());
            }
            "look" => {
                if let Some(location) = &self.current_location {
                    self.game_log.push(format!("\n📍 Location: {}", location.name));
                    self.game_log.push(format!("{}\n", location.description));
                    
                    if !location.exits.is_empty() {
                        let mut exits = String::from("Exits: ");
                        for (direction, _) in &location.exits {
                            exits.push_str(&format!("{}, ", direction));
                        }
                        exits.truncate(exits.len() - 2);
                        self.game_log.push(format!("{}\n", exits));
                    }

                    if !location.quests.is_empty() {
                        self.game_log.push("Available Quests:".to_string());
                        for quest in &location.quests {
                            let status = if quest.completed { "✓" } else { "◆" };
                            self.game_log.push(format!("{} {}", status, quest.description));
                        }
                        self.game_log.push("".to_string());
                    }
                }
            }
            "inventory" => {
                if let Some(player) = &self.player {
                    if player.inventory.is_empty() {
                        self.game_log.push("Your inventory is empty.".to_string());
                    } else {
                        self.game_log.push("Inventory:".to_string());
                        for item in &player.inventory {
                            self.game_log.push(format!("- {}", item));
                        }
                    }
                }
            }
            cmd if cmd.starts_with("go ") => {
                let direction = cmd.trim_start_matches("go ").trim();
                
                // Check if current quest is completed before allowing movement
                if let Some(location) = &self.current_location {
                    let has_incomplete_quest = location.quests.iter().any(|q| !q.completed);
                    if has_incomplete_quest && direction == "right" {
                        self.game_log.push("Complete the current challenge before proceeding.".to_string());
                        self.current_input.clear();
                        return;
                    }
                }

                // Get the next location name
                let next_location_name = self.current_location.as_ref()
                    .and_then(|loc| loc.exits.get(direction))
                    .cloned();
                
                if let Some(next_name) = next_location_name {
                    // Get the new location data
                    if let Some(new_location) = self.locations.get(&next_name).cloned() {
                        // Update player location
                        if let Some(player) = &mut self.player {
                            player.current_location = next_name.clone();
                        }
                        
                        // Update current location
                        self.current_location = Some(new_location);
                        
                        self.game_log.push(format!("\nYou move {} to {}.", direction, next_name));
                        if let Some(loc) = &self.current_location {
                            self.game_log.push(format!("{}\n", loc.description));
                        }
                    }
                } else {
                    self.game_log.push("\nYou can't go that way!\n".to_string());
                }
            }
            "save" => {
                if let Some(player) = &self.player {
                    match player.save("savegame.json") {
                        Ok(_) => self.game_log.push("Game saved successfully.".to_string()),
                        Err(e) => self.game_log.push(format!("Error saving game: {}", e)),
                    }
                }
            }
            "load" => {
                match Player::load("savegame.json") {
                    Ok(loaded_player) => {
                        self.player = Some(loaded_player);
                        self.game_log.push("Game loaded successfully.".to_string());
                        if let Some(player) = &self.player {
                            self.current_location = self.locations.get(&player.current_location).cloned();
                        }
                    }
                    Err(e) => self.game_log.push(format!("Error loading game: {}", e)),
                }
            }
            "quit" | "exit" => {
                self.game_log.push("Thanks for playing! Closing game...".to_string());
                std::process::exit(0);
            }
            "fight" => {
                if self.current_enemy.is_none() {
                    // Create a new enemy when fight is initiated
                    self.current_enemy = Some(Enemy::new("Training Dummy", 30, 5));
                    self.game_log.push("\nA Training Dummy appears!".to_string());
                    self.game_log.push("Commands: 'attack' to fight, 'run' to flee".to_string());
                }
                self.game_log.push("You are in combat!".to_string());
            }
            "attack" => {
                // Handle attack command and check for victory
                if let Some(enemy) = &mut self.current_enemy {
                    enemy.health -= 10;
                    self.game_log.push("You attack the Training Dummy!".to_string());
                    self.game_log.push(format!("Enemy health: {}", enemy.health));
                    
                    if enemy.health <= 0 {
                        self.game_log.push("You defeated the Training Dummy!".to_string());
                        self.complete_current_quest();
                        self.current_enemy = None;
                    } else {
                        self.player_health -= 5;
                        self.game_log.push("The Training Dummy attacks you for 5 damage!".to_string());
                        self.game_log.push(format!("Your health: {}", self.player_health));
                    }
                }
            }
            "run" => {
                if self.current_enemy.is_some() {
                    self.game_log.push("You run away from the fight!".to_string());
                    self.current_enemy = None;
                } else {
                    self.game_log.push("There is nothing to run from!".to_string());
                }
            }
            _ => self.game_log.push("Unknown command. Type 'help' for available commands.".to_string()),
        }
        
        self.current_input.clear();
    }
}

// Implement the eframe::App trait for GameState
impl eframe::App for GameState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set default text styles with better sizes
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(16.0, egui::FontFamily::Monospace)),
        ].into();
        ctx.set_style(style);

        // Left Panel: Stats and Inventory
        egui::SidePanel::left("info_panel")
            .resizable(true)
            .default_width(200.0)
            .width_range(180.0..=250.0)
            .show(ctx, |ui| {
                ui.heading("Warrior Stats");
                ui.add_space(4.0);
                
                ui.group(|ui| {
                    ui.label(format!("Health: {}/100", self.player_health));
                    if let Some(player) = &self.player {
                        ui.label(format!("Level: {}", player.completed_quests.len()));
                        ui.label(format!("Location: {}", player.current_location));
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);
                
                ui.heading("Inventory");
                ui.group(|ui| {
                    if let Some(player) = &self.player {
                        if player.inventory.is_empty() {
                            ui.label("Empty");
                        } else {
                            for item in &player.inventory {
                                ui.label(format!("• {}", item));
                            }
                        }
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.heading("Quests");
                ui.group(|ui| {
                    if let Some(location) = &self.current_location {
                        for quest in &location.quests {
                            let status = if quest.completed { "✓" } else { "◆" };
                            ui.horizontal(|ui| {
                                ui.label(status);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!("⚡ {}", quest.description))
                                            .strong()
                                    );
                                    ui.small(format!("🦀 Rust: {}", quest.rust_concept));
                                    if quest.completed {
                                        ui.small("✨ Completed!");
                                    } else {
                                        ui.small("⏳ In Progress");
                                    }
                                });
                            });
                            ui.add_space(4.0);
                        }
                    } else {
                        ui.label("📋 No quests available");
                    }
                });
            });

        // Main Panel: Game Log and Input
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title and subtitle
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("Rust Warrior");
                ui.label("Learn Rust through adventure!");
                ui.add_space(10.0);
            });

            // Commands section
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Available Commands:");
                    ui.add_space(5.0);
                    ui.horizontal_wrapped(|ui| {
                        let commands = [
                            "help", "look", "inventory", 
                            "go <direction>", "fight", 
                            "save", "load", "quit"
                        ];
                        for cmd in commands {
                            ui.label(
                                egui::RichText::new(cmd)
                                    .monospace()
                                    .background_color(egui::Color32::from_rgb(45, 45, 45))
                            );
                            ui.add_space(8.0);
                        }
                    });
                });
            });
            ui.add_space(10.0);

            // Game log
            ui.group(|ui| {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for message in &self.game_log {
                            if message.starts_with('>') {
                                ui.colored_label(egui::Color32::LIGHT_BLUE, message);
                            } else if message.contains("Error") || message.contains("can't") {
                                ui.colored_label(egui::Color32::LIGHT_RED, message);
                            } else if message.contains("Welcome to Rust Warrior!") {
                                ui.label(
                                    egui::RichText::new(message)
                                        .size(20.0)
                                        .color(egui::Color32::LIGHT_YELLOW)
                                );
                            } else {
                                ui.label(message);
                            }
                            ui.add_space(2.0);
                        }
                    });
            });

            // Input area
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let text_edit = ui.add_sized(
                        [ui.available_width() - 60.0, 32.0],
                        egui::TextEdit::singleline(&mut self.current_input)
                            .hint_text("Enter command...")
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    if (ui.button("Enter").clicked() || 
                        text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) 
                        && !self.current_input.is_empty() 
                    {
                        self.handle_input();
                    }
                });
            });
        });

        // Right Panel: Map
        egui::SidePanel::right("map_panel")
            .resizable(true)
            .default_width(300.0)
            .width_range(250.0..=400.0)
            .show(ctx, |ui| {
                ui.heading("World Map");
                
                ui.group(|ui| {
                    if let Some(location) = &self.current_location {
                        ui.label("Available Exits:");
                        ui.add_space(4.0);
                        for (direction, destination) in &location.exits {
                            ui.label(format!("→ {} to {}", direction, destination));
                        }
                    }
                });

                // Wrap the entire Code Challenge section in a ScrollArea
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 100.0)
                    .show(ui, |ui| {
                        // Code Challenge section
                        if let Some(location) = &self.current_location {
                            for quest in &location.quests {
                                if let Some(challenge) = &quest.code_challenge {
                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(8.0);
                                    
                                    ui.heading("Code Challenge");
                                    ui.add_space(4.0);
                                    
                                    // Battle Visualization
                                    if let Some(battle_state) = &self.current_battle {
                                        ui.group(|ui| {
                                            ui.label("Battle Visualization:");
                                            ui.add_space(4.0);
                                            
                                            // Show health
                                            ui.label(format!("♥ {}", battle_state.warrior_health));
                                            ui.add_space(4.0);
                                            
                                            // Show map
                                            ui.label(
                                                egui::RichText::new(&battle_state.render())
                                                    .font(egui::FontId::new(16.0, egui::FontFamily::Monospace))
                                            );
                                            
                                            // Show battle log
                                            ui.group(|ui| {
                                                egui::ScrollArea::vertical()
                                                    .max_height(150.0)
                                                    .show(ui, |ui| {
                                                        for log in &battle_state.battle_log {
                                                            ui.label(log);
                                                        }
                                                    });
                                            });
                                        });
                                    }

                                    // Challenge description in a group
                                    ui.group(|ui| {
                                        ui.label(
                                            egui::RichText::new(&challenge.description)
                                                .font(egui::FontId::new(14.0, egui::FontFamily::Monospace))
                                            .background_color(egui::Color32::from_rgb(45, 45, 45))
                                        );
                                    });
                                    
                                    ui.add_space(8.0);
                                    
                                    // Available methods section
                                    ui.group(|ui| {
                                        ui.label("Available Methods:");
                                        ui.add_space(4.0);
                                        
                                        let methods = [
                                            ("warrior.attack()", "Attacks forward, dealing 5 damage"),
                                            ("warrior.rest()", "Recovers 10% of max health"),
                                            ("warrior.walk()", "Moves forward one space"),
                                            ("warrior.health()", "Returns current health"),
                                            ("warrior.max_health()", "Returns maximum health"),
                                        ];
                                        
                                        for (method, description) in methods {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(method)
                                                        .font(egui::FontId::new(14.0, egui::FontFamily::Monospace))
                                                        .background_color(egui::Color32::from_rgb(45, 45, 45))
                                                );
                                            });
                                            ui.label(description);
                                            ui.add_space(2.0);
                                        }
                                    });
                                    
                                    ui.add_space(8.0);
                                    
                                    // Code editor
                                    ui.group(|ui| {
                                        ui.label("Your Solution:");
                                        ui.add_space(4.0);
                                        
                                        let mut code = challenge.initial_code.clone();
                                        ui.add_sized(
                                            [ui.available_width(), 150.0],
                                            egui::TextEdit::multiline(&mut code)
                                                .font(egui::FontId::new(14.0, egui::FontFamily::Monospace))
                                                .code_editor()
                                                .desired_rows(8)
                                                .lock_focus(true)
                                        );
                                        
                                        ui.add_space(8.0);
                                        
                                        if ui.button("Submit Solution").clicked() {
                                            self.game_log.push("Solution submitted for checking...".to_string());
                                        }
                                    });
                                }
                            }
                        }
                    });
            });
    }
}

// modify your main function

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we want to run in web mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--web" {
        server::run_server().await?;
        Ok(())
    } else {
        // Run native GUI
        let options = eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size([800.0, 600.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Rust Warrior",
            options,
            Box::new(|_cc| Box::new(GameState::new())),
        )?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Quest {
    name: String,
    description: String,
    rust_concept: String,
    completed: bool,
    code_challenge: Option<CodeChallenge>,
}

impl Quest {
    fn with_challenge(name: &str, rust_concept: &str, challenge: CodeChallenge) -> Self {
        Quest {
            name: name.to_string(),
            description: "Complete this challenge to advance.".to_string(),
            rust_concept: rust_concept.to_string(),
            completed: false,
            code_challenge: Some(challenge),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CodeChallenge {
    initial_code: String,
    solution_template: String,
    description: String,
    test_cases: Vec<TestCase>,
}

impl CodeChallenge {
    fn new(description: &str, initial_code: &str) -> Self {
        CodeChallenge {
            initial_code: initial_code.to_string(),
            solution_template: String::new(), // We'll set this later
            description: description.to_string(),
            test_cases: Vec::new(), // We'll add these later
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TestCase {
    input: String,
    expected_output: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Player {
    name: String,
    current_location: String,
    inventory: Vec<String>,
    completed_quests: Vec<String>,
    health: i32,
    damage: i32,
}

impl Player {
    fn new(name: &str, start_location: &str) -> Self {
        Player {
            name: name.to_string(),
            current_location: start_location.to_string(),
            inventory: Vec::new(),
            completed_quests: Vec::new(),
            health: 100,
            damage: 10,
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

#[derive(Debug, Clone)]
struct Enemy {
    name: String,
    symbol: char,
    position: (usize, usize),
    health: i32,
    damage: i32,
}

impl Enemy {
    fn new(name: &str, health: i32, damage: i32) -> Self {
        Enemy {
            name: name.to_string(),
            symbol: 'a',
            position: (0, 0),
            health,
            damage,
        }
    }
}

#[derive(Debug, Clone)]
struct BattleState {
    map: Vec<Vec<char>>,
    warrior_position: (usize, usize),
    turn: u32,
    warrior_health: i32,
    enemies: Vec<Enemy>,
    battle_log: Vec<String>,
}

impl BattleState {
    fn new() -> Self {
        // Create initial map for Level 1
        let map = vec![
            vec!['╔', '═', '═', '═', '═', '═', '═', '═', '╗'],
            vec!['║', '@', ' ', 'a', ' ', 'S', ' ', '>', '║'],
            vec!['╚', '═', '═', '═', '═', '═', '═', '═', '╝'],
        ];

        let mut enemies = Vec::new();
        enemies.push(Enemy {
            name: "Thick Sludge".to_string(),
            health: 24,
            damage: 3,
            symbol: 'a',
            position: (1, 1),
        });
        enemies.push(Enemy {
            name: "Archer".to_string(),
            health: 15,
            damage: 3,
            symbol: 'a',
            position: (1, 3),
        });

        BattleState {
            map,
            warrior_position: (1, 1),
            turn: 1,
            warrior_health: 20,
            enemies,
            battle_log: Vec::new(),
        }
    }

    fn execute_turn(&mut self, action: &str) {
        self.battle_log.push(format!("\nturn {:03}", self.turn));
        
        match action {
            "walk" => self.warrior_walk(),
            "attack" => self.warrior_attack(),
            "rest" => self.warrior_rest(),
            _ => self.battle_log.push("Invalid action".to_string()),
        }

        // Enemy turns
        self.enemy_actions();
        self.turn += 1;
    }

    fn warrior_walk(&mut self) {
        let (row, col) = self.warrior_position;
        if col + 1 < self.map[row].len() && self.map[row][col + 1] == ' ' {
            self.map[row][col] = ' ';
            self.map[row][col + 1] = '@';
            self.warrior_position = (row, col + 1);
            self.battle_log.push("Warrior walks forward".to_string());
        }
    }

    fn warrior_attack(&mut self) {
        let (row, col) = self.warrior_position;
        for enemy in &mut self.enemies {
            if enemy.position == (row, col + 1) {
                enemy.health -= 5;
                self.battle_log.push(format!(
                    "Warrior attacks {} dealing 5 damage ({} HP remaining)",
                    enemy.name, enemy.health
                ));
                if enemy.health <= 0 {
                    self.battle_log.push(format!("{} is defeated!", enemy.name));
                    // Remove enemy from map
                    self.map[row][col + 1] = ' ';
                }
                break;
            }
        }
    }

    fn warrior_rest(&mut self) {
        if self.warrior_health < 20 {
            self.warrior_health += 2;
            self.battle_log.push(format!(
                "Warrior rests and recovers 2 HP ({} HP total)",
                self.warrior_health
            ));
        }
    }

    fn enemy_actions(&mut self) {
        // Remove defeated enemies
        self.enemies.retain(|e| e.health > 0);

        // Each surviving enemy attacks if in range
        for enemy in &self.enemies {
            let (ey_row, ey_col) = enemy.position;
            let (w_row, w_col) = self.warrior_position;
            
            if enemy.symbol == 'a' || // Archer can attack from distance
               (ey_row == w_row && (ey_col as i32 - w_col as i32).abs() == 1) // Melee range
            {
                self.warrior_health -= enemy.damage;
                self.battle_log.push(format!(
                    "{} attacks warrior dealing {} damage ({} HP remaining)",
                    enemy.name, enemy.damage, self.warrior_health
                ));

                if self.warrior_health <= 0 {
                    self.battle_log.push("Warrior has been defeated!".to_string());
                }
            }
        }
    }

    fn render(&self) -> String {
        self.map.iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }
}