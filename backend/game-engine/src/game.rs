use crate::board::{Board, Space, Destination};
use crate::player::Player;
use crate::cards::{ChanceDeck, HereAndNowDeck, ChanceCardAction, HereAndNowCardAction};
use crate::passport::Stamp;
use colored::*;
use rand::Rng;
use std::io::{self, Write};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DiceResult {
    Normal(u8, u8),
    Double(u8),
    BusinessDeal(u8)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameStep {
    WaitingForRoll,
    WaitingForForcedDeal,
    WaitingForPurchaseDecision { dest_id: u8, price: i32, buyer_idx: usize },
    WaitingForFirstClassDecision { buyer_idx: usize },
    WaitingForAirportDecision { buyer_idx: usize },
    WaitingForAirportDestination { buyer_idx: usize },
    WaitingForTargetSelection { action: String, card_id: Option<String>, selector_idx: usize },
    WaitingForDiceDuel { 
        challenger_idx: usize, 
        target_idx: usize, 
        challenger_roll: Option<(u8, u8)>, 
        target_roll: Option<(u8, u8)> 
    },
    WaitingForDiceDuelResult { 
        challenger_idx: usize, 
        target_idx: usize, 
        challenger_roll: (u8, u8), 
        target_roll: (u8, u8) 
    },
    WaitingForAuction { dest_id: u8, current_bid: i32, highest_bidder: Option<usize> },
    WaitingForJailDecision,
    WaitingForRerollDice { player_idx: usize },
}

// Add a helper struct to track pending reroll state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingReroll {
    pub player_idx: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JailAction {
    PayFine,
    UseCard,
    Roll,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameAction {
    Payment { from: Option<usize>, to: Option<usize>, amount: i32, initiator: Option<usize> },
    StampTransfer { from: Option<usize>, to: Option<usize>, stamp_name: String, stamp_id: String, is_first_class: bool, initiator: Option<usize> },
    GoToJail { player_idx: usize },
    Move { player_idx: usize, from: u8, to: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLogEntry {
    pub player_idx: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub die1: u8,
    pub die2: u8,
    pub is_double: bool,
    pub is_forced_deal: bool,
    pub new_position: u8,
    pub went_to_jail: bool,
    pub turn_ends: bool,
    pub current_player_index: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRecord {
    pub dest_id: u8,
    pub buyer_idx: usize,
    pub price: i32,
    pub name: String,
    pub is_first_class: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub board: Board,
    pub players: Vec<Player>,
    pub current_player_idx: usize,
    pub chance_deck: ChanceDeck,
    pub here_and_now_deck: HereAndNowDeck,
    pub first_class_stamps_available: u8,
    pub turn_number: u32,
    pub game_over: bool,
    pub step: GameStep,
    pub last_dice: Option<(u8, u8)>,
    pub history: Vec<GameAction>,
    pub activity_log: Vec<ActivityLogEntry>,
    pub last_purchase: Option<PurchaseRecord>,
    pub pending_reroll: Option<PendingReroll>,
}

impl Game {
    pub fn new(player_names: Vec<String>) -> Self {
        let board = Board::new();
        let mut players: Vec<Player> = player_names
            .into_iter()
            .map(|name| Player::new(name))
            .collect();

        let mut here_and_now_deck = HereAndNowDeck::new();

        // Fiecare jucător primește 2 cartonașe Here&Now la început
        for player in &mut players {
            let (card1, card2) = here_and_now_deck.deal_initial();
            player.here_and_now_cards.push(card1);
            player.here_and_now_cards.push(card2);
        }

        Game {
            board,
            players,
            current_player_idx: 0,
            chance_deck: ChanceDeck::new(),
            here_and_now_deck,
            first_class_stamps_available: 20,
            turn_number: 1,
            game_over: false,
            step: GameStep::WaitingForRoll,
            last_dice: None,
            history: Vec::new(),
            activity_log: Vec::new(),
            last_purchase: None,
            pending_reroll: None,
        }
    }

    fn log_action(&mut self, player_idx: Option<usize>, message: String) {
        self.activity_log.push(ActivityLogEntry { player_idx, message });
    }

    pub fn update_all_reactive_statuses(&mut self) {
        let players_count = self.players.len();
        for i in 0..players_count {
            self.players[i].can_use_say_no = self.check_can_player_say_no(i);
            self.players[i].can_use_discount = self.check_can_player_use_discount(i);
            self.players[i].can_use_intercept = self.check_can_player_use_intercept(i);
            self.players[i].can_use_collect_tax = self.check_can_player_use_collect_tax(i);
            self.players[i].can_use_steal_first_class = self.check_can_player_use_steal_first_class(i);
        }
    }

    pub fn check_can_player_say_no(&self, player_idx: usize) -> bool {
        self.history.iter().rev().take(3).any(|action| {
             match action {
                GameAction::Payment { from, to, initiator, .. } => *from == Some(player_idx) && to.is_some() && *initiator != Some(player_idx),
                GameAction::StampTransfer { from, initiator, .. } => 
                    *from == Some(player_idx) && *initiator != Some(player_idx),
                _ => false
            }
        })
    }

    pub fn check_can_player_use_discount(&self, player_idx: usize) -> bool {
        // 1. Reactive: recently paid > 100 to board
        let recently_paid = self.history.iter().rev().take(10).any(|action| {
            if let GameAction::Payment { from, to: None, amount, .. } = action {
                *from == Some(player_idx) && *amount > 100
            } else {
                false
            }
        });

        // 2. Proactive: currently in a step waiting for purchase decision for THIS player
        let is_waiting_for_decision = match &self.step {
            GameStep::WaitingForPurchaseDecision { buyer_idx, price, .. } => *buyer_idx == player_idx && *price > 100,
            _ => false
        };

        recently_paid || is_waiting_for_decision
    }

    pub fn check_can_player_use_intercept(&self, player_idx: usize) -> bool {
        if let Some(record) = &self.last_purchase {
            record.buyer_idx != player_idx
        } else {
            false
        }
    }

    pub fn check_can_player_use_collect_tax(&self, player_idx: usize) -> bool {
        self.history.iter().rev().take(10).any(|action| {
            if let GameAction::Payment { from, to: Some(_), .. } = action {
                *from == Some(player_idx)
            } else {
                false
            }
        })
    }

    pub fn check_can_player_use_steal_first_class(&self, player_idx: usize) -> bool {
        if let Some(record) = &self.last_purchase {
            record.is_first_class && record.buyer_idx != player_idx
        } else {
            false
        }
    }

    pub fn roll_dice(&mut self) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over".to_string());
        }
        if !matches!(self.step, GameStep::WaitingForRoll | GameStep::WaitingForRerollDice { .. }) {
            return Err("Not waiting for roll".to_string());
        }
        
        // Handle reroll dice case
        if let GameStep::WaitingForRerollDice { player_idx } = self.step {
            if self.current_player_idx != player_idx {
                return Err("Not your turn to reroll".to_string());
            }
            return self.resolve_reroll_dice();
        }

        let player_idx = self.current_player_idx;
        let in_jail = self.players[player_idx].in_jail;

        // --- Logică Închisoare ---
        if in_jail {
            return self.handle_jail_roll();
        }

        // --- Logică Normală ---
        let dice_result = Self::roll_dice_internal(); // Folosim funcția statică existentă
        self.display_dice_result(&dice_result);

        match dice_result {
            DiceResult::BusinessDeal(val) => {
                let d2 = val;
                self.last_dice = Some((1, d2));
                self.players[player_idx].consecutive_doubles = 0;

                if !self.can_anyone_swap(player_idx) {
                    println!("\n{}", "💼 AFACERE FORȚATĂ! (Imposibil de făcut schimb => mutare automată)".bright_magenta());
                    self.move_player(d2 as i32);
                    self.handle_landing(player_idx);
                    
                    let turn_ends = match self.step {
                        GameStep::WaitingForPurchaseDecision { .. } |
                        GameStep::WaitingForFirstClassDecision { .. } |
                        GameStep::WaitingForAirportDecision { .. } |
                        GameStep::WaitingForAirportDestination { .. } |
                        GameStep::WaitingForTargetSelection { .. } |
                        GameStep::WaitingForDiceDuel { .. } |
                        GameStep::WaitingForAuction { .. } |
                        GameStep::WaitingForForcedDeal |
                        GameStep::WaitingForRerollDice { .. } => false,
                        _ => true,
                    };

                    if turn_ends {
                        self.end_turn();
                    }

                    Ok(TurnResult {
                        die1: 1, 
                        die2: d2, 
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends,
                        current_player_index: self.current_player_idx as u8,
                    })
                } else {
                    println!("\n{}", "💼 AFACERE FORȚATĂ!".bright_magenta());
                    self.step = GameStep::WaitingForForcedDeal;
                    
                    Ok(TurnResult {
                        die1: 1, 
                        die2: d2, 
                        is_double: false,
                        is_forced_deal: true,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: false, // Așteaptă input
                        current_player_index: self.current_player_idx as u8,
                    })
                }
            }
            DiceResult::Normal(d1, d2) => {
                let total = d1 + d2;
                self.last_dice = Some((d1, d2));
                self.move_player(total as i32);
                self.handle_landing(player_idx);
                self.players[player_idx].consecutive_doubles = 0; // Reset doubles

                // Only end turn if no decisions are pending
                let turn_ends = match self.step {
                        GameStep::WaitingForPurchaseDecision { .. } |
                        GameStep::WaitingForFirstClassDecision { .. } |
                        GameStep::WaitingForAirportDecision { .. } |
                        GameStep::WaitingForAirportDestination { .. } |
                        GameStep::WaitingForTargetSelection { .. } |
                        GameStep::WaitingForDiceDuel { .. } |
                        GameStep::WaitingForAuction { .. } |
                        GameStep::WaitingForForcedDeal |
                        GameStep::WaitingForRerollDice { .. } => false,
                        _ => true,
                    };

                if turn_ends {
                    self.end_turn();
                }

                Ok(TurnResult {
                    die1: d1,
                    die2: d2,
                    is_double: false,
                    is_forced_deal: false,
                    new_position: self.players[player_idx].position as u8,
                    went_to_jail: false,
                    turn_ends,
                    current_player_index: self.current_player_idx as u8,
                })
            }
            DiceResult::Double(val) => {
                let d1 = val;
                let d2 = val;
                let total = d1 + d2;
                self.last_dice = Some((d1, d2));
                
                self.players[player_idx].consecutive_doubles += 1;

                if self.players[player_idx].consecutive_doubles >= 3 {
                    println!("\n{}", "🚫 A TREIA DUBLĂ! Mergi direct la ÎNCHISOARE!".red());
                    self.players[player_idx].send_to_jail();
                    self.players[player_idx].consecutive_doubles = 0;
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} rolled doubles 3 times in a row and got into prison", name));
                    self.end_turn();

                    return Ok(TurnResult {
                        die1: d1,
                        die2: d2,
                        is_double: true,
                        is_forced_deal: false,
                        new_position: 10,
                        went_to_jail: true,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    });
                }

                println!("\n{}", format!("Dublă #{}! Mută {} spații și mai arunci o dată!", self.players[player_idx].consecutive_doubles, total).bright_green());
                self.move_player(total as i32);
                self.handle_landing(player_idx);

                let moved_to_jail = self.players[player_idx].in_jail;
                if moved_to_jail || self.game_over {
                    self.end_turn();
                     return Ok(TurnResult {
                        die1: d1,
                        die2: d2,
                        is_double: true,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: moved_to_jail,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    });
                }

                // If a decision is pending, turn stops for input but doesn't technically end (still same player's turn context)
                let pending_decision = matches!(self.step,
                    GameStep::WaitingForTargetSelection { .. } |
                    GameStep::WaitingForDiceDuel { .. } |
                    GameStep::WaitingForPurchaseDecision { .. } |
                    GameStep::WaitingForFirstClassDecision { .. } |
                    GameStep::WaitingForAirportDecision { .. } |
                    GameStep::WaitingForAirportDestination { .. } |
                    GameStep::WaitingForAuction { .. } |
                    GameStep::WaitingForRerollDice { .. }
                );
                
                Ok(TurnResult {
                    die1: d1,
                    die2: d2,
                    is_double: true,
                    is_forced_deal: false,
                    new_position: self.players[player_idx].position as u8,
                    went_to_jail: false,
                    turn_ends: pending_decision, // If false, client knows they can roll again after double. If true, it just means current roll action ends.
                    current_player_index: self.current_player_idx as u8,
                })
            }
        }
    }

    fn handle_jail_roll(&mut self) -> Result<TurnResult, String> {
        let player_idx = self.current_player_idx;
        self.players[player_idx].jail_turns += 1;

        println!("\n{}", "🔒 Încercare evadare din închisoare...".cyan());
        
        // Simplificare: DOAR aruncăm zarurile (fără opțiune de a plăti momentan în API roll)
        // În viitor am putea adăuga o metodă `pay_jail_fine`
        
        let dice_result = Self::roll_dice_internal();
        self.display_dice_result(&dice_result);

        let (d1, d2, is_forced_deal) = match dice_result {
            DiceResult::Normal(a, b) => (a, b, false),
            DiceResult::Double(a) => (a, a, false),
            DiceResult::BusinessDeal(b) => (1, b, true),
        };
        
        // Asigurăm că salvăm ultimele zaruri pentru sincronizarea cu UI-ul
        self.last_dice = Some((d1, d2));

        if let DiceResult::Double(val) = dice_result {
            println!("\n{}", "🎉 Ai dat dublă! Ești liber!".bright_green());
            self.players[player_idx].release_from_jail();
            self.move_player((val * 2) as i32);
            self.handle_landing(player_idx);
            
            // Verificăm dacă tura se încheie (poate a aterizat pe o proprietate)
            let turn_ends = match self.step {
                        GameStep::WaitingForPurchaseDecision { .. } |
                        GameStep::WaitingForFirstClassDecision { .. } |
                        GameStep::WaitingForAirportDecision { .. } |
                        GameStep::WaitingForAirportDestination { .. } |
                        GameStep::WaitingForTargetSelection { .. } |
                        GameStep::WaitingForDiceDuel { .. } |
                        GameStep::WaitingForAuction { .. } |
                        GameStep::WaitingForForcedDeal => false,
                        _ => true,
                    };

            if turn_ends {
                self.end_turn();
            }
            
            Ok(TurnResult {
                die1: val, die2: val,
                is_double: true,
                is_forced_deal: false,
                new_position: self.players[player_idx].position as u8,
                went_to_jail: false,
                turn_ends,
                current_player_index: self.current_player_idx as u8,
            })
        } else {
            println!("\n{}", "Nu ai dat dublă.".yellow());
            let mut released = false;
            
            if self.players[player_idx].jail_turns >= 3 {
                 println!("{}", "Ai stat 3 ture! Plătești M100 și ieși automat (sau faliment).".yellow());
                 if self.players[player_idx].pay_money(100) {
                     self.players[player_idx].release_from_jail();
                     released = true;
                     let name = self.players[player_idx].name.clone();
                     self.log_action(Some(player_idx), format!("{} paid $100 to get out of prison", name));
                     let move_amount = d1 + d2;
                     self.move_player(move_amount as i32);
                     self.handle_landing(player_idx);
                 } else {
                     self.handle_bankruptcy(player_idx, None);
                     return Ok(TurnResult {
                        die1: d1, die2: d2,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: true,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    });
                 }
            }
            
            // Dacă a ieșit forțat, verificăm dacă e Afacere Forțată sau dacă trebuie să aleagă ceva
            if released && is_forced_deal && self.can_anyone_swap(player_idx) {
                 println!("\n{}", "💼 AFACERE FORȚATĂ!".bright_magenta());
                 self.step = GameStep::WaitingForForcedDeal;
                 // Nu mai e nevoie să setăm last_dice aici, e setat deja sus
            }

            let turn_ends = match self.step {
                        GameStep::WaitingForPurchaseDecision { .. } |
                        GameStep::WaitingForFirstClassDecision { .. } |
                        GameStep::WaitingForAirportDecision { .. } |
                        GameStep::WaitingForAirportDestination { .. } |
                        GameStep::WaitingForTargetSelection { .. } |
                        GameStep::WaitingForDiceDuel { .. } |
                        GameStep::WaitingForAuction { .. } |
                        GameStep::WaitingForForcedDeal |
                        GameStep::WaitingForRerollDice { .. } => false,
                        _ => true,
                    };

            if turn_ends {
                self.end_turn();
            }
            
            Ok(TurnResult {
                die1: d1, die2: d2,
                is_double: false,
                is_forced_deal: released && is_forced_deal && self.can_anyone_swap(player_idx),
                new_position: self.players[player_idx].position as u8,
                went_to_jail: !released,
                turn_ends,
                current_player_index: self.current_player_idx as u8,
            })
        }
    }

    pub fn resolve_jail_decision(&mut self, action: JailAction) -> Result<TurnResult, String> {
        if self.step != GameStep::WaitingForJailDecision {
            return Err("Not waiting for jail decision".to_string());
        }

        let player_idx = self.current_player_idx;
        if !self.players[player_idx].in_jail {
            return Err("Player is not in jail".to_string());
        }

        match action {
            JailAction::PayFine => {
                if self.players[player_idx].pay_money(100) {
                    println!("{}", "Ai plătit amenda de M100!".green());
                    self.players[player_idx].release_from_jail();
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} paid $100 up to get out of jail", name));
                    self.end_turn();
                    Ok(TurnResult {
                        die1: 0, die2: 0,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    })
                } else {
                    Err("Nu ai suficienți bani pentru amendă!".to_string())
                }
            },
            JailAction::UseCard => {
                // Check if player has the card
                if let Some(pos) = self.players[player_idx].chance_cards.iter().position(|c| c.id == "jail_free") {
                    let card = self.players[player_idx].chance_cards.remove(pos);
                    self.chance_deck.discard(card);
                    self.players[player_idx].release_from_jail();
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} used a Get Out of Jail Free card", name));
                    self.end_turn();
                    Ok(TurnResult {
                        die1: 0, die2: 0,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    })
                } else if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::GetOutOfJailFree)) {
                     let card = self.players[player_idx].here_and_now_cards.remove(pos);
                     self.here_and_now_deck.discard(card);
                     self.players[player_idx].release_from_jail();
                     let name = self.players[player_idx].name.clone();
                     self.log_action(Some(player_idx), format!("{} used a Get Out of Jail Free card", name));
                     self.end_turn();
                     Ok(TurnResult {
                        die1: 0, die2: 0,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    })
                } else {
                    Err("Nu ai un cartonaș 'Ieși din închisoare'!".to_string())
                }
            },
            JailAction::Roll => {
                self.handle_jail_roll()
            }
        }
    }

    pub fn resolve_reroll_dice(&mut self) -> Result<TurnResult, String> {
        if let GameStep::WaitingForRerollDice { player_idx } = self.step {
            if self.current_player_idx != player_idx {
                return Err("Not your turn to reroll".to_string());
            }

            let mut rng = rand::thread_rng();
            let d = rng.gen_range(1..=6);
            let old_pos = self.players[player_idx].position;
            println!("{}", format!("Arunci din nou zarul: {}. Te muți {} spații.", d, d).cyan());
            
            self.move_player(d as i32);
            let new_pos = self.players[player_idx].position;
            
            self.history.push(GameAction::Move {
                player_idx,
                from: old_pos as u8,
                to: new_pos as u8
            });
            
            self.handle_landing(player_idx);
            
            // Check if there are pending decisions that need to be made
            let has_pending_decision = matches!(self.step,
                GameStep::WaitingForPurchaseDecision { .. } |
                GameStep::WaitingForFirstClassDecision { .. } |
                GameStep::WaitingForAirportDecision { .. } |
                GameStep::WaitingForAirportDestination { .. } |
                GameStep::WaitingForTargetSelection { .. } |
                GameStep::WaitingForDiceDuel { .. } |
                GameStep::WaitingForAuction { .. } |
                GameStep::WaitingForForcedDeal
            );

            // Clear the pending reroll state - player has used their reroll
            self.pending_reroll = None;

            if has_pending_decision {
                // Keep the current step (the decision state) - don't end turn yet
            } else {
                // No pending decision - end the turn, no more rerolls allowed
                self.step = GameStep::WaitingForRoll;
                self.end_turn();
            }

            Ok(TurnResult {
                die1: d,
                die2: 0,
                is_double: false,
                is_forced_deal: false,
                new_position: self.players[player_idx].position as u8,
                went_to_jail: false,
                turn_ends: !has_pending_decision,
                current_player_index: self.current_player_idx as u8,
            })
        } else {
            Err("Not waiting for reroll dice".to_string())
        }
    }

    pub fn resolve_forced_deal(&mut self, action: &str, target_name: Option<String>) -> Result<TurnResult, String> {
        if self.step != GameStep::WaitingForForcedDeal {
                return Err("Not waiting for forced deal".to_string());
            }

            let player_idx = self.current_player_idx;

            match action {
                "SneakySwap" => {
                    let target_idx = if let Some(name) = target_name {
                        self.players.iter().position(|p| p.name == name)
                    } else {
                        None
                    };
                    self.handle_business_deal(player_idx, target_idx);
                    println!("Schimb de ștampile efectuat! Tura se încheie.");

                    // Pentru swap, tura se încheie IMEDIAT
                    self.step = GameStep::WaitingForRoll;
                    self.end_turn();

                    return Ok(TurnResult {
                        die1: 0, die2: 0,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    });
                },
                "move" => {
                    // Mută cu valoarea de pe die2 (salvată în last_dice)
                    let steps = if let Some((_, d2)) = self.last_dice { d2 as i32 } else { 1 };
                    println!("Ales 'Move': Mută {} spații", steps);
                    self.move_player(steps);
                    self.handle_landing(player_idx);
                }
                _ => return Err("Invalid action".to_string())
            }

            // Pentru move, verificăm dacă am ajuns într-o stare de așteptare decizie
            let turn_ends = match self.step {
                GameStep::WaitingForPurchaseDecision { .. } |
                GameStep::WaitingForFirstClassDecision { .. } |
                GameStep::WaitingForAirportDecision { .. } |
                GameStep::WaitingForAirportDestination { .. } |
                GameStep::WaitingForTargetSelection { .. } |
                GameStep::WaitingForDiceDuel { .. } |
                GameStep::WaitingForAuction { .. } |
                GameStep::WaitingForRerollDice { .. } => false,  // Așteaptă decizie
                _ => true,  // Altfel, tura se încheie
            };

            if turn_ends {
                self.step = GameStep::WaitingForRoll;
                self.end_turn();
            }
        
        self.update_all_reactive_statuses();

            Ok(TurnResult {
                die1: 0, die2: 0,
                is_double: false,
                is_forced_deal: false,
                new_position: self.players[player_idx].position as u8,
                went_to_jail: false,
                turn_ends,
                current_player_index: self.current_player_idx as u8,
            })
        }

    /// Rezolvă decizia de cumpărare a unei proprietăți
    pub fn resolve_purchase(&mut self, buy: bool) -> Result<TurnResult, String> {
            let (dest_id, price, buyer_idx) = match &self.step {
                GameStep::WaitingForPurchaseDecision { dest_id, price, buyer_idx } => (*dest_id, *price, *buyer_idx),
                _ => return Err("Not waiting for purchase decision".to_string()),
            };

            let player_idx = buyer_idx; // Use the buyer_idx from the state

            if buy {
                // Găsește destinația pentru a crea ștampila
                if let Some(dest) = self.board.find_destination_by_id(dest_id) {
                    let dest = dest.clone();
                    if self.players[player_idx].pay_money(price) {
                        // Înregistrăm achiziția pentru interceptare ulterioară
                        self.last_purchase = Some(PurchaseRecord {
                            dest_id,
                            buyer_idx: player_idx,
                            price,
                            name: dest.name.clone(),
                            is_first_class: false,
                        });

                        // Record Payment
                        self.history.push(GameAction::Payment {
                            from: Some(player_idx),
                            to: None,
                            amount: price,
                            initiator: None
                        });


                        // Cumpărarea proprietății (include verificare set și win)
                        if self.acquire_property(player_idx, &dest) {
                            self.step = GameStep::WaitingForRoll;
                            if buyer_idx == self.current_player_idx {
                                self.end_turn();
                            }
                            return Ok(TurnResult {
                                die1: 0, die2: 0,
                                is_double: false,
                                is_forced_deal: false,
                                new_position: self.players[buyer_idx].position as u8,
                                went_to_jail: false,
                                turn_ends: true,
                                current_player_index: self.current_player_idx as u8,
                            });
                        }
                    } else {
                        println!("{}", "Nu ai suficienți bani!".red());
                    }
                }
            } else {
            println!("Jucătorul a refuzat să cumpere proprietatea.");
            // Start auction: bidding starts at M20
            println!("{}", "🔨 AUCTION! Bidding starts at M20".bright_yellow());
            self.step = GameStep::WaitingForAuction {
                dest_id,
                current_bid: 20,
                highest_bidder: None,
            };
            return Ok(TurnResult {
                die1: 0, die2: 0,
                is_double: false,
                is_forced_deal: false,
                new_position: self.players[buyer_idx].position as u8,
                went_to_jail: false,
                turn_ends: false,
                current_player_index: self.current_player_idx as u8,
            });
        }

        self.step = GameStep::WaitingForRoll;
        if buyer_idx == self.current_player_idx { self.end_turn(); }

            Ok(TurnResult {
                die1: 0, die2: 0,
                is_double: false,
                is_forced_deal: false,
                new_position: self.players[buyer_idx].position as u8,
                went_to_jail: false,
                turn_ends: true,
                current_player_index: self.current_player_idx as u8,
            })
        }

    /// Rezolvă decizia de cumpărare First Class
    pub fn resolve_first_class(&mut self, buy: bool) -> Result<TurnResult, String> {
            let buyer_idx = match &self.step {
                GameStep::WaitingForFirstClassDecision { buyer_idx } => *buyer_idx,
                _ => return Err("Not waiting for first class decision".to_string()),
            };

            let player_idx = buyer_idx;

            if buy {
                if self.players[player_idx].pay_money(100) {
                    // Înregistrăm achiziția pentru interceptare ulterioară
                    self.last_purchase = Some(PurchaseRecord {
                        dest_id: 0, // Id fictiv pentru First Class
                        buyer_idx: player_idx,
                        price: 100,
                        name: "First Class".to_string(),
                        is_first_class: true,
                    });

                    // Record Payment
                    self.history.push(GameAction::Payment {
                        from: Some(player_idx),
                        to: None,
                        amount: 100,
                        initiator: None
                    });

                    println!("{}", format!("✅ {} a plătit M100 pentru First Class!", self.players[player_idx].name).green());

                    let fc_player_name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} bought First Class", fc_player_name));

                    let stamp = Stamp::first_class();

                    // Transfer direct de la bancă la cumpărător
                    self.history.push(GameAction::StampTransfer {
                        from: None,
                        to: Some(player_idx),
                        stamp_name: "First Class".to_string(),
                        stamp_id: "first_class".to_string(),
                        is_first_class: true,
                        initiator: Some(player_idx)
                    });

                    self.players[player_idx].passport.add_stamp(stamp);
                    self.first_class_stamps_available -= 1;

                    println!("{}", "✅ Ai cumpărat stampila Clasa Întâi!".green());

                    self.check_and_handle_win(player_idx);
                } else {
                    println!("{}", "Nu ai suficienți bani!".red());
                }
            } else {
                println!("Jucătorul a refuzat să cumpere First Class.");
            }

            self.step = GameStep::WaitingForRoll;
            if buyer_idx == self.current_player_idx { self.end_turn(); }

            Ok(TurnResult {
                die1: 0, die2: 0,
                is_double: false,
                is_forced_deal: false,
                new_position: self.players[player_idx].position as u8,
                went_to_jail: false,
                turn_ends: true,
                current_player_index: self.current_player_idx as u8,
            })
        }

    /// Rezolvă decizia de zbor de la Aeroport
    pub fn resolve_airport_decision(&mut self, buy_flight: bool) -> Result<TurnResult, String> {
        let buyer_idx = if let GameStep::WaitingForAirportDecision { buyer_idx } = self.step {
            buyer_idx
        } else {
            return Err("Not waiting for airport decision".to_string());
        };

        let player_idx = buyer_idx;

        if buy_flight {
            if self.players[player_idx].pay_money(100) {
                 // Record Payment
                 self.history.push(GameAction::Payment { 
                    from: Some(player_idx), 
                    to: None, 
                    amount: 100,
                    initiator: None
                });
                 println!("✅ Ai plătit M100 pentru zbor! Alege destinația.");
                 self.step = GameStep::WaitingForAirportDestination { buyer_idx: player_idx };
            } else {
                println!("{}", "Nu ai suficienți bani!".red());
                self.step = GameStep::WaitingForRoll;
                self.end_turn();
            }
        } else {
            println!("Jucătorul a refuzat zborul.");
            self.step = GameStep::WaitingForRoll;
            if player_idx == self.current_player_idx {
                self.end_turn();
            }
        }

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[player_idx].position as u8,
            went_to_jail: false,
            turn_ends: self.step == GameStep::WaitingForRoll,
            current_player_index: self.current_player_idx as u8,
        })
    }

    /// Rezolvă alegerea destinației de zbor
    pub fn resolve_airport_destination(&mut self, target_position: u8) -> Result<TurnResult, String> {
        let idx = if let GameStep::WaitingForAirportDestination { buyer_idx } = self.step {
            buyer_idx
        } else {
            return Err("Nu ești în etapa de a alege destinația de zbor!".to_string());
        };

        self.players[idx].position = target_position as usize;
        println!("🛬 {} a aterizat pe poziția {}!", self.players[idx].name, target_position);

        self.step = GameStep::WaitingForRoll; // Reset implicitly for now
        self.handle_landing(idx);

        let turn_ends = matches!(self.step, GameStep::WaitingForRoll);

        if turn_ends && idx == self.current_player_idx {
            self.end_turn();
        }

        Ok(TurnResult {
            die1: 0,
            die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[idx].position as u8,
            went_to_jail: false,
            turn_ends,
            current_player_index: self.current_player_idx as u8,
        })
    }

    pub fn resolve_target_selection(&mut self, target_name: String) -> Result<TurnResult, String> {
        let (action, _card_id, selector_idx) = match &self.step {
            GameStep::WaitingForTargetSelection { action, card_id, selector_idx } => (action.clone(), card_id.clone(), *selector_idx),
            _ => return Err("Nu ești în etapa de a alege un jucător!".to_string()),
        };

        // Note: Turn validation should happen in schema.rs regarding selector_idx vs current user session

        let target_idx = self.players.iter().position(|p| p.name == target_name)
            .ok_or("Jucătorul țintă nu există!")?;
        
        if target_idx == selector_idx {
             return Err("Nu te poți alege pe tine!".to_string());
        }

        match action.as_str() {
            "SwapStamps" => {
                self.handle_stamp_swap(selector_idx, Some(target_idx));
            },
            "DiceDuel" => {
                println!("🎲 Începe duelul cu zaruri între {} și {}!", self.players[selector_idx].name, self.players[target_idx].name);
                self.step = GameStep::WaitingForDiceDuel {
                    challenger_idx: selector_idx,
                    target_idx,
                    challenger_roll: None,
                    target_roll: None
                };
                
                return Ok(TurnResult {
                    die1: 0, die2: 0, 
                    is_double: false, is_forced_deal: false,
                    new_position: self.players[selector_idx].position as u8,
                    went_to_jail: false,
                    turn_ends: false, 
                    current_player_index: self.current_player_idx as u8 
                });
            },
            "SneakySwap" => {
                self.handle_business_deal(selector_idx, Some(target_idx));
            }
            "StealStampAndPay" => {
                self.handle_steal_stamp_and_pay(selector_idx, target_idx);
            }
            _ => return Err("Acțiune necunoscută".to_string())
        }

        // If action implies immediate completion (like swaps), we reset state
        // Check if we are in DiceDuel, if so, don't reset
        let is_duel = matches!(self.step, GameStep::WaitingForDiceDuel { .. });
        
        if !is_duel {
            // Check if this action was initiated by the current player or by another player
            let action_by_current_player = selector_idx == self.current_player_idx;
            
            if action_by_current_player {
                // Current player used a card - restore their reroll or end turn
                if let Some(pending_reroll) = &self.pending_reroll {
                    if pending_reroll.player_idx == selector_idx {
                        self.step = GameStep::WaitingForRerollDice { player_idx: selector_idx };
                    } else {
                        self.step = GameStep::WaitingForRoll;
                        self.end_turn();
                    }
                } else {
                    self.step = GameStep::WaitingForRoll;
                    self.end_turn();
                }
            } else {
                // Another player used a reactive card against the current player
                // Preserve the current player's reroll state - they should still get to roll the reroll
                if self.pending_reroll.is_some() {
                    // Restore reroll state for the current player
                    self.step = GameStep::WaitingForRerollDice { player_idx: self.current_player_idx };
                } else {
                    // No reroll pending, just continue with normal turn
                    self.step = GameStep::WaitingForRoll;
                }
            }
        }

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[self.current_player_idx].position as u8,
            went_to_jail: false,
            turn_ends: !is_duel,
            current_player_index: self.current_player_idx as u8,
        })
    }

    pub fn roll_duel_dice(&mut self) -> Result<TurnResult, String> {
        let (challenger_idx, target_idx, mut c_roll, mut t_roll) = match &self.step {
            GameStep::WaitingForDiceDuel { challenger_idx, target_idx, challenger_roll, target_roll } => 
                (*challenger_idx, *target_idx, *challenger_roll, *target_roll),
            _ => return Err("Nu ești într-un duel de zaruri!".to_string()),
        };

        // Determine who is rolling based on current state
        let roller_idx = if c_roll.is_none() { challenger_idx } else { target_idx };
        
        // Roll TWO dice
        let d1 = rand::thread_rng().gen_range(1..=6);
        let d2 = rand::thread_rng().gen_range(1..=6);
        let sum = d1 + d2;
        println!("🎲 {} a dat {} ({} + {}) în duel!", self.players[roller_idx].name, sum, d1, d2);

        if c_roll.is_none() {
            c_roll = Some((d1, d2));
        } else {
            t_roll = Some((d1, d2));
        }

        self.step = GameStep::WaitingForDiceDuel {
            challenger_idx,
            target_idx,
            challenger_roll: c_roll,
            target_roll: t_roll
        };

        if let (Some((c1, c2)), Some((t1, t2))) = (c_roll, t_roll) {
            self.step = GameStep::WaitingForDiceDuelResult {
                challenger_idx,
                target_idx,
                challenger_roll: (c1, c2),
                target_roll: (t1, t2)
            };

            return Ok(TurnResult {
                die1: t1, die2: t2, // Show target's roll
                is_double: t1 == t2,
                is_forced_deal: false,
                new_position: self.players[self.current_player_idx].position as u8,
                went_to_jail: false,
                turn_ends: false,
                current_player_index: self.current_player_idx as u8,
            });
        }

        Ok(TurnResult {
            die1: d1, die2: d2,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[self.current_player_idx].position as u8,
            went_to_jail: false,
            turn_ends: false,
            current_player_index: self.current_player_idx as u8,
        })
    }

    pub fn resolve_dice_duel(&mut self) -> Result<(), String> {
        let (challenger_idx, target_idx, c1, c2, t1, t2) = match &self.step {
            GameStep::WaitingForDiceDuelResult { challenger_idx, target_idx, challenger_roll: (c1, c2), target_roll: (t1, t2) } => 
                (*challenger_idx, *target_idx, *c1, *c2, *t1, *t2),
            _ => return Err("Nu există un duel de finalizat!".to_string()),
        };

        let cr_sum = c1 + c2;
        let tr_sum = t1 + t2;
        println!("🏁 Rezultat Duel FINAL: {} ({}) vs {} ({})", self.players[challenger_idx].name, cr_sum, self.players[target_idx].name, tr_sum);
        
        if cr_sum > tr_sum {
             println!("🏆 {} câștigă duelul! Primește M100.", self.players[challenger_idx].name);
             self.players[target_idx].pay_money(100);
             self.players[challenger_idx].add_money(100);
             self.history.push(GameAction::Payment { 
                 from: Some(target_idx), 
                 to: Some(challenger_idx), 
                 amount: 100,
                 initiator: None
             });
             let d_loser = self.players[target_idx].name.clone();
             let d_winner = self.players[challenger_idx].name.clone();
             self.log_action(Some(target_idx), format!("{} paid $100 to {}", d_loser, d_winner));
        } else if tr_sum > cr_sum {
             println!("🏆 {} câștigă duelul! Primește M100.", self.players[target_idx].name);
             self.players[challenger_idx].pay_money(100);
             self.players[target_idx].add_money(100);
             self.history.push(GameAction::Payment { 
                 from: Some(challenger_idx), 
                 to: Some(target_idx), 
                 amount: 100,
                 initiator: None
             });
             let d_loser2 = self.players[challenger_idx].name.clone();
             let d_winner2 = self.players[target_idx].name.clone();
             self.log_action(Some(challenger_idx), format!("{} paid $100 to {}", d_loser2, d_winner2));
        } else {
             println!("🤝 Egalitate! Nimeni nu plătește.");
        }

        self.step = GameStep::WaitingForRoll;
        self.end_turn();
        self.update_all_reactive_statuses();
        Ok(())
    }

    /// Folosește un cartonaș Here&Now din mână
    pub fn use_here_and_now_card(&mut self, player_idx: usize, card_id: String) -> Result<TurnResult, String> {
        let card_id_inner = card_id.clone();
        let card = self.players[player_idx].here_and_now_cards.iter().find(|c| c.id == card_id_inner)
            .ok_or_else(|| "Nu deții acest cartonaș!".to_string())?.clone();

        self.execute_here_and_now_action(player_idx, card.action.clone())?;
        
        // Remove only after success
        if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| c.id == card_id) {
            self.players[player_idx].here_and_now_cards.remove(pos);
        }
        
        let player_name = self.players[player_idx].name.clone();
        println!("🎭 {} folosește cartonașul: {}", player_name, card.description);
        self.log_action(Some(player_idx), format!("{} used treasure card: {}", player_name, card.description));
        self.here_and_now_deck.discard(card);
        self.update_all_reactive_statuses();

        // Check if action put us in a wait state
        let turn_ends = match self.step {
             GameStep::WaitingForTargetSelection { .. } |
             GameStep::WaitingForDiceDuel { .. } |
             GameStep::WaitingForPurchaseDecision { .. } | 
             GameStep::WaitingForFirstClassDecision { .. } | 
             GameStep::WaitingForAirportDecision { .. } | 
             GameStep::WaitingForAirportDestination { .. } | 
             GameStep::WaitingForForcedDeal |
             GameStep::WaitingForRerollDice { .. } => false,
             _ => {
                 // After using a Here & Now card, stay in current state
                 // (player can still roll the reroll die if they have one pending)
                 // Don't end the turn yet
                 false
             }
        };
        
        Ok(TurnResult {
            die1: 0,
            die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[player_idx].position as u8,
            went_to_jail: false,
            turn_ends, 
            current_player_index: self.current_player_idx as u8,
        })
    }

    fn execute_here_and_now_action(&mut self, player_idx: usize, action: HereAndNowCardAction) -> Result<(), String> {
        match action {
            HereAndNowCardAction::MoveSteps(steps) => {
                if player_idx != self.current_player_idx {
                    return Err("Poți folosi acest cartonaș doar în tura ta!".to_string());
                }
                let board_size = self.board.spaces.len();
                if self.players[player_idx].move_by(steps, board_size) {
                    println!("💰 Ai trecut pe la START! Primești M200.");
                    self.players[player_idx].add_money(200);
                }
                self.handle_landing(player_idx);
            }
            HereAndNowCardAction::MoveAnywhere => {
                if player_idx != self.current_player_idx {
                    return Err("Poți folosi acest cartonaș doar în tura ta!".to_string());
                }
                self.step = GameStep::WaitingForAirportDestination { buyer_idx: player_idx };
                println!("🚀 Alege orice destinație de pe tablă!");
            }
            HereAndNowCardAction::GetOutOfJailFree => {
                if !self.players[player_idx].in_jail {
                    return Err("Nu ești în închisoare!".to_string());
                }
                self.players[player_idx].release_from_jail();
                println!("🔓 Ai ieșit gratuit din închisoare!");
            }
            HereAndNowCardAction::SwapStamps => {
                 if self.players[player_idx].passport.stamp_count() == 0 {
                     return Err("Nu ai nicio ștampilă de oferit la schimb!".to_string());
                 }
                 // Trigger target selection directly
                 self.step = GameStep::WaitingForTargetSelection {
                    action: "SwapStamps".to_string(),
                    card_id: None,
                    selector_idx: player_idx
                 };
            }
            HereAndNowCardAction::TakeAllLastStamps => {
                let has_others_with_stamps = self.players.iter().enumerate()
                    .any(|(i, p)| i != player_idx && p.passport.stamp_count() > 0);
                
                if !has_others_with_stamps {
                    return Err("Niciun alt jucător nu are ștampile!".to_string());
                }

                for i in 0..self.players.len() {
                    if i != player_idx {
                        if let Some(stamp) = self.players[i].passport.remove_last_stamp() {
                             println!("🚫 Ștampila {} a lui {} a fost scoasă din pașaport!", stamp.name, self.players[i].name);
                             
                             self.log_action(Some(i), format!("Last stamp ({}) removed from passport", stamp.name));
                             
                             self.history.push(GameAction::StampTransfer {
                                 from: Some(i),
                                 to: None,
                                 stamp_name: stamp.name.clone(),
                                 stamp_id: format!("{}", stamp.destination_id.unwrap_or(0)),
                                 is_first_class: stamp.destination_id.is_none(),
                                 initiator: Some(player_idx),
                             });
                        }
                    }
                }
            }
            HereAndNowCardAction::CollectFromRichest => {
                let opponents: Vec<usize> = (0..self.players.len())
                    .filter(|&i| i != player_idx)
                    .collect();
                
                if opponents.is_empty() {
                    return Err("Nu există alți jucători!".to_string());
                }

                // Identificăm numărul maxim de ștampile printre adversari
                let max_stamps = opponents.iter()
                    .map(|&i| self.players[i].passport.stamp_count())
                    .max()
                    .unwrap_or(0);

                if max_stamps == 0 {
                    return Err("Niciun adversar nu are ștampile!".to_string());
                }

                let my_stamps = self.players[player_idx].passport.stamp_count();
                if max_stamps <= my_stamps {
                    return Err("Nimeni nu are mai multe ștampile decât tine!".to_string());
                }

                // Jucătorii de pe primul loc (pot fi mai mulți)
                let mut richest: Vec<usize> = opponents.into_iter()
                    .filter(|&i| self.players[i].passport.stamp_count() == max_stamps)
                    .collect();

                // Sortăm "richest" după înălțimea totală a pașaportului (tie-break)
                richest.sort_by(|&a, &b| {
                    let h_a = self.players[a].passport.left_height_used + self.players[a].passport.right_height_used;
                    let h_b = self.players[b].passport.left_height_used + self.players[b].passport.right_height_used;
                    h_b.partial_cmp(&h_a).unwrap_or(std::cmp::Ordering::Equal)
                });

                let count = richest.len();
                let payments = if count == 1 {
                    vec![(richest[0], 200)]
                } else if count == 2 {
                    vec![(richest[0], 100), (richest[1], 100)]
                } else {
                    // Dacă sunt 3 sau mai mulți, top 2 plătesc 70, al 3-lea plătește 60. Restul nimic (limităm la 200 total)
                    let mut p = Vec::new();
                    if richest.len() >= 1 { p.push((richest[0], 70)); }
                    if richest.len() >= 2 { p.push((richest[1], 70)); }
                    if richest.len() >= 3 { p.push((richest[2], 60)); }
                    p
                };

                for (idx, amount) in payments {
                    let actual_amount = if self.players[idx].money >= amount {
                        amount
                    } else {
                        self.players[idx].money.max(0)
                    };

                    if actual_amount > 0 {
                        self.players[idx].pay_money(actual_amount);
                        self.players[player_idx].add_money(actual_amount);
                        println!("💰 {} a plătit M{} (cotă parte)!", self.players[idx].name, actual_amount);
                        self.history.push(GameAction::Payment {
                            from: Some(idx),
                            to: Some(player_idx),
                            amount: actual_amount,
                            initiator: None
                        });
                        self.log_action(Some(idx), format!("{} paid M{} to {}", self.players[idx].name, actual_amount, self.players[player_idx].name));
                    }
                }
            }
            HereAndNowCardAction::InterceptPurchase => {
                let purchase_data = self.last_purchase.clone();
                if purchase_data.is_none() {
                    return Err("Nicio achiziție recentă de interceptat!".to_string());
                }
                
                if let Some(record) = purchase_data {
                    if record.buyer_idx == player_idx {
                        return Err("Nu poți intercepta propria cumpărare!".to_string());
                    }
                    
                    let price = record.price;
                    let old_buyer_idx = record.buyer_idx;
                    let stamp_name = record.name.clone();
                    // Verificăm dacă eu am bani să-l cumpăr
                    if self.players[player_idx].pay_money(price) {
                        // Refundăm vechiul cumpărător
                        self.players[old_buyer_idx].add_money(price);
                        
                        // Mutăm ștampila de la el la mine
                        if let Some(pos) = self.players[old_buyer_idx].passport.find_stamp_index(&stamp_name) {
                            if let Some(stamp) = self.players[old_buyer_idx].passport.remove_stamp_at(pos) {
                                 let s_id = format!("{}", stamp.destination_id.unwrap_or(0));
                                 let is_fc = stamp.destination_id.is_none();

                                 self.add_stamp_with_checks(player_idx, stamp);
                                 println!("🎯 INTERCEPT! {} a preluat '{}' de la {} pentru M{}!", 
                                          self.players[player_idx].name, stamp_name, self.players[old_buyer_idx].name, price);
                                
                                 self.history.push(GameAction::Payment { from: Some(player_idx), to: None, amount: price, initiator: None });
                                 self.history.push(GameAction::Payment { from: None, to: Some(old_buyer_idx), amount: price, initiator: None });

                                 self.history.push(GameAction::StampTransfer { 
                                     from: Some(old_buyer_idx), 
                                     to: Some(player_idx), 
                                     stamp_name: stamp_name.clone(), 
                                     stamp_id: s_id,
                                     is_first_class: is_fc,
                                     initiator: Some(player_idx)
                                 });

                                 self.log_action(Some(player_idx), format!("{} intercepted {} from {}", self.players[player_idx].name, stamp_name, self.players[old_buyer_idx].name));

                                self.last_purchase = None;
                                return Ok(());
                            }
                        }
                        self.players[player_idx].add_money(price);
                        self.players[old_buyer_idx].pay_money(price);
                        return Err("Proprietatea nu mai este în pașaportul cumpărătorului original.".to_string());
                    } else {
                        return Err("Nu ai suficienți bani pentru interceptare!".to_string());
                    }
                } else {
                    return Err("Nicio achiziție recentă de interceptat.".to_string());
                }
            },
            HereAndNowCardAction::SayNo => {
                // LOGICĂ UNDO / REVERT
                let found_revert = self.history.iter().enumerate().rev().take(3)
                    .find_map(|(i, action)| {
                        match action {
            GameAction::Payment { from, to, amount, initiator } if *from == Some(player_idx) && to.is_some() && *initiator != Some(player_idx) => {
                                Some((i, action.clone()))
                            },
                            GameAction::StampTransfer { from, initiator, .. } if *from == Some(player_idx) && *initiator != Some(player_idx) => {
                                Some((i, action.clone()))
                            },
                            _ => None
                        }
                    });

                if let Some((idx_in_history, action)) = found_revert {
                    self.players[player_idx].say_no_cards += 1;
                    match action {
                        GameAction::Payment { to, amount, .. } => {
                            let receiver_idx = to.unwrap();
                            if self.players[receiver_idx].pay_money(amount) {
                                self.players[player_idx].add_money(amount);
                                println!("🛑 SAY NO! Plata de M{} a fost anulată. Banii s-au întors.", amount);
                                self.log_action(Some(player_idx), format!("Just Say No! Payment of M{} cancelled", amount));
                            }
                        },
                        GameAction::StampTransfer { to, stamp_name, .. } => {
                            if let Some(target_idx) = to {
                                // ✅ Verificăm dacă a fost un schimb (Swap)
                                let mut swap_info = None;
                                if idx_in_history > 0 {
                                    if let Some(prev_action) = self.history.get(idx_in_history - 1).cloned() {
                                        if let GameAction::StampTransfer { from: Some(f_idx), to: Some(t_idx), stamp_name: other_stamp_name, .. } = prev_action {
                                            if f_idx == target_idx && t_idx == player_idx {
                                                swap_info = Some((other_stamp_name, target_idx));
                                            }
                                        }
                                    }
                                }

                                if let Some((other_stamp_name, t_idx)) = swap_info {
                                    // 🔄 REVERSAL ATOMIC PENTRU SCHIMB (SWAP)
                                    // Găsim pozițiile ambelor ștampile
                                    let pos_target = self.players[t_idx].passport.find_stamp_index(&stamp_name);
                                    let pos_me = self.players[player_idx].passport.find_stamp_index(&other_stamp_name);

                                    if let (Some(pa), Some(pb)) = (pos_target, pos_me) {
                                        // Scoatem ambele ștampile înainte de a le adăuga la loc
                                        let s_to_me = self.players[t_idx].passport.remove_stamp_at(pa);
                                        let s_to_target = self.players[player_idx].passport.remove_stamp_at(pb);

                                        if let (Some(sa), Some(sb)) = (s_to_me, s_to_target) {
                                            self.add_stamp_with_checks(player_idx, sa);
                                            self.add_stamp_with_checks(t_idx, sb);
                                            println!("🛑 Schimb anulat ATOMIC! {} și {} și-au recuperat ștampilele în coloanele corecte.", self.players[player_idx].name, self.players[t_idx].name);
                                            self.log_action(Some(player_idx), format!("Just Say No! Swap {} <-> {} reversed atomically", stamp_name, other_stamp_name));
                                        }
                                    }
                                } else {
                                    // 🔄 REVERSAL PENTRU TRANSFER SIMPLU
                                    if let Some(pos) = self.players[target_idx].passport.find_stamp_index(&stamp_name) {
                                        if let Some(s) = self.players[target_idx].passport.remove_stamp_at(pos) {
                                            self.add_stamp_with_checks(player_idx, s);
                                            println!("🛑 SAY NO! Transferul ștampilei {} a fost anulat.", stamp_name);
                                            self.log_action(Some(player_idx), format!("Just Say No! Stamp {} recovered", stamp_name));
                                        }
                                    }
                                }
                            } else {
                                // A fost o eliminare (TakeAllLastStamps) -> o punem la loc
                                // Trebuie să găsim obiectul Stamp original. 
                                let stamp_obj = if stamp_name == "First Class" {
                                    Stamp::first_class()
                                } else {
                                    self.board.find_destination_by_name(&stamp_name)
                                        .map(|d| Stamp::from_destination(d))
                                        .unwrap_or(Stamp::first_class()) // Fallback
                                };
                                self.add_stamp_with_checks(player_idx, stamp_obj);
                                println!("🛑 SAY NO! Ștampila {} a fost recuperată!", stamp_name);
                                self.log_action(Some(player_idx), format!("Just Say No! Stamp {} recovered", stamp_name));
                            }
                        },
                        _ => {}
                    }
                } else {
                    return Err("Nu există nicio acțiune recentă împotriva ta pe care să o poți anula!".to_string());
                }
            },
            HereAndNowCardAction::DiscountPurchase => {
                self.players[player_idx].discount_purchase_ready = true;
                
                // ✅ REACTIV: Dacă suntem deja în curs de a cumpăra și prețul e mai mare de 100, îl reducem
                if let GameStep::WaitingForPurchaseDecision { dest_id, price, buyer_idx } = &self.step {
                    let (dest_id, price, buyer_idx) = (*dest_id, *price, *buyer_idx);
                    if buyer_idx == player_idx && price > 100 {
                         println!("💸 DISCOUNT REACTIV! Prețul de cumpărare a scăzut la M100.");
                         self.step = GameStep::WaitingForPurchaseDecision {
                             dest_id,
                             price: 100,
                             buyer_idx
                         };
                         self.players[player_idx].discount_purchase_ready = false;
                         return Ok(()); // Ieșim pentru a evita logică de refund de mai jos
                    }
                }

                let refund_data = self.history.iter().rev().take(10).find_map(|action| {
                    if let GameAction::Payment { from, to: None, amount, .. } = action {
                        if *from == Some(player_idx) && *amount > 100 {
                            return Some(*amount - 100);
                        }
                    }
                    None
                });

                if let Some(refund) = refund_data {
                    self.players[player_idx].add_money(refund);
                    println!("💰 DISCOUNT REACTIV! Ai primit înapoi M{} din ultima plată.", refund);
                    self.players[player_idx].discount_purchase_ready = false; 
                }
            },
            HereAndNowCardAction::CollectTax => {
                // ✅ Verificăm dacă tocmai am plătit o taxă unui adversar (în ultimele 10 acțiuni)
                let found_revert = self.history.iter().rev().take(10).find_map(|action| {
                    if let GameAction::Payment { from, to: Some(receiver_idx), amount, .. } = action {
                        if *from == Some(player_idx) {
                             return Some((*receiver_idx, *amount));
                        }
                    }
                    None
                });

                if found_revert.is_none() {
                    return Err("Nu poți folosi acest card pentru că nu ai plătit nicio taxă recent!".to_string());
                }

                self.players[player_idx].collect_tax_ready = true;
                
                if let Some((receiver_idx, amount)) = found_revert {
                    // Verificăm dacă celălalt are bani să ne dea înapoi + să ne plătească taxa
                    // (Pentru simplitate, măcar să dea înapoi ce a primit)
                    if self.players[receiver_idx].pay_money(amount) {
                        self.players[player_idx].add_money(amount);
                        
                        // Colectăm taxa de la el (dacă mai are bani)
                        if self.players[receiver_idx].pay_money(amount) {
                             self.players[player_idx].add_money(amount);
                             println!("🧾 COLLECT TAX REACTIV! Plata de M{} a fost anulată și ai colectat M{} de la {}!", amount, amount, self.players[receiver_idx].name);
                             
                             self.history.push(GameAction::Payment {
                                 from: Some(receiver_idx),
                                 to: Some(player_idx),
                                 amount: amount * 2,
                                 initiator: None
                             });
                        } else {
                             println!("🧾 COLLECT TAX REACTIV! Plata de M{} a fost anulată, dar {} nu mai are bani să-ți plătească taxa!", amount, self.players[receiver_idx].name);
                        }
                        self.players[player_idx].collect_tax_ready = false;
                    }
                }
            },
            HereAndNowCardAction::StealFirstClass => {
                let (target_idx, amount) = if let Some(record) = &self.last_purchase {
                    if record.is_first_class && record.buyer_idx != player_idx {
                        (record.buyer_idx, record.price)
                    } else {
                        return Err("Nu poți folosi acest card acum. Nicio ștampilă First Class nu a fost cumpărată recent de alți jucători.".to_string());
                    }
                } else {
                    return Err("Nu poți folosi acest card acum.".to_string());
                };

                // Perform the steal
                if let Some(stamp) = self.players[target_idx].passport.remove_last_stamp() {
                    println!("✈️ STEAL FIRST CLASS REACTIV! Ai furat ștampila de la {}.", self.players[target_idx].name);
                    
                    // Refund the original buyer
                    self.players[target_idx].add_money(amount);
                    println!("💰 Jucătorul {} a fost despăgubit cu M{}.", self.players[target_idx].name, amount);

                    let s_name = stamp.name.clone();
                    let s_id = stamp.destination_id.map(|id| format!("{}", id)).unwrap_or_else(|| "first_class".to_string());
                    let is_fc = true; // By definition for this card

                    // Record the transfer in history
                    self.history.push(GameAction::StampTransfer {
                        from: Some(target_idx),
                        to: Some(player_idx),
                        stamp_name: s_name,
                        stamp_id: s_id,
                        is_first_class: is_fc,
                        initiator: Some(player_idx),
                    });

                    // Add stamp to the stealer
                    self.add_stamp_with_checks(player_idx, stamp);
                    self.log_action(Some(player_idx), format!("Steal First Class used against {}", self.players[target_idx].name));
                } else {
                    return Err("Jucătorul țintă nu mai are ștampila!".to_string());
                }
            }
        }
        Ok(())
    }

    fn end_turn(&mut self) {
        // If turn ending is triggered, we move next regardless of old doubles
        // unless they are currently rolling doubles and are NOT in jail.
        let in_jail = self.players[self.current_player_idx].in_jail;
        let consecutive_doubles = self.players[self.current_player_idx].consecutive_doubles;
        
        if consecutive_doubles == 0 || in_jail {
             // Logică incrementare tură globală
            if self.current_player_idx == self.players.len() - 1 {
                self.turn_number += 1;
                println!("\n{}", "=".repeat(60).on_bright_blue().white());
                println!("{}  ÎNCEPE TURA GLOBALĂ #{}  {}", "🌍".yellow(), self.turn_number, "🌍".yellow());
            }
            
            // Reset "ready" flags for the player who just finished their turn
            let p = &mut self.players[self.current_player_idx];
            p.collect_tax_ready = false;
            p.discount_purchase_ready = false;
            p.intercept_purchase_ready = false;
            p.steal_first_class_ready = false;

            self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
            
            if self.players[self.current_player_idx].in_jail {
                self.step = GameStep::WaitingForJailDecision;
            } else {
                self.step = GameStep::WaitingForRoll;
            }
            
            println!("\n{} {}", "Rândul jucătorului:".green().bold(), self.players[self.current_player_idx].name.yellow().bold());
        } 
        // Dacă are duble consecutive > 0 și nu e în închisoare, RĂMÂNE pe același player (nu apelăm end_turn în roll_dice pt duble)
    }


    // --- Helper intern redenumit pentru claritate ---
    fn roll_dice_internal() -> DiceResult {
        let mut rng = rand::thread_rng();
        let d1 = rng.gen_range(1..=6);
        let d2 = rng.gen_range(1..=6);

        // Dacă primul zar e 1, e Afacere Forțată (Mr. Monopoly)
        if d1 == 1 {
            return DiceResult::BusinessDeal(d2);
        }

        if d1 == d2 { 
            DiceResult::Double(d1) 
        } else { 
            DiceResult::Normal(d1, d2) 
        }
    }


    fn display_dice_result(&self, result: &DiceResult) {
        match result {
            DiceResult::Normal(d1, d2) => {
                println!("🎲 {} + {} = {}", d1, d2, d1 + d2);
            }
            DiceResult::Double(value) => {
                println!("🎲🎲 DUBLĂ! {} + {} = {}", value, value, value * 2);
            }
            DiceResult::BusinessDeal(val) => {
                println!("💼 AFACERE FORȚATĂ! (1 + {})", val);
            }
        }
    }

    fn move_player(&mut self, steps: i32) {
        let board_size = self.board.total_spaces();
        let player_idx = self.current_player_idx;
        let old_position = self.players[player_idx].position;

        let passed_start = self.players[player_idx].move_by(steps, board_size);

        println!("\n{} se mută de la poziția {} la poziția {}",
                 self.players[player_idx].name.yellow(),
                 old_position,
                 self.players[player_idx].position);

        if passed_start {
            self.players[player_idx].add_money(200);
            println!("{}", "✨ Ai trecut pe la START! Primești M200.".green());
            
            // Record partial move or just payment? 
            // START money is a transaction from Bank
            self.history.push(GameAction::Payment { 
                from: None, 
                to: Some(player_idx), 
                amount: 200,
                initiator: None 
            });
            let name = self.players[player_idx].name.clone();
            self.log_action(Some(player_idx), format!("{} landed on Start and received $200", name));
        }
        
        self.history.push(GameAction::Move {
            player_idx,
            from: old_position as u8,
            to: self.players[player_idx].position as u8
        });
    }

    /// Verifică dacă jucătorul curent a câștigat și termină jocul dacă da
    /// Returnează true dacă jocul s-a terminat
    fn check_and_handle_win(&mut self, player_idx: usize) -> bool {
        if self.players[player_idx].passport.is_full() {
            println!(
                "\n🎉 PAȘAPORT DEPĂȘIT – {} A CÂȘTIGAT JOCUL!",
                self.players[player_idx].name
            );
            self.players[player_idx].display_status();
            self.game_over = true;
            return true;
        }
        false
    }

    /// Folosește un cartonaș Șansă din mână (ex: Ieșire din închisoare)
    pub fn use_chance_card(&mut self, player_idx: usize, card_id: String) -> Result<TurnResult, String> {
        let card_idx = self.players[player_idx].chance_cards.iter().position(|c| c.id == card_id)
            .ok_or_else(|| "Nu deții acest cartonaș!".to_string())?;

        let card = self.players[player_idx].chance_cards.remove(card_idx);
        println!("🎲 {} folosește cartonașul Șansă: {}", self.players[player_idx].name, card.description);

        self.execute_chance_action(player_idx, card.action.clone())?;
        self.chance_deck.discard(card);

        Ok(TurnResult {
            die1: 0,
            die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[player_idx].position as u8,
            went_to_jail: false,
            turn_ends: false, 
            current_player_index: self.current_player_idx as u8,
        })
    }

    fn execute_chance_action(&mut self, idx: usize, action: ChanceCardAction) -> Result<(), String> {
        match action {
            ChanceCardAction::CollectMoney(amount) => {
                self.players[idx].add_money(amount as i32);
                self.history.push(GameAction::Payment { 
                    from: None, 
                    to: Some(idx), 
                    amount: amount as i32,
                    initiator: None
                });
                println!("{}", format!("Primești M{}", amount).green());
            }

            ChanceCardAction::PayHospital => {
                let amount = 200;
                if !self.players[idx].pay_money(amount) {
                    println!("{}", "Nu ai suficienți bani!".red());
                    self.handle_bankruptcy(idx, None);
                } else {
                    println!("{}", format!("Plătești M{}", amount).yellow());
                    self.history.push(GameAction::Payment { 
                        from: Some(idx), 
                        to: None, 
                        amount,
                        initiator: None
                    });
                }
            }

            ChanceCardAction::FirstClassBonus => {
                let count = self.players[idx]
                    .passport
                    .all_stamps()
                    .iter()
                    .filter(|s| s.name.to_uppercase().contains("FIRST CLASS"))
                    .count() as u32;

                let gain = 40 * count;
                self.players[idx].add_money(gain as i32);
                println!("{}", format!("Ai {} ștampile First Class => primești M{}", count, gain).bright_green());
                self.history.push(GameAction::Payment { 
                    from: None, 
                    to: Some(idx), 
                    amount: gain as i32,
                    initiator: None 
                });
            }

            ChanceCardAction::CollectFromEachPlayer(amount) => {
                println!("{}", format!("Colectezi M{} de la fiecare jucător", amount).green());
                for i in 0..self.players.len() {
                    if i != idx {
                        if self.players[i].pay_money(amount as i32) {
                            self.players[idx].add_money(amount as i32);
                            self.history.push(GameAction::Payment { 
                                from: Some(i), 
                                to: Some(idx), 
                                amount: amount as i32,
                                initiator: None
                            });
                        } else {
                            println!("{}", format!("{} nu are bani suficienți!", self.players[i].name).yellow());
                        }
                    }
                }
            }

            ChanceCardAction::MoveSteps(steps) => {
                self.move_player(steps);
                self.handle_landing(idx);
            }

            ChanceCardAction::AdvanceToStart => {
                let old_pos = self.players[idx].position;
                self.players[idx].move_to(0);
                self.players[idx].add_money(200);
                println!("{}", "Avansezi la START și colectezi M200.".bright_green());
                
                self.history.push(GameAction::Move {
                    player_idx: idx,
                    from: old_pos as u8,
                    to: 0
                });
                self.history.push(GameAction::Payment {
                    from: None,
                    to: Some(idx),
                    amount: 200,
                    initiator: None
                });
            }

            ChanceCardAction::GoToJail => {
                self.players[idx].send_to_jail();
                self.history.push(GameAction::GoToJail { player_idx: idx });
                println!("{}", "Mergi direct la închisoare! (Fără START)".red());
            }

            ChanceCardAction::GetOutOfJailFree => {
                if !self.players[idx].in_jail {
                    return Err("Nu ești în închisoare!".to_string());
                }
                self.players[idx].release_from_jail();
                println!("🔓 Ai ieșit gratuit din închisoare!");
                let name = self.players[idx].name.clone();
                self.log_action(Some(idx), format!("{} used chance card: Get Out Of Jail Free", name));
            }

            ChanceCardAction::RerollOneDie => {
                println!("{}" , "Ai primit o carte de șansă: Aruncă din nou un zar!".cyan());
                self.step = GameStep::WaitingForRerollDice { player_idx: idx };
                self.pending_reroll = Some(PendingReroll { player_idx: idx });
            }

            ChanceCardAction::DiceChallenge => {
                 self.step = GameStep::WaitingForTargetSelection {
                    action: "DiceDuel".to_string(),
                    card_id: None, // Chance cards don't have IDs in this context yet, or we need to pass it
                    selector_idx: idx
                 };
            }
            ChanceCardAction::SwapTwoPlayersStamps => {
                 // SNEAKY SWAP: Găsim toți jucătorii care au cel puțin o ștampilă
                 let mut eligible: Vec<usize> = self.players.iter().enumerate()
                     .filter(|(_, p)| p.passport.stamp_count() > 0)
                     .map(|(i, _)| i)
                     .collect();

                 if eligible.len() >= 2 {
                     let mut rng = rand::thread_rng();
                     let idx1 = rng.gen_range(0..eligible.len());
                     let p1_idx = eligible.remove(idx1);
                     
                     let idx2 = rng.gen_range(0..eligible.len());
                     let p2_idx = eligible.remove(idx2);

                     if let Some(s1) = self.players[p1_idx].passport.remove_last_stamp() {
                         if let Some(s2) = self.players[p2_idx].passport.remove_last_stamp() {
                             println!("♻️ SNEAKY SWAP! {} și {} schimbă ultimele ștampile ('{}' ↔ '{}')!", 
                                      self.players[p1_idx].name, self.players[p2_idx].name, s1.name, s2.name);
                             self.history.push(GameAction::StampTransfer {
                                 from: Some(p1_idx),
                                 to: Some(p2_idx),
                                 stamp_name: s1.name.clone(),
                                 stamp_id: format!("{}", s1.destination_id.unwrap_or(0)),
                                 is_first_class: s1.destination_id.is_none(),
                                 initiator: None,
                             });
                             self.history.push(GameAction::StampTransfer {
                                 from: Some(p2_idx),
                                 to: Some(p1_idx),
                                 stamp_name: s2.name.clone(),
                                 stamp_id: format!("{}", s2.destination_id.unwrap_or(0)),
                                 is_first_class: s2.destination_id.is_none(),
                                 initiator: None,
                             });

                             self.log_action(None, format!("♻️ Sneaky Swap: {} and {} swapped stamps!", self.players[p1_idx].name, self.players[p2_idx].name));

                             self.add_stamp_with_checks(p1_idx, s2);
                             self.add_stamp_with_checks(p2_idx, s1);

                             // Verificăm câștig
                             if self.check_and_handle_win(p1_idx) { return Ok(()); }
                             if self.check_and_handle_win(p2_idx) { return Ok(()); }
                         }
                     }
                 } else {
                     println!("Nu sunt suficienți jucători cu ștampile pentru Sneaky Swap!");
                 }
            }
            ChanceCardAction::StealStampAndPay => {
                 let has_targets = self.players.iter().enumerate()
                    .any(|(i, p)| i != idx && p.passport.stamp_count() > 0);
                 
                 if has_targets {
                     self.step = GameStep::WaitingForTargetSelection {
                         action: "StealStampAndPay".to_string(),
                         card_id: None,
                         selector_idx: idx
                     };
                 } else {
                     println!("Niciun adversar nu are ștampile de furat!");
                 }
            }
        }
        Ok(())
    }
    fn handle_landing(&mut self, player_idx: usize) {
        let position = self.players[player_idx].position;
        let space = self.board.get_space(position).clone();

        println!("\n{} {}", "📍 Ai ajuns pe:".cyan(), self.get_space_name(&space).yellow().bold());

        match space {
            Space::Start => {
                println!("{}", "🏁 START - Relaxează-te!".green());
            }
            Space::Destination(dest) => {
                self.handle_destination(dest, player_idx);
            }
            Space::FirstClass => {
                self.handle_first_class(player_idx);
            }
            Space::Airport => {
                self.handle_airport(player_idx);
            }
            Space::HereAndNow => {
                self.handle_here_and_now(player_idx);
            }
            Space::Chance => {
                self.handle_chance(player_idx);
            }
            Space::FreeParking => {
                println!("{}", "🅿️ Parcare Gratuită - Ia o pauză!".green());
            }
            Space::GoToJail => {
                println!("{}", "👮 Mergi direct la ÎNCHISOARE!".red());
                let name = self.players[player_idx].name.clone();
                let player = &mut self.players[player_idx];
                player.send_to_jail();
                self.history.push(GameAction::GoToJail { player_idx });
                self.log_action(Some(player_idx), format!("{} got into prison", name));
            }
            Space::JustVisiting => {
                println!("{}", "👀 Doar în vizită la închisoare.".cyan());
            }
        }
    }

    fn handle_destination(&mut self, dest: Destination, player_idx: usize) {
        println!("🌍 {} - Preț: M{}, Taxă: M{}",
                 dest.name, dest.price, dest.tourist_tax);

        // Verificăm cine deține această destinație
        let owner_idx = self.find_destination_owner(dest.id);

        if let Some(owner_idx) = owner_idx {
            if owner_idx == player_idx {
                println!("{}", "Aceasta este destinația ta! Nu plătești nimic.".green());
            } else {
                // Trebuie să plătim taxă
                let mut tax = dest.tourist_tax;

                // Verificăm dacă proprietarul are setul complet de culoare
                if let Some(color_set) = self.board.color_sets.get(&dest.color) {
                    if self.players[owner_idx].has_color_set(color_set) {
                        tax *= 2;
                        println!("{}", "⚠️ Proprietarul are setul complet! Taxa este DUBLĂ!".red());
                    }
                }

                // Verificăm dacă proprietarul are bonus dublu activ
                if self.players[owner_idx].double_rent_active {
                    tax *= 2;
                    println!("{}", "⚠️ Proprietarul are bonus Here&Now activ! Taxa este DUBLĂ!".red());
                    self.players[owner_idx].double_rent_active = false;
                }

                // ✅ COLLECT TAX: jucătorul curent colectează taxa în loc să o plătească
                if self.players[player_idx].collect_tax_ready {
                    println!("{}", "🧾 COLLECT TAX ACTIV! Colectezi taxa în loc să o plătești!".bright_green());

                    if self.players[owner_idx].pay_money(tax as i32) {
                        self.players[player_idx].add_money(tax as i32);
                        self.history.push(GameAction::Payment {
                            from: Some(owner_idx),
                            to: Some(player_idx),
                            amount: tax as i32,
                            initiator: None
                        });
                        println!("{}", format!("{} îți plătește M{}", self.players[owner_idx].name, tax).green());
                    } else {
                        println!("{}", format!("{} nu are bani!", self.players[owner_idx].name).yellow());
                    }

                    self.players[player_idx].collect_tax_ready = false;

                    // Găsim și ștergem cardul din mână
                    if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::CollectTax)) {
                        let card = self.players[player_idx].here_and_now_cards.remove(pos);
                        self.here_and_now_deck.discard(card);
                    }

                    return;
                }

                println!("{}", format!("Plătești M{} către {}", tax, self.players[owner_idx].name).yellow());

                if self.players[player_idx].pay_money(tax as i32) {
                    self.players[owner_idx].add_money(tax as i32);
                    self.history.push(GameAction::Payment {
                        from: Some(player_idx),
                        to: Some(owner_idx),
                        amount: tax as i32,
                        initiator: None
                    });
                    let payer = self.players[player_idx].name.clone();
                    let receiver = self.players[owner_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} paid ${} to {}", payer, tax, receiver));
                    println!("{}", "Taxă plătită!".green());
                } else {
                    println!("{}", "Nu ai suficienți bani!".red());
                    self.handle_bankruptcy(player_idx, Some(owner_idx));
                }
            }
        } else {
            // Destinația nu este deținută de nimeni
            println!("\n{}", "Această destinație nu este deținută de nimeni!".cyan());

            let mut final_price = dest.price;
            let buyer_idx = player_idx;

            // ✅ DISCOUNT PURCHASE: plătești doar M100
            if self.players[player_idx].discount_purchase_ready {
                final_price = 100;
                println!("{}", "💸 DISCOUNT PURCHASE ACTIV! Plătești doar M100!".bright_green());

                self.players[player_idx].discount_purchase_ready = false;

                // Găsim și ștergem cardul din mână
                if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::DiscountPurchase)) {
                    let card = self.players[player_idx].here_and_now_cards.remove(pos);
                    self.here_and_now_deck.discard(card);
                }
            }

            println!("Vrei să o cumperi pentru M{}? (y/n)", final_price);

            // Verificăm dacă jucătorul are bani - dacă da, așteptăm decizia
            if self.players[buyer_idx].money >= final_price as i32 {
                // Setăm starea pentru a aștepta decizia jucătorului
                self.step = GameStep::WaitingForPurchaseDecision { 
                    dest_id: dest.id, 
                    price: final_price as i32,
                    buyer_idx: buyer_idx
                };
                println!("Așteptăm decizia jucătorului {}...", self.players[buyer_idx].name);
            } else {
                println!("Nu ai destui bani pentru a cumpăra această proprietate. Se intră automat la LICITAȚIE!");
                println!("{}", "🔨 AUCTION! Bidding starts at M20".bright_yellow());
                self.step = GameStep::WaitingForAuction {
                    dest_id: dest.id,
                    current_bid: 20,
                    highest_bidder: None,
                };
            }
        }
    }

    fn handle_first_class(&mut self, player_idx: usize) {
            println!("✈️ Clasa Întâi disponibilă pentru M100");

            if self.first_class_stamps_available == 0 {
                println!("Nu mai sunt stampile Clasa Întâi.");
                return;
            }

            // Verificăm dacă jucătorul are bani pentru a cumpăra
            if self.players[player_idx].money >= 100 {
                // Setăm starea pentru a aștepta decizia jucătorului ORIGINAL
                self.step = GameStep::WaitingForFirstClassDecision { buyer_idx: player_idx };
                println!("Așteptăm decizia jucătorului {} pentru First Class...", self.players[player_idx].name);
            } else {
                println!("Nu ai destui bani pentru a cumpăra First Class.");
            }
        }

    fn handle_airport(&mut self, player_idx: usize) {
        println!("✈️ AEROPORT - Poți zbura oriunde pentru M100");
        if self.players[player_idx].money >= 100 {
            self.step = GameStep::WaitingForAirportDecision { buyer_idx: player_idx };
            println!("Așteptăm decizia jucătorului pentru aeroport...");
        } else {
            println!("Nu ai destui bani pentru zbor.");
        }
    }

    fn handle_here_and_now(&mut self, player_idx: usize) {
        println!("🎴 Tragi un cartonaș Here & Now...");

        let card = self.here_and_now_deck.draw();
        println!("{}", format!("📜 {}", card.description).bright_cyan());

        let h_name = self.players[player_idx].name.clone();
        self.log_action(Some(player_idx), format!("{} got a treasure card", h_name));

        // îl punem în mână mereu
        self.players[player_idx].here_and_now_cards.push(card.clone());

        // pentru anumite cărți, afișăm doar ce fac (nu se mai armează automat)
        match card.action {
            HereAndNowCardAction::SayNo => {
                println!("{}", "✅ Ai primit 'Spune nu!' (folosește-l din mână pentru a anula o acțiune).".bright_green());
            }
            HereAndNowCardAction::InterceptPurchase => {
                println!("{}", "✅ Intercept Purchase păstrat în mână. Îl poți folosi pentru a fura ultima proprietate cumpărată!".bright_green());
            }
            HereAndNowCardAction::DiscountPurchase => {
                println!("{}", "✅ Discount Purchase păstrat în mână. Folosește-l înainte de a cumpăra pentru a plăti doar M100!".bright_green());
            }
            HereAndNowCardAction::CollectTax => {
                println!("{}", "✅ Collect Tax păstrat în mână. Folosește-l când trebuie să plătești o taxă pentru a o colecta tu!".bright_green());
            }
            HereAndNowCardAction::StealFirstClass => {
                println!("{}", "✅ Steal First Class păstrat în mână. Folosește-l când altcineva primește First Class pentru a-l fura!".bright_green());
            }
            _ => {
                println!("{}", "✅ Cartonaș păstrat în mână (folosibil oricând).".bright_green());
            }
        }
    }

    fn handle_chance(&mut self, player_idx: usize) {
        println!("🎲 Tragi un cartonaș Șansă...");

        let card = self.chance_deck.draw();
        println!("{}", format!("📜 {}", card.description).bright_yellow());

        let c_name = self.players[player_idx].name.clone();
        let c_desc = card.description.clone();
        self.log_action(Some(player_idx), format!("{} got a chance card: {}", c_name, c_desc));

        if card.can_keep {
            self.players[player_idx].chance_cards.push(card);
            println!("{}", "✅ Cartonaș păstrat în mână (folosibil oricând).".bright_green());
        } else {
            let action = card.action.clone();
            let _ = self.execute_chance_action(player_idx, action);
            self.chance_deck.discard(card);
        }
    }

    fn handle_business_deal(&mut self, player_idx: usize, target_idx: Option<usize>) {
        // Schimbăm ultima stampilă cu a unui adversar

        if self.players[player_idx].passport.stamp_count() == 0 {
            println!("{}", "Nu ai stampile de schimbat!".yellow());
            return;
        }

        // Dacă nu avem target_idx (vechea logică sau nespecificat), găsim primul adversar eligibil
        let opp_idx = if let Some(idx) = target_idx {
            if idx == player_idx || self.players[idx].passport.stamp_count() == 0 {
                None
            } else {
                Some(idx)
            }
        } else {
            self.players.iter()
                .enumerate()
                .find(|(i, p)| *i != player_idx && p.passport.stamp_count() > 0)
                .map(|(i, _)| i)
        };

        if let Some(opp_idx) = opp_idx {
            if let Some(my_stamp) = self.players[player_idx].passport.remove_last_stamp() {
                if let Some(opp_stamp) = self.players[opp_idx].passport.remove_last_stamp() {
                    println!("{}", format!("Schimbi '{}' cu '{}' de la {}",
                                           my_stamp.name, opp_stamp.name, self.players[opp_idx].name).bright_magenta());

                    self.history.push(GameAction::StampTransfer {
                        from: Some(player_idx),
                        to: Some(opp_idx),
                        stamp_name: my_stamp.name.clone(),
                        stamp_id: format!("{}", my_stamp.destination_id.unwrap_or(0)),
                        is_first_class: my_stamp.destination_id.is_none(),
                        initiator: Some(player_idx),
                    });

                    self.history.push(GameAction::StampTransfer {
                        from: Some(opp_idx),
                        to: Some(player_idx),
                        stamp_name: opp_stamp.name.clone(),
                        stamp_id: format!("{}", opp_stamp.destination_id.unwrap_or(0)),
                        is_first_class: opp_stamp.destination_id.is_none(),
                        initiator: Some(player_idx),
                    });

                    let my_s_name = my_stamp.name.clone();
                    let opp_s_name = opp_stamp.name.clone();

                    // Log the swap BEFORE moving the stamps and checking win
                    let my_name = self.players[player_idx].name.clone();
                    let opp_name = self.players[opp_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} swapped {} for {} with {}", my_name, my_s_name, opp_s_name, opp_name));

                    self.add_stamp_with_checks(player_idx, opp_stamp);
                    self.add_stamp_with_checks(opp_idx, my_stamp);

                    // Verifică câștig pentru ambii jucători
                    if self.check_and_handle_win(player_idx) {
                        return;
                    }
                    if self.check_and_handle_win(opp_idx) {
                        return;
                    }
                }
            }
        } else {
            println!("{}", "Niciun adversar nu are stampile de schimbat!".yellow());
        }
    }

    fn handle_stamp_swap(&mut self, player_idx: usize, target_idx: Option<usize>) {
         // Logică similară cu handle_business_deal, dar schimb de ștampile (SwapStamps)
         println!("♻️ Începe schimbul de ștampile...");
         
         if self.players[player_idx].passport.stamp_count() == 0 {
             println!("Nu ai stampile de schimbat!");
             return;
         }

          let opp_idx = if let Some(idx) = target_idx {
                if idx == player_idx || self.players[idx].passport.stamp_count() == 0 {
                    None
                } else {
                    Some(idx)
                }
            } else {
                self.players.iter()
                    .enumerate()
                    .find(|(i, p)| *i != player_idx && p.passport.stamp_count() > 0)
                    .map(|(i, _)| i)
            };

            if let Some(opp_idx) = opp_idx {
                let s1 = self.players[player_idx].passport.remove_last_stamp();
                let s2 = self.players[opp_idx].passport.remove_last_stamp();
                
                if let (Some(stamp1), Some(stamp2)) = (s1, s2) {
                     println!("♻️ Schimb de ștampile între {} și {}!", self.players[player_idx].name, self.players[opp_idx].name);
                     
                     self.history.push(GameAction::StampTransfer {
                         from: Some(player_idx),
                         to: Some(opp_idx),
                         stamp_name: stamp1.name.clone(),
                         stamp_id: format!("{}", stamp1.destination_id.unwrap_or(0)),
                         is_first_class: stamp1.destination_id.is_none(),
                         initiator: Some(player_idx),
                     });

                     self.history.push(GameAction::StampTransfer {
                         from: Some(opp_idx),
                         to: Some(player_idx),
                         stamp_name: stamp2.name.clone(),
                         stamp_id: format!("{}", stamp2.destination_id.unwrap_or(0)),
                         is_first_class: stamp2.destination_id.is_none(),
                         initiator: Some(player_idx),
                     });
                     
                     self.add_stamp_with_checks(player_idx, stamp2.clone());
                     self.add_stamp_with_checks(opp_idx, stamp1.clone());
                     
                     // Verifică câștig
                     if self.check_and_handle_win(player_idx) { return; }
                     if self.check_and_handle_win(opp_idx) { return; }

                     // Log the swap
                     let p_name = self.players[player_idx].name.clone();
                     let o_name = self.players[opp_idx].name.clone();
                     self.log_action(Some(player_idx), format!("{} swapped {} for {} with {}", p_name, stamp1.name, stamp2.name, o_name));
                }
            } else {
                 println!("Niciun adversar nu are stampile!");
            }
    }

    fn handle_steal_stamp_and_pay(&mut self, player_idx: usize, target_idx: usize) {
        if let Some(stamp) = self.players[target_idx].passport.remove_last_stamp() {
            // Găsim prețul real al proprietății
            let price = if stamp.destination_id.is_none() {
                100 // First Class are preț fix de 100
            } else {
                self.board.find_destination_by_name(&stamp.name)
                    .map(|d| d.price)
                    .unwrap_or(200) // Fallback generic
            };

            if self.players[player_idx].pay_money(price as i32) {
                self.players[target_idx].add_money(price as i32);
                self.history.push(GameAction::Payment {
                    from: Some(player_idx),
                    to: Some(target_idx),
                    amount: price as i32,
                    initiator: Some(player_idx)
                });
                
                let stamp_name = stamp.name.clone();
                let is_first_class = stamp.destination_id.is_none();
                
                self.history.push(GameAction::StampTransfer {
                    from: Some(target_idx),
                    to: Some(player_idx),
                    stamp_name: stamp.name.clone(),
                    stamp_id: format!("{}", stamp.destination_id.unwrap_or(0)),
                    is_first_class,
                    initiator: Some(player_idx),
                });
                
                self.add_stamp_with_checks(player_idx, stamp);
                println!("💸 Ai furat ștampila {} de la {} pentru M{}!", stamp_name, self.players[target_idx].name, price);
                self.log_action(Some(player_idx), format!("{} stole {} from {} for M{}", self.players[player_idx].name, stamp_name, self.players[target_idx].name, price));
            } else {
                // Dacă nu are bani, returnăm ștampila țintei
                self.players[target_idx].passport.add_stamp(stamp);
                println!("Nu ai destui bani să plătești ștampila (M{})!", price);
            }
        } else {
            println!("Jucătorul țintă nu are nicio ștampilă de furat!");
        }
    }

    /// Place a bid during an auction. Any player can bid.
    pub fn place_bid(&mut self, bidder_idx: usize, bid_amount: u32) -> Result<TurnResult, String> {
        let (dest_id, current_bid, _highest_bidder) = match &self.step {
            GameStep::WaitingForAuction { dest_id, current_bid, highest_bidder } => (*dest_id, *current_bid, *highest_bidder),
            _ => return Err("No auction active".to_string()),
        };

        // Validate bid increment: must be exactly +20, +50, or +100
        let increment = (bid_amount as i32).checked_sub(current_bid).unwrap_or(0);
        if increment != 20 && increment != 50 && increment != 100 {
            return Err(format!("Invalid bid increment: {}. Must be +20, +50, or +100", increment));
        }

        // Validate player has enough money
        if self.players[bidder_idx].money < bid_amount as i32 {
            return Err("Not enough money to bid".to_string());
        }

        println!("{}", format!("🔨 {} bids M{}!", self.players[bidder_idx].name, bid_amount).bright_yellow());

        self.step = GameStep::WaitingForAuction {
            dest_id,
            current_bid: bid_amount as i32,
            highest_bidder: Some(bidder_idx),
        };

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[bidder_idx].position as u8,
            went_to_jail: false,
            turn_ends: false,
            current_player_index: self.current_player_idx as u8,
        })
    }

    /// Resolve an auction when the timer expires. Awards property to highest bidder.
    pub fn resolve_auction(&mut self) -> Result<TurnResult, String> {
        let (dest_id, current_bid, highest_bidder) = match &self.step {
            GameStep::WaitingForAuction { dest_id, current_bid, highest_bidder } => (*dest_id, *current_bid, *highest_bidder),
            _ => return Err("No auction active".to_string()),
        };

        if let Some(winner_idx) = highest_bidder {
            // Winner pays and gets the property
            if let Some(dest) = self.board.find_destination_by_id(dest_id) {
                let dest = dest.clone();
                if self.players[winner_idx].pay_money(current_bid) {
                    self.history.push(GameAction::Payment {
                        from: Some(winner_idx),
                        to: None,
                        amount: current_bid,
                        initiator: None
                    });
                    println!("{}", format!("🔨 {} wins the auction for M{}!", self.players[winner_idx].name, current_bid).bright_green());
                    let a_name = self.players[winner_idx].name.clone();
                    let a_dest_name = dest.name.clone();
                    self.log_action(Some(winner_idx), format!("{} won auction for {} at ${}", a_name, a_dest_name, current_bid));
                    self.acquire_property(winner_idx, &dest);
                } else {
                    println!("{}", "Winner can't afford the bid!".red());
                }
            }
        } else {
            println!("{}", "🔨 No one bid. Property stays unowned.".yellow());
        }

        self.step = GameStep::WaitingForRoll;
        self.end_turn();

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[self.current_player_idx].position as u8,
            went_to_jail: false,
            turn_ends: true,
            current_player_index: self.current_player_idx as u8,
        })
    }

    /// Helper pentru a adăuga o proprietate unui jucător, a verifica seturile și a verifica victoria
    fn acquire_property(&mut self, player_idx: usize, dest: &Destination) -> bool {
        let stamp = Stamp::from_destination(dest);
        println!("{}", format!("✅ {} a cumpărat destinația {} și a primit stampila!", self.players[player_idx].name, dest.name).green());
        
                    self.history.push(GameAction::StampTransfer {
                                from: None,
                                to: Some(player_idx),
                                stamp_name: stamp.name.clone(),
                                stamp_id: format!("{}", dest.id),
                                is_first_class: false,
                                initiator: Some(player_idx),
                            });

        let name = self.players[player_idx].name.clone();
        let dest_name = dest.name.clone();
        self.log_action(Some(player_idx), format!("{} bought [DOT:{:?}] {}", name, dest.color, dest_name));

        self.add_stamp_with_checks(player_idx, stamp)
    }

    /// Adaugă o ștampilă și verifică seturi/victorie
    fn add_stamp_with_checks(&mut self, player_idx: usize, stamp: Stamp) -> bool {
        let dest_id = stamp.destination_id;
        self.players[player_idx].passport.add_stamp(stamp);

        // Dacă e o destinație, verificăm dacă s-a completat un set
        if let Some(id) = dest_id {
            if let Some(dest) = self.board.find_destination_by_id(id) {
                if let Some(color_set) = self.board.color_sets.get(&dest.color) {
                    if self.players[player_idx].has_color_set(color_set) {
                        println!("{}", "🎉 Ai completat setul de culoare! Primești o stampilă Clasa Întâi GRATIS!".bright_green());
                        self.give_first_class_stamp(player_idx, true);
                    }
                }
            }
        }

        self.check_and_handle_win(player_idx)
    }

    fn give_first_class_stamp(&mut self, player_idx: usize, free: bool) {
        if self.first_class_stamps_available == 0 {
            println!("{}", "Nu mai sunt stampile Clasa Întâi!".red());
            return;
        }

        let stamp = Stamp::first_class();
        if self.players[player_idx].passport.add_stamp(stamp) {
            self.first_class_stamps_available -= 1;

            if free {
                println!("{}", "✅ Ai primit stampila Clasa Întâi GRATUIT!".bright_green());
            } else {
                println!("{}", "✅ Ai cumpărat stampila Clasa Întâi!".green());
            }

            self.check_and_handle_win(player_idx);
        } else {
            println!("{}", "⚠️ Pașaportul tău este plin!".red());
            if !free {
                self.players[player_idx].add_money(100);
            }
        }
    }

    fn handle_bankruptcy(&mut self, player_idx: usize, creditor_idx: Option<usize>) {
        println!("\n{}", "💸 FALIMENT!".red().bold());

        if let Some(stamp) = self.players[player_idx].passport.remove_last_stamp() {
            if let Some(creditor) = creditor_idx {
                println!("{}", format!("Ultima ștampilă '{}' merge la {}",
                                       stamp.name, self.players[creditor].name).red());
                self.add_stamp_with_checks(creditor, stamp);
            } else {
                println!("{}", format!("Ultima ștampilă '{}' revine pe tablă", stamp.name).red());
                // When returning to board, the property becomes unowned.
                // find_destination_by_id and similar will naturally see it's not in anyone's passport.
            }
        } else {
            println!("{}", "Jucătorul nu are ștampile de cedat.".yellow());
        }

        println!("{}", format!("{} rămâne fără bani suficienți pentru această plată, dar păstrează cash-ul curent (M{}) și continuă să joace!",
                               self.players[player_idx].name, self.players[player_idx].money).yellow());
    }

    fn find_destination_owner(&self, dest_id: u8) -> Option<usize> {
        for (i, player) in self.players.iter().enumerate() {
            if player.passport.get_destination_ids().contains(&dest_id) {
                return Some(i);
            }
        }
        None
    }

    #[allow(dead_code)]
    fn display_current_player_status(&self) {
        self.players[self.current_player_idx].display_status();
    }

    fn get_space_name(&self, space: &Space) -> String {
        match space {
            Space::Start => "START".to_string(),
            Space::Destination(dest) => dest.name.clone(),
            Space::FirstClass => "Clasa Întâi".to_string(),
            Space::Airport => "Aeroport".to_string(),
            Space::HereAndNow => "Here & Now".to_string(),
            Space::Chance => "Șansă".to_string(),
            Space::FreeParking => "Parcare Gratuită".to_string(),
            Space::GoToJail => "Mergi la Închisoare".to_string(),
            Space::JustVisiting => "În Vizită / Închisoare".to_string(),
        }
    }

    fn can_anyone_swap(&self, player_idx: usize) -> bool {
        if self.players[player_idx].passport.stamp_count() == 0 {
            return false;
        }
        self.players.iter().enumerate().any(|(i, p)| i != player_idx && p.passport.stamp_count() > 0)
    }

    #[allow(dead_code)]
    fn wait_for_enter(&self) {
        print!("\n{}", "Apasă ENTER pentru următoarea tură...".bright_black());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }
}
