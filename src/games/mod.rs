pub mod coin_flip;
use std::collections::HashMap;

pub type GameFunction = fn() -> String;

pub fn init() -> HashMap<String, GameFunction> {
    let mut game_functions: HashMap<String, GameFunction> = HashMap::new();
    game_functions.insert(
        "coin-flip".to_string(),
        coin_flip::play_coin_flip
    );

    game_functions
}
