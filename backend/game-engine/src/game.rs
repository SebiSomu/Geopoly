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
    WaitingForPurchaseDecision { dest_id: u8, price: u32 },
    WaitingForFirstClassDecision,
    WaitingForAirportDecision,
    WaitingForAirportDestination,
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
        }
    }

    /// Execută o aruncare de zaruri pentru jucătorul curent
    pub fn roll_dice(&mut self) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over".to_string());
        }
        if self.step != GameStep::WaitingForRoll {
            return Err("Not waiting for roll".to_string());
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

                if self.players[player_idx].passport.stamp_count() == 0 {
                    println!("\n{}", "💼 AFACERE FORȚATĂ! (N-ai ștampile => mutare automată)".bright_magenta());
                    self.move_player(d2 as i32);
                    self.handle_landing();
                    
                    let turn_ends = match self.step {
                        GameStep::WaitingForPurchaseDecision { .. } | 
                        GameStep::WaitingForFirstClassDecision |
                        GameStep::WaitingForAirportDecision |
                        GameStep::WaitingForAirportDestination => false,
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
                self.handle_landing();
                self.players[player_idx].consecutive_doubles = 0; // Reset doubles

                // Only end turn if no decisions are pending
                let turn_ends = match self.step {
                    GameStep::WaitingForPurchaseDecision { .. } | 
                    GameStep::WaitingForFirstClassDecision |
                    GameStep::WaitingForAirportDecision |
                    GameStep::WaitingForAirportDestination => false,
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
                self.handle_landing();

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
                    GameStep::WaitingForPurchaseDecision { .. } | 
                    GameStep::WaitingForFirstClassDecision |
                    GameStep::WaitingForAirportDecision |
                    GameStep::WaitingForAirportDestination
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
            self.handle_landing();
            
            // Verificăm dacă tura se încheie (poate a aterizat pe o proprietate)
            let turn_ends = match self.step {
                GameStep::WaitingForPurchaseDecision { .. } | 
                GameStep::WaitingForFirstClassDecision |
                GameStep::WaitingForAirportDecision |
                GameStep::WaitingForAirportDestination => false,
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
                     let move_amount = d1 + d2;
                     self.move_player(move_amount as i32);
                     self.handle_landing();
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
            if released && is_forced_deal && self.players[player_idx].passport.stamp_count() > 0 {
                 println!("\n{}", "💼 AFACERE FORȚATĂ!".bright_magenta());
                 self.step = GameStep::WaitingForForcedDeal;
                 // Nu mai e nevoie să setăm last_dice aici, e setat deja sus
            }

            let turn_ends = match self.step {
                GameStep::WaitingForPurchaseDecision { .. } |
                GameStep::WaitingForFirstClassDecision |
                GameStep::WaitingForAirportDecision |
                GameStep::WaitingForAirportDestination |
                GameStep::WaitingForForcedDeal => false,
                _ => true,
            };

            if turn_ends {
                self.end_turn();
            }
            
            Ok(TurnResult {
                die1: d1, die2: d2,
                is_double: false,
                is_forced_deal: released && is_forced_deal && self.players[player_idx].passport.stamp_count() > 0,
                new_position: self.players[player_idx].position as u8,
                went_to_jail: !released,
                turn_ends,
                current_player_index: self.current_player_idx as u8,
            })
        }
    }

    pub fn resolve_forced_deal(&mut self, action: &str, target_name: Option<String>) -> Result<TurnResult, String> {
        if self.step != GameStep::WaitingForForcedDeal {
            return Err("Not waiting for forced deal".to_string());
        }

        let player_idx = self.current_player_idx;
        
        match action {
            "sneaky_swap" => {
                let target_idx = if let Some(name) = target_name {
                    self.players.iter().position(|p| p.name == name)
                } else {
                    None
                };
                self.handle_business_deal(target_idx);
            },
            "move" => {
                // Mută cu valoarea de pe die2 (salvată în last_dice)
                let steps = if let Some((_, d2)) = self.last_dice { d2 as i32 } else { 1 };
                println!("Ales 'Move': Mută {} spații", steps);
                self.move_player(steps);
                self.handle_landing();
            }
            _ => return Err("Invalid action".to_string())
        }

        // Verificăm dacă am ajuns într-o stare de așteptare decizie
        let turn_ends = match self.step {
            GameStep::WaitingForPurchaseDecision { .. } | 
            GameStep::WaitingForFirstClassDecision |
            GameStep::WaitingForAirportDecision |
            GameStep::WaitingForAirportDestination => false,
            _ => true,
        };

        if turn_ends {
            self.step = GameStep::WaitingForRoll;
            self.end_turn();
        }

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
        let (dest_id, price) = match &self.step {
            GameStep::WaitingForPurchaseDecision { dest_id, price } => (*dest_id, *price),
            _ => return Err("Not waiting for purchase decision".to_string()),
        };

        let player_idx = self.current_player_idx;

        if buy {
            // Găsește destinația pentru a crea ștampila
            if let Some(dest) = self.board.find_destination_by_id(dest_id) {
                let dest = dest.clone();
                if self.players[player_idx].pay_money(price) {
                    // Cumpărarea proprietății (include verificare set și win)
                    if self.acquire_property(player_idx, &dest) {
                        self.step = GameStep::WaitingForRoll;
                        return Ok(TurnResult {
                            die1: 0, die2: 0,
                            is_double: false,
                            is_forced_deal: false,
                            new_position: self.players[player_idx].position as u8,
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
        }

        self.step = GameStep::WaitingForRoll;
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
    }

    /// Rezolvă decizia de cumpărare First Class
    pub fn resolve_first_class(&mut self, buy: bool) -> Result<TurnResult, String> {
        if self.step != GameStep::WaitingForFirstClassDecision {
            return Err("Not waiting for first class decision".to_string());
        }

        let player_idx = self.current_player_idx;

        if buy {
            if self.players[player_idx].pay_money(100) {
                let stamp = Stamp::first_class();

                // Logica de furt First Class de alți jucători
                let mut stealer_idx: Option<usize> = None;
                for i in 0..self.players.len() {
                    if i != player_idx && self.players[i].steal_first_class_ready {
                        println!("{}", format!("✈️ {} FURĂ ștampila First Class cu cardul Here&Now!", self.players[i].name).bright_magenta());
                        stealer_idx = Some(i);

                        self.players[i].steal_first_class_ready = false;
                        if let Some(pos) = self.players[i].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::StealFirstClass)) {
                            let card = self.players[i].here_and_now_cards.remove(pos);
                            self.here_and_now_deck.discard(card);
                        }
                        break;
                    }
                }

                let final_idx = stealer_idx.unwrap_or(player_idx);
                self.players[final_idx].passport.add_stamp(stamp);
                self.first_class_stamps_available -= 1;

                if stealer_idx.is_some() {
                    println!("{}", format!("✅ {} a furat stampila Clasa Întâi!", self.players[final_idx].name).bright_green());
                } else {
                    println!("{}", "✅ Ai cumpărat stampila Clasa Întâi!".green());
                }

                self.check_and_handle_win(final_idx);
            } else {
                println!("{}", "Nu ai suficienți bani!".red());
            }
        } else {
            println!("Jucătorul a refuzat să cumpere First Class.");
        }

        self.step = GameStep::WaitingForRoll;
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
    }

    /// Rezolvă decizia de zbor de la Aeroport
    pub fn resolve_airport_decision(&mut self, buy_flight: bool) -> Result<TurnResult, String> {
        if self.step != GameStep::WaitingForAirportDecision {
            return Err("Not waiting for airport decision".to_string());
        }

        let player_idx = self.current_player_idx;

        if buy_flight {
            if self.players[player_idx].pay_money(100) {
                println!("✅ Ai plătit M100 pentru zbor! Alege destinația.");
                self.step = GameStep::WaitingForAirportDestination;
            } else {
                println!("{}", "Nu ai suficienți bani!".red());
                self.step = GameStep::WaitingForRoll;
                self.end_turn();
            }
        } else {
            println!("Jucătorul a refuzat zborul.");
            self.step = GameStep::WaitingForRoll;
            self.end_turn();
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
        let idx = self.current_player_idx;
        if !matches!(self.step, GameStep::WaitingForAirportDestination) {
            return Err("Nu ești în etapa de a alege destinația de zbor!".to_string());
        }

        self.players[idx].position = target_position as usize;
        println!("🛬 {} a aterizat pe poziția {}!", self.players[idx].name, target_position);

        self.step = GameStep::WaitingForRoll; // Reset implicitly for now
        self.handle_landing();

        let turn_ends = matches!(self.step, GameStep::WaitingForRoll);

        if turn_ends {
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

    /// Folosește un cartonaș Here&Now din mână
    pub fn use_here_and_now_card(&mut self, player_idx: usize, card_id: String) -> Result<TurnResult, String> {
        let card_idx = self.players[player_idx].here_and_now_cards.iter().position(|c| c.id == card_id)
            .ok_or_else(|| "Nu deții acest cartonaș!".to_string())?;

        let card = self.players[player_idx].here_and_now_cards.remove(card_idx);
        println!("🎭 {} folosește cartonașul: {}", self.players[player_idx].name, card.description);

        self.execute_here_and_now_action(player_idx, card.action.clone())?;
        self.here_and_now_deck.discard(card);

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

    fn execute_here_and_now_action(&mut self, player_idx: usize, action: HereAndNowCardAction) -> Result<(), String> {
        match action {
            HereAndNowCardAction::MoveSteps(steps) => {
                let board_size = self.board.spaces.len();
                if self.players[player_idx].move_by(steps, board_size) {
                    println!("💰 Ai trecut pe la START! Primești M200.");
                    self.players[player_idx].add_money(200);
                }
                self.handle_landing();
            }
            HereAndNowCardAction::MoveAnywhere => {
                if player_idx == self.current_player_idx {
                    self.step = GameStep::WaitingForAirportDestination;
                    println!("🚀 Alege orice destinație de pe tablă!");
                }
            }
            HereAndNowCardAction::GetOutOfJailFree => {
                if self.players[player_idx].in_jail {
                    self.players[player_idx].release_from_jail();
                    println!("🔓 Ai ieșit gratuit din închisoare!");
                }
            }
            HereAndNowCardAction::SwapStamps => {
                let my_stamp = self.players[player_idx].passport.remove_last_stamp();
                if let Some(s1) = my_stamp {
                    for i in 0..self.players.len() {
                        if i != player_idx {
                            if let Some(s2) = self.players[i].passport.remove_last_stamp() {
                                println!("♻️ Schimb de ștampile între {} și {}!", self.players[player_idx].name, self.players[i].name);
                                self.add_stamp_with_checks(player_idx, s2);
                                self.add_stamp_with_checks(i, s1);
                                return Ok(());
                            }
                        }
                    }
                    self.players[player_idx].passport.add_stamp(s1);
                }
            }
            HereAndNowCardAction::TakeAllLastStamps => {
                for i in 0..self.players.len() {
                    if i != player_idx {
                        if let Some(stamp) = self.players[i].passport.remove_last_stamp() {
                             println!("🚫 Ștampila {} a lui {} a fost scoasă din pașaport!", stamp.name, self.players[i].name);
                        }
                    }
                }
            }
            HereAndNowCardAction::CollectFromRichest => {
                let mut richest_idx = 0;
                let mut max_stamps = 0;
                for (i, p) in self.players.iter().enumerate() {
                    let count = p.passport.left_column.len() + p.passport.right_column.len();
                    if count > max_stamps {
                        max_stamps = count;
                        richest_idx = i;
                    }
                }
                if richest_idx != player_idx {
                    let amount = 200;
                    if self.players[richest_idx].money >= amount {
                        self.players[richest_idx].pay_money(amount);
                        self.players[player_idx].add_money(amount);
                        println!("💰 {} (cel mai bogat) ți-a plătit M200!", self.players[richest_idx].name);
                    } else {
                         let money = self.players[richest_idx].money;
                         self.players[richest_idx].pay_money(money);
                         self.players[player_idx].add_money(money);
                         println!("💰 {} ți-a plătit tot ce avea (M{})!", self.players[richest_idx].name, money);
                    }
                }
            }
            HereAndNowCardAction::InterceptPurchase => self.players[player_idx].intercept_purchase_ready = true,
            HereAndNowCardAction::SayNo => self.players[player_idx].say_no_cards += 1,
            HereAndNowCardAction::DiscountPurchase => self.players[player_idx].discount_purchase_ready = true,
            HereAndNowCardAction::CollectTax => self.players[player_idx].collect_tax_ready = true,
            HereAndNowCardAction::StealFirstClass => self.players[player_idx].steal_first_class_ready = true,
        }
        Ok(())
    }

    fn end_turn(&mut self) {
        if self.players[self.current_player_idx].consecutive_doubles == 0 || self.players[self.current_player_idx].in_jail {
             // Logică incrementare tură globală
            if self.current_player_idx == self.players.len() - 1 {
                self.turn_number += 1;
                println!("\n{}", "=".repeat(60).on_bright_blue().white());
                println!("{}  ÎNCEPE TURA GLOBALĂ #{}  {}", "🌍".yellow(), self.turn_number, "🌍".yellow());
            }
            
            self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
            self.step = GameStep::WaitingForRoll;
            
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
        let player = &mut self.players[self.current_player_idx];
        let old_position = player.position;

        let passed_start = player.move_by(steps, board_size);

        println!("\n{} se mută de la poziția {} la poziția {}",
                 player.name.yellow(),
                 old_position,
                 player.position);

        if passed_start {
            player.add_money(200);
            println!("{}", "✨ Ai trecut pe la START! Primești M200.".green());
        }
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
                self.players[idx].add_money(amount);
                println!("{}", format!("Primești M{}", amount).green());
            }

            ChanceCardAction::PayHospital => {
                let amount = 200;
                if !self.players[idx].pay_money(amount) {
                    println!("{}", "Nu ai suficienți bani!".red());
                    self.handle_bankruptcy(idx, None);
                } else {
                    println!("{}", format!("Plătești M{}", amount).yellow());
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
                self.players[idx].add_money(gain);
                println!("{}", format!("Ai {} ștampile First Class => primești M{}", count, gain).bright_green());
            }

            ChanceCardAction::CollectFromEachPlayer(amount) => {
                println!("{}", format!("Colectezi M{} de la fiecare jucător", amount).green());
                for i in 0..self.players.len() {
                    if i != idx {
                        if self.players[i].pay_money(amount) {
                            self.players[idx].add_money(amount);
                        } else {
                            println!("{}", format!("{} nu are bani suficienți!", self.players[i].name).yellow());
                        }
                    }
                }
            }

            ChanceCardAction::MoveSteps(steps) => {
                self.move_player(steps);
                self.handle_landing();
            }

            ChanceCardAction::AdvanceToStart => {
                self.players[idx].move_to(0);
                self.players[idx].add_money(200);
                println!("{}", "Avansezi la START și colectezi M200.".bright_green());
            }

            ChanceCardAction::GoToJail => {
                self.players[idx].send_to_jail();
                println!("{}", "Mergi direct la închisoare! (Fără START)".red());
            }

            ChanceCardAction::GetOutOfJailFree => {
                if self.players[idx].in_jail {
                    self.players[idx].release_from_jail();
                    println!("🔓 Ai ieșit gratuit din închisoare!");
                } else {
                    // Dacă îl "folosește" când nu e în închisoare (probabil din greșeală sau UI a permis)
                    // Îl punem la loc dacă nu e cazul sau doar dăm mesaj
                    println!("Nu ești în închisoare!");
                }
            }

            ChanceCardAction::RerollOneDie => {
                let mut rng = rand::thread_rng();
                let d = rng.gen_range(1..=6);
                println!("{}", format!("Arunci încă un zar: {}. Te muți {} spații.", d, d).cyan());
                self.move_player(d);
                self.handle_landing();
            }

            ChanceCardAction::DiceChallenge => {
                let opponent = (0..self.players.len()).find(|&i| i != idx);
                if let Some(opp) = opponent {
                    let mut rng = rand::thread_rng();
                    let my = rng.gen_range(1..=6);
                    let his = rng.gen_range(1..=6);
                    println!("{}", format!("Duel zaruri: tu={} vs {}={}", my, self.players[opp].name, his).bright_cyan());

                    if my > his {
                        println!("{}", format!("{} plătește M100 către tine.", self.players[opp].name).green());
                        if self.players[opp].pay_money(100) {
                            self.players[idx].add_money(100);
                        }
                    } else if his > my {
                        println!("{}", format!("Tu plătești M100 către {}.", self.players[opp].name).yellow());
                        if self.players[idx].pay_money(100) {
                            self.players[opp].add_money(100);
                        }
                    } else {
                        println!("Egalitate! Nimeni nu plătește.");
                    }
                }
            }
            ChanceCardAction::SwapTwoPlayersStamps => {
                // Simplificat: primii doi jucători diferiți de robot? sau primii doi în general
                 if self.players.len() >= 2 {
                     let (idx1, idx2) = (0, 1);
                     let s1 = self.players[idx1].passport.remove_last_stamp();
                     let s2 = self.players[idx2].passport.remove_last_stamp();
                     
                     if let Some(stamp1) = s1 { self.players[idx2].passport.add_stamp(stamp1); }
                     if let Some(stamp2) = s2 { self.players[idx1].passport.add_stamp(stamp2); }
                     
                     println!("♻️ Schimb de ștampile între {} și {}!", self.players[idx1].name, self.players[idx2].name);
                 }
            }
            ChanceCardAction::StealStampAndPay => {
                 let target = (0..self.players.len()).find(|&i| i != idx);
                 if let Some(t_idx) = target {
                     if let Some(stamp) = self.players[t_idx].passport.remove_last_stamp() {
                         // Plătim valoarea (estimată la M100 pentru simplitate, regulile pot varia)
                         if self.players[idx].pay_money(150) {
                             self.players[t_idx].add_money(150);
                             let stamp_name = stamp.name.clone();
                             self.add_stamp_with_checks(idx, stamp);
                             println!("💸 Ai furat ștampila {} de la {} pentru M150!", stamp_name, self.players[t_idx].name);
                         } else {
                             self.players[t_idx].passport.add_stamp(stamp); // returnăm
                             println!("Nu ai destui bani să plătești ștampila!");
                         }
                     }
                 }
            }
        }
        Ok(())
    }
    fn handle_landing(&mut self) {
        let position = self.players[self.current_player_idx].position;
        let space = self.board.get_space(position).clone();

        println!("\n{} {}", "📍 Ai ajuns pe:".cyan(), self.get_space_name(&space).yellow().bold());

        match space {
            Space::Start => {
                println!("{}", "🏁 START - Relaxează-te!".green());
            }
            Space::Destination(dest) => {
                self.handle_destination(dest);
            }
            Space::FirstClass => {
                self.handle_first_class();
            }
            Space::Airport => {
                self.handle_airport();
            }
            Space::HereAndNow => {
                self.handle_here_and_now();
            }
            Space::Chance => {
                self.handle_chance();
            }
            Space::FreeParking => {
                println!("{}", "🅿️ Parcare Gratuită - Ia o pauză!".green());
            }
            Space::GoToJail => {
                println!("{}", "👮 Mergi direct la ÎNCHISOARE!".red());
                let player = &mut self.players[self.current_player_idx];
                player.send_to_jail();
            }
            Space::JustVisiting => {
                println!("{}", "👀 Doar în vizită la închisoare.".cyan());
            }
        }
    }

    fn handle_destination(&mut self, dest: Destination) {
        println!("🌍 {} - Preț: M{}, Taxă: M{}",
                 dest.name, dest.price, dest.tourist_tax);

        // Verificăm cine deține această destinație
        let owner_idx = self.find_destination_owner(dest.id);

        if let Some(owner_idx) = owner_idx {
            if owner_idx == self.current_player_idx {
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
                if self.players[self.current_player_idx].collect_tax_ready {
                    println!("{}", "🧾 COLLECT TAX ACTIV! Colectezi taxa în loc să o plătești!".bright_green());

                    if self.players[owner_idx].pay_money(tax) {
                        self.players[self.current_player_idx].add_money(tax);
                        println!("{}", format!("{} îți plătește M{}", self.players[owner_idx].name, tax).green());
                    } else {
                        println!("{}", format!("{} nu are bani!", self.players[owner_idx].name).yellow());
                    }

                    self.players[self.current_player_idx].collect_tax_ready = false;

                    // Găsim și ștergem cardul din mână
                    if let Some(pos) = self.players[self.current_player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::CollectTax)) {
                        let card = self.players[self.current_player_idx].here_and_now_cards.remove(pos);
                        self.here_and_now_deck.discard(card);
                    }

                    return;
                }

                println!("{}", format!("Plătești M{} către {}", tax, self.players[owner_idx].name).yellow());

                if self.players[self.current_player_idx].pay_money(tax) {
                    self.players[owner_idx].add_money(tax);
                    println!("{}", "Taxă plătită!".green());
                } else {
                    println!("{}", "Nu ai suficienți bani!".red());
                    self.handle_bankruptcy(self.current_player_idx, Some(owner_idx));
                }
            }
        } else {
            // Destinația nu este deținută de nimeni
            println!("\n{}", "Această destinație nu este deținută de nimeni!".cyan());

            let mut final_price = dest.price;
            let mut buyer_idx = self.current_player_idx;

            // ✅ DISCOUNT PURCHASE: plătești doar M100
            if self.players[self.current_player_idx].discount_purchase_ready {
                final_price = 100;
                println!("{}", "💸 DISCOUNT PURCHASE ACTIV! Plătești doar M100!".bright_green());

                self.players[self.current_player_idx].discount_purchase_ready = false;

                // Găsim și ștergem cardul din mână
                if let Some(pos) = self.players[self.current_player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::DiscountPurchase)) {
                    let card = self.players[self.current_player_idx].here_and_now_cards.remove(pos);
                    self.here_and_now_deck.discard(card);
                }
            }

            // ✅ INTERCEPT PURCHASE: alt jucător interceptează cumpărarea
            for i in 0..self.players.len() {
                if i != self.current_player_idx && self.players[i].intercept_purchase_ready {
                    if self.players[i].money >= final_price {
                        println!("{}", format!("🎯 {} INTERCEPTEAZĂ cumpărarea cu cardul Here&Now!", self.players[i].name).bright_magenta());
                        buyer_idx = i;

                        self.players[i].intercept_purchase_ready = false;

                        // Găsim și ștergem cardul din mână
                        if let Some(pos) = self.players[i].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::InterceptPurchase)) {
                            let card = self.players[i].here_and_now_cards.remove(pos);
                            self.here_and_now_deck.discard(card);
                        }

                        break;
                    }
                }
            }

            println!("Vrei să o cumperi pentru M{}? (y/n)", final_price);

            // Verificăm dacă jucătorul are bani - dacă da, așteptăm decizia
            if self.players[buyer_idx].money >= final_price {
                // Setăm starea pentru a aștepta decizia jucătorului
                self.step = GameStep::WaitingForPurchaseDecision { 
                    dest_id: dest.id, 
                    price: final_price 
                };
                println!("Așteptăm decizia jucătorului...");
                // Nu facem nimic altceva - metoda resolve_purchase() va gestiona restul
            } else {
                println!("Nu ai destui bani pentru a cumpăra această proprietate.");
                // Nu organizăm licitație deocamdată - trecem direct mai departe
            }
        }
    }

    fn handle_first_class(&mut self) {
        println!("✈️ Clasa Întâi disponibilă pentru M100");

        if self.first_class_stamps_available == 0 {
            println!("Nu mai sunt stampile Clasa Întâi.");
            return;
        }

        let idx = self.current_player_idx;

        // Verificăm dacă jucătorul are bani pentru a cumpăra
        if self.players[idx].money >= 100 {
            // Setăm starea pentru a aștepta decizia jucătorului
            self.step = GameStep::WaitingForFirstClassDecision;
            println!("Așteptăm decizia jucătorului pentru First Class...");
        } else {
            println!("Nu ai destui bani pentru a cumpăra First Class.");
        }
    }

    fn handle_airport(&mut self) {
        println!("✈️ AEROPORT - Poți zbura oriunde pentru M100");
        let idx = self.current_player_idx;
        if self.players[idx].money >= 100 {
            self.step = GameStep::WaitingForAirportDecision;
            println!("Așteptăm decizia jucătorului pentru aeroport...");
        } else {
            println!("Nu ai destui bani pentru zbor.");
        }
    }

    fn handle_here_and_now(&mut self) {
        println!("🎴 Tragi un cartonaș Here & Now...");

        let card = self.here_and_now_deck.draw();
        println!("{}", format!("📜 {}", card.description).bright_cyan());

        let idx = self.current_player_idx;

        // îl punem în mână mereu
        self.players[idx].here_and_now_cards.push(card.clone());

        // pentru anumite cărți, „armăm” efectul ca flag (simplificare)
        match card.action {
            HereAndNowCardAction::SayNo => {
                self.players[idx].say_no_cards += 1;
                println!("{}", "✅ Ai primit 'Spune nu!' (îl poți folosi oricând).".bright_green());
            }
            HereAndNowCardAction::InterceptPurchase => {
                self.players[idx].intercept_purchase_ready = true;
                println!("{}", "✅ Intercept Purchase este ACTIV (când altcineva cumpără, îl poți intercepta).".bright_green());
            }
            HereAndNowCardAction::DiscountPurchase => {
                self.players[idx].discount_purchase_ready = true;
                println!("{}", "✅ Discount Purchase este ACTIV (următorul loc nedeținut: plătești M100).".bright_green());
            }
            HereAndNowCardAction::CollectTax => {
                self.players[idx].collect_tax_ready = true;
                println!("{}", "✅ Collect Tax este ACTIV (la următoarea taxă, colectezi în loc să plătești).".bright_green());
            }
            HereAndNowCardAction::StealFirstClass => {
                self.players[idx].steal_first_class_ready = true;
                println!("{}", "✅ Steal First Class este ACTIV (când altcineva primește First Class, îl furi).".bright_green());
            }
            _ => {
                println!("{}", "✅ Cartonaș păstrat în mână (folosibil oricând).".bright_green());
            }
        }
    }

    fn handle_chance(&mut self) {
        println!("🎲 Tragi un cartonaș Șansă...");

        let card = self.chance_deck.draw();
        println!("{}", format!("📜 {}", card.description).bright_yellow());

        let idx = self.current_player_idx;

        if card.can_keep {
            self.players[idx].chance_cards.push(card);
            println!("{}", "✅ Cartonaș păstrat în mână (folosibil oricând).".bright_green());
        } else {
            let action = card.action.clone();
            let _ = self.execute_chance_action(idx, action);
            self.chance_deck.discard(card);
        }
    }

    fn handle_business_deal(&mut self, target_idx: Option<usize>) {
        // Schimbăm ultima stampilă cu a unui adversar
        let current_player_idx = self.current_player_idx;

        if self.players[current_player_idx].passport.stamp_count() == 0 {
            println!("{}", "Nu ai stampile de schimbat!".yellow());
            return;
        }

        // Dacă nu avem target_idx (vechea logică sau nespecificat), găsim primul adversar eligibil
        let opp_idx = if let Some(idx) = target_idx {
            if idx == current_player_idx || self.players[idx].passport.stamp_count() == 0 {
                None
            } else {
                Some(idx)
            }
        } else {
            self.players.iter()
                .enumerate()
                .find(|(i, p)| *i != current_player_idx && p.passport.stamp_count() > 0)
                .map(|(i, _)| i)
        };

        if let Some(opp_idx) = opp_idx {
            if let Some(my_stamp) = self.players[current_player_idx].passport.remove_last_stamp() {
                if let Some(opp_stamp) = self.players[opp_idx].passport.remove_last_stamp() {
                    println!("{}", format!("Schimbi '{}' cu '{}' de la {}",
                                           my_stamp.name, opp_stamp.name, self.players[opp_idx].name).bright_magenta());

                    self.add_stamp_with_checks(current_player_idx, opp_stamp);
                    self.add_stamp_with_checks(opp_idx, my_stamp);

                    // Verifică câștig pentru ambii jucători
                    if self.check_and_handle_win(current_player_idx) {
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

    fn handle_auction(&mut self, dest: &Destination) {
        println!("\n{}", "🔨 LICITAȚIE!".bright_yellow());
        println!("Licitația pornește de la M20");

        for i in 0..self.players.len() {
            if self.players[i].money >= 20 {
                println!("{} cumpără la licitație pentru M20!", self.players[i].name);
                self.players[i].pay_money(20);
                self.acquire_property(i, dest);
                return;
            }
        }

        println!("{}", "Nimeni nu vrea destinația!".yellow());
    }

    /// Helper pentru a adăuga o proprietate unui jucător, a verifica seturile și a verifica victoria
    fn acquire_property(&mut self, player_idx: usize, dest: &Destination) -> bool {
        let stamp = Stamp::from_destination(dest);
        println!("{}", format!("✅ {} a cumpărat destinația {} și a primit stampila!", self.players[player_idx].name, dest.name).green());
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
            }
        }

        println!("{}", format!("{} rămâne fără bani dar continuă să joace!",
                               self.players[player_idx].name).yellow());
    }

    fn find_destination_owner(&self, dest_id: u8) -> Option<usize> {
        for (i, player) in self.players.iter().enumerate() {
            if player.passport.get_destination_ids().contains(&dest_id) {
                return Some(i);
            }
        }
        None
    }

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

    fn wait_for_enter(&self) {
        print!("\n{}", "Apasă ENTER pentru următoarea tură...".bright_black());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }
}