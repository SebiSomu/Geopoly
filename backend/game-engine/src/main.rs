mod game;
mod board;
mod player;
mod cards;
mod passport;

use game::Game;
use colored::*;

fn main() {
    println!("{}", "=".repeat(60).bright_blue());
    println!("{}", "MONOPOLY HERE & NOW - World Edition".bright_yellow().bold());
    println!("{}", "=".repeat(60).bright_blue());
    println!();

    // Creăm jocul cu 4 jucători
    let mut game = Game::new(vec![
        "Jucător 1".to_string(),
        "Jucător 2".to_string(),
        "Jucător 3".to_string(),
        "Jucător 4".to_string(),
    ]);

    println!("{}", "Jocul a început! Primul jucător care își umple pașaportul câștigă!".green());
    println!();

    // Loop principal de joc
    game.play();
}