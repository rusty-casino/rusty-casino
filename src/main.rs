use easy_api;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json;

mod database;
use database::Database;

mod crypto;

mod games;

const ADDR: &str = "0.0.0.0:80";

fn pong() -> easy_api::EasyResult {
    println!("Pong");
    Ok(())
}


fn bet_wrapper(db: &mut Database, received: String, game_functions: &HashMap<String, games::GameFunction>) -> Result<bool, Box<dyn std::error::Error>> {
    let mut parts = received.split_whitespace();

    let player_tok = parts.next().ok_or("Missing token")?.to_owned();
    let market_id: i64 = parts.next().ok_or("Missing market ID")?.parse()?;
    let wager: i64 = parts.next().ok_or("Missing wager")?.parse()?;

    if parts.next().is_some() {
        return Err("Too many arguments".into());
    }

    let player_id = db.get_player_id_token(player_tok)?;

    let game_type = db.get_game_type_for_market(market_id)?;
    let game_func = game_functions
        .get(&game_type)
        .ok_or("Game implementation not found")?;

    let bet_id = db.place_bet(player_id, market_id, wager)?;

    let round_id = db.get_round_id(bet_id)?;
    db.play_game(*game_func, round_id)?;

    let win = db.payout_bet(bet_id)?;

    Ok(win)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    easy_api::set_verbose(true);
    // Serve Client Directory
    easy_api::static_files::serve_static("/", "client");

    let game_functions = games::init();

    let mut db = Arc::new(Mutex::new(Database::new("casino.db")?));

    easy_api::add_tap_api("/api/ping", pong);

    let cdb = Arc::clone(&mut db);
    easy_api::add_grab_give_api("/api/bet", move |received| {
        let result = bet_wrapper(
            &mut cdb.lock().unwrap(),
            received,
            &game_functions
        );

        match result {
            Ok(true) => Ok("win".to_string()),
            Ok(false) => Ok("loss".to_string()),
            Err(e) => {
                eprintln!("Bet Failed! {}", e);
                Err(500)
            }
        }
    });

    let c2db = Arc::clone(&mut db);
    easy_api::add_grab_give_api("/api/login", move |recived| {
        let srecv: Vec<&str> = recived.split(' ').collect();
        let (username, password): (String, String) = (srecv[0].to_owned(), srecv[1].to_owned());

        let out = crypto::login(&mut c2db.lock().unwrap(), username, password);
        if out.is_err() {
            eprintln!("Login Failed! {}", out.unwrap_err());
            return Err(500); 
        }
        let out = out.unwrap();
        if out == "x" {
            return Err(401);
        }
        Ok(out)
    });

    let c3db = Arc::clone(&mut db);
    easy_api::add_grab_give_api("/api/signup", move |recived| {
        let srecv: Vec<&str> = recived.split(' ').collect();
        let (username, password): (String, String) = (srecv[0].to_owned(), srecv[1].to_owned());

        let out = crypto::signup(&mut c3db.lock().unwrap(), username, password);
        if out.is_err() {
            eprintln!("Signup Failed! {}", out.unwrap_err());
            return Err(500); 
        }
         Ok(out.unwrap())
    });

    let c4db = Arc::clone(&mut db);
    easy_api::add_tap_give_api("/api/games", move || {
        let games = c4db.lock().unwrap().get_games();
        if games.is_err() {
            eprintln!("Get Games Failed! {}", games.unwrap_err());
            return Err(500);
        }

        let json = serde_json::to_string(&games.unwrap());
        if json.is_err() {
            eprintln!("Get Games Failed! {}", json.unwrap_err());
            return Err(500);
        }
        Ok(json.unwrap())
    });

    easy_api::start_on(ADDR);
    loop {
        easy_api::park();
    }
}
