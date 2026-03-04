use crate::board::{Board, Space, Destination};
use crate::player::Player;
use crate::cards::{ChanceDeck, HereAndNowDeck, ChanceCardAction, HereAndNowCardAction};
use crate::passport::Stamp;
use rand::Rng;
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
    WaitingForStampSelection { player_idx: usize, action: String },
}

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
    Payment { is_tax: bool, from: Option<usize>, to: Option<usize>, amount: i32, initiator: Option<usize> },
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
    pub previous_step: Option<Box<GameStep>>,
    pub has_rolled_this_turn: bool,
}

impl Game {
    pub fn new(player_names: Vec<String>) -> Self {
        let board = Board::new();
        let mut players: Vec<Player> = player_names
            .into_iter()
            .map(|name| Player::new(name))
            .collect();

        let mut here_and_now_deck = HereAndNowDeck::new();

        for player in &mut players {
            let (card1, card2) = here_and_now_deck.deal_initial();
            player.here_and_now_cards.push(card1);
            player.here_and_now_cards.push(card2);
        }

        let game = Game {
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
            previous_step: None,
            has_rolled_this_turn: false,
        };
        game
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
        let window_size = 15;
        for action in self.history.iter().rev().take(window_size) {
            match action {
                GameAction::Payment { from, to, initiator, .. } => {
                    if *from == Some(player_idx) && *initiator != Some(player_idx) && *to != None { return true; }
                    if *initiator == Some(player_idx) { return false; }
                },
                GameAction::StampTransfer { from, initiator, .. } => {
                    if *from == Some(player_idx) && *initiator != Some(player_idx) { return true; }
                    if *initiator == Some(player_idx) { return false; }
                },
                GameAction::GoToJail { player_idx: pj } => {
                    if *pj == player_idx { return false; }
                },
                GameAction::Move { player_idx: pm, .. } => {
                    if *pm == player_idx { return false; }
                }
            }
        }
        false
    }

    pub fn check_can_player_use_discount(&self, player_idx: usize) -> bool {
        // 1. Reactive: recently paid > 100 to board
        let recently_paid = self.history.iter().rev().take(10).any(|action| {
            if let GameAction::Payment { is_tax: false, from, to: None, amount, .. } = action {
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
            if record.buyer_idx == player_idx {
                return false;
            }

            if record.is_first_class {
                return false;
            }

            for (idx, p) in self.players.iter().enumerate() {
                if p.passport.find_stamp_index(&record.name).is_some() {
                    return idx != player_idx;
                }
            }
            false
        } else {
            false
        }
    }

    pub fn check_can_player_use_collect_tax(&self, player_idx: usize) -> bool {
        for action in self.history.iter().rev().take(5) {
            match action {
                GameAction::Move { .. } => break,
                GameAction::Payment { is_tax: true, from, to: Some(_), .. } => {
                    if *from == Some(player_idx) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    pub fn check_can_player_use_steal_first_class(&self, player_idx: usize) -> bool {
        if let Some(record) = &self.last_purchase {
            if !record.is_first_class {
                return false;
            }
            if record.buyer_idx == player_idx {
                return false;
            }

            let buyer = &self.players[record.buyer_idx];
            for stamp in buyer.passport.left_column.iter().chain(buyer.passport.right_column.iter()) {
                if stamp.destination_id.is_none() {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    pub fn roll_dice(&mut self) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }

        self.last_purchase = None;
        if !matches!(self.step, GameStep::WaitingForRoll | GameStep::WaitingForRerollDice { .. }) {
            return Err("Not waiting for roll".to_string());
        }

        if let GameStep::WaitingForRerollDice { player_idx } = self.step {
            if self.current_player_idx != player_idx {
                return Err("Not your turn to reroll".to_string());
            }
            return self.resolve_reroll_dice();
        }

        let player_idx = self.current_player_idx;
        let in_jail = self.players[player_idx].in_jail;

        if in_jail {
            return self.handle_jail_roll();
        }

        if self.has_rolled_this_turn && self.players[player_idx].consecutive_doubles == 0 {
            return Err("You already rolled the dice this turn!".to_string());
        }

        let dice_result = Self::roll_dice_internal();

        self.has_rolled_this_turn = true;

        match dice_result {
            DiceResult::BusinessDeal(val) => {
                let d2 = val;
                self.last_dice = Some((1, d2));
                self.players[player_idx].consecutive_doubles = 0;

                if !self.can_anyone_swap(player_idx) {
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
                        GameStep::WaitingForStampSelection { .. } => false,
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
                    self.step = GameStep::WaitingForForcedDeal;

                    Ok(TurnResult {
                        die1: 1,
                        die2: d2,
                        is_double: false,
                        is_forced_deal: true,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: false,
                        current_player_index: self.current_player_idx as u8,
                    })
                }
            }
            DiceResult::Normal(d1, d2) => {
                let total = d1 + d2;
                self.last_dice = Some((d1, d2));
                self.move_player(total as i32);
                self.handle_landing(player_idx);
                self.players[player_idx].consecutive_doubles = 0;

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
                    GameStep::WaitingForStampSelection { .. } => false,
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

                if self.players[player_idx].double_blocked {
                    self.players[player_idx].double_blocked = false;
                    self.players[player_idx].consecutive_doubles = 0;
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{}'s double was blocked — turn ends", name));
                    self.end_turn();
                    return Ok(TurnResult {
                        die1: d1,
                        die2: d2,
                        is_double: true,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    });
                }

                if self.players[player_idx].skip_next_turn {
                    self.end_turn();
                    return Ok(TurnResult {
                        die1: d1,
                        die2: d2,
                        is_double: true,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    });
                }

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
                    turn_ends: pending_decision,
                    current_player_index: self.current_player_idx as u8,
                })
            }
        }
    }

    fn handle_jail_roll(&mut self) -> Result<TurnResult, String> {
        let player_idx = self.current_player_idx;
        self.players[player_idx].jail_turns += 1;


        let dice_result = Self::roll_dice_internal();

        let (d1, d2, _is_forced_deal) = match dice_result {
            DiceResult::Normal(a, b) => (a, b, false),
            DiceResult::Double(a) => (a, a, false),
            DiceResult::BusinessDeal(b) => (1, b, true),
        };

        self.last_dice = Some((d1, d2));
        self.has_rolled_this_turn = true;

        if let DiceResult::Double(val) = dice_result {
            self.players[player_idx].release_from_jail();
            self.move_player((val * 2) as i32);
            self.handle_landing(player_idx);

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

            if self.players[player_idx].jail_turns >= 3 {
                if self.players[player_idx].pay_money(100) {
                    self.players[player_idx].release_from_jail();
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} paid $100 to get out of prison (no move this turn)", name));
                    self.step = GameStep::WaitingForRoll;
                    self.end_turn();
                    return Ok(TurnResult {
                        die1: d1, die2: d2,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: true,
                        current_player_index: self.current_player_idx as u8,
                    });
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
                GameStep::WaitingForStampSelection { .. } => false,
                _ => true,
            };

            if turn_ends {
                self.end_turn();
            }

            Ok(TurnResult {
                die1: d1, die2: d2,
                is_double: false,
                is_forced_deal: false,
                new_position: self.players[player_idx].position as u8,
                went_to_jail: true,
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
                    self.players[player_idx].release_from_jail();
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} paid $100 up to get out of jail", name));
                    self.step = GameStep::WaitingForRoll;
                    Ok(TurnResult {
                        die1: 0, die2: 0,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: false,
                        current_player_index: self.current_player_idx as u8,
                    })
                } else {
                    Err("You do not have enough money for the fine!".to_string())
                }
            },
            JailAction::UseCard => {
                if let Some(pos) = self.players[player_idx].chance_cards.iter().position(|c| c.id == "jail_free") {
                    let card = self.players[player_idx].chance_cards.remove(pos);
                    self.chance_deck.discard(card);
                    self.players[player_idx].release_from_jail();
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} used a Get Out of Jail Free card", name));
                    self.step = GameStep::WaitingForRoll;
                    Ok(TurnResult {
                        die1: 0, die2: 0,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: false,
                        current_player_index: self.current_player_idx as u8,
                    })
                } else if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::GetOutOfJailFree)) {
                    let card = self.players[player_idx].here_and_now_cards.remove(pos);
                    self.here_and_now_deck.discard(card);
                    self.players[player_idx].release_from_jail();
                    let name = self.players[player_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} used a Get Out of Jail Free card", name));
                    self.step = GameStep::WaitingForRoll;
                    Ok(TurnResult {
                        die1: 0, die2: 0,
                        is_double: false,
                        is_forced_deal: false,
                        new_position: self.players[player_idx].position as u8,
                        went_to_jail: false,
                        turn_ends: false,
                        current_player_index: self.current_player_idx as u8,
                    })
                } else {
                    Err("You do not have a Get Out of Jail Free card!".to_string())
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

            self.move_player(d as i32);
            let new_pos = self.players[player_idx].position;

            self.history.push(GameAction::Move {
                player_idx,
                from: old_pos as u8,
                to: new_pos as u8
            });

            self.handle_landing(player_idx);

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

            self.pending_reroll = None;

            if has_pending_decision {
            } else {

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
        if self.game_over {
            return Err("Game is over!".to_string());
        }
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
                let steps = if let Some((_, d2)) = self.last_dice { d2 as i32 } else { 1 };
                self.move_player(steps);
                self.handle_landing(player_idx);
            }
            _ => return Err("Invalid action".to_string())
        }

        let turn_ends = match self.step {
            GameStep::WaitingForPurchaseDecision { .. } |
            GameStep::WaitingForFirstClassDecision { .. } |
            GameStep::WaitingForAirportDecision { .. } |
            GameStep::WaitingForAirportDestination { .. } |
            GameStep::WaitingForTargetSelection { .. } |
            GameStep::WaitingForDiceDuel { .. } |
            GameStep::WaitingForAuction { .. } |
            GameStep::WaitingForRerollDice { .. } => false,
            GameStep::WaitingForStampSelection { .. } => false,
            _ => true,
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

    pub fn resolve_purchase(&mut self, buy: bool) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let (dest_id, price, buyer_idx) = match &self.step {
            GameStep::WaitingForPurchaseDecision { dest_id, price, buyer_idx } => (*dest_id, *price, *buyer_idx),
            _ => return Err("Not waiting for purchase decision".to_string()),
        };

        let player_idx = buyer_idx;

        if buy {
            if let Some(dest) = self.board.find_destination_by_id(dest_id) {
                let dest = dest.clone();
                if self.players[player_idx].pay_money(price) {
                    self.last_purchase = Some(PurchaseRecord {
                        dest_id,
                        buyer_idx: player_idx,
                        price,
                        name: dest.name.clone(),
                        is_first_class: false,
                    });

                    self.history.push(GameAction::Payment { is_tax: false,
                        from: Some(player_idx),
                        to: None,
                        amount: price,
                        initiator: None
                    });


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
                }
            }
        } else {
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

        let had_prev = self.previous_step.is_some();
        if let Some(prev) = self.previous_step.take() {
            self.step = *prev;
        } else {
            self.step = GameStep::WaitingForRoll;
            if buyer_idx == self.current_player_idx { self.end_turn(); }
        }

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[buyer_idx].position as u8,
            went_to_jail: false,
            turn_ends: !had_prev && buyer_idx == self.current_player_idx,
            current_player_index: self.current_player_idx as u8,
        })
    }

    pub fn resolve_first_class(&mut self, buy: bool) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let buyer_idx = match &self.step {
            GameStep::WaitingForFirstClassDecision { buyer_idx } => *buyer_idx,
            _ => return Err("Not waiting for first class decision".to_string()),
        };

        let player_idx = buyer_idx;

        if buy {
            if self.players[player_idx].pay_money(100) {
                self.last_purchase = Some(PurchaseRecord {
                    dest_id: 0,
                    buyer_idx: player_idx,
                    price: 100,
                    name: "First Class".to_string(),
                    is_first_class: true,
                });

                self.history.push(GameAction::Payment { is_tax: false,
                    from: Some(player_idx),
                    to: None,
                    amount: 100,
                    initiator: None
                });


                let fc_player_name = self.players[player_idx].name.clone();
                self.log_action(Some(player_idx), format!("{} bought First Class", fc_player_name));

                let stamp = Stamp::first_class();

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


                self.check_and_handle_win(player_idx);
            } else {
            }
        } else {
        }

        let had_prev = self.previous_step.is_some();
        if let Some(prev) = self.previous_step.take() {
            self.step = *prev;
        } else {
            self.step = GameStep::WaitingForRoll;
            if buyer_idx == self.current_player_idx { self.end_turn(); }
        }

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[buyer_idx].position as u8,
            went_to_jail: false,
            turn_ends: !had_prev && buyer_idx == self.current_player_idx,
            current_player_index: self.current_player_idx as u8,
        })
    }

    pub fn resolve_airport_decision(&mut self, buy_flight: bool) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let buyer_idx = if let GameStep::WaitingForAirportDecision { buyer_idx } = self.step {
            buyer_idx
        } else {
            return Err("Not waiting for airport decision".to_string());
        };

        let player_idx = buyer_idx;

        if buy_flight {
            if self.players[player_idx].pay_money(100) {
                self.history.push(GameAction::Payment { is_tax: false,
                    from: Some(player_idx),
                    to: None,
                    amount: 100,
                    initiator: None
                });
                self.step = GameStep::WaitingForAirportDestination { buyer_idx: player_idx };
            } else {
                self.step = GameStep::WaitingForRoll;
                self.end_turn();
            }
        } else {
        }
        if !buy_flight {
           let had_prev = self.previous_step.is_some();
           if let Some(prev) = self.previous_step.take() {
                self.step = *prev;
           } else {
                self.step = GameStep::WaitingForRoll;
                self.end_turn();
           }

            return Ok(TurnResult {
                die1: 0, die2: 0,
                is_double: false,
                is_forced_deal: false,
                new_position: self.players[player_idx].position as u8,
                went_to_jail: false,
                turn_ends: !had_prev,
                current_player_index: self.current_player_idx as u8,
            });
        }

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[player_idx].position as u8,
            went_to_jail: false,
            turn_ends: false,
            current_player_index: self.current_player_idx as u8,
        })
    }

    pub fn resolve_airport_destination(&mut self, target_position: u8) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let idx = if let GameStep::WaitingForAirportDestination { buyer_idx } = self.step {
            buyer_idx
        } else {
            return Err("You are not in the stage of choosing a flight destination!".to_string());
        };

        self.players[idx].position = target_position as usize;

        self.step = GameStep::WaitingForRoll;
        self.handle_landing(idx);

        let turn_ends = matches!(self.step, GameStep::WaitingForRoll);

        if turn_ends && idx == self.current_player_idx {
            if let Some(prev) = self.previous_step.take() {
                self.step = *prev;
            } else {
                self.end_turn();
            }
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
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let (action, _card_id, selector_idx) = match &self.step {
            GameStep::WaitingForTargetSelection { action, card_id, selector_idx } => (action.clone(), card_id.clone(), *selector_idx),
            _ => return Err("You are not in the stage of choosing a player!".to_string()),
        };

        let target_idx = self.players.iter().position(|p| p.name == target_name)
            .ok_or("Target player does not exist!")?;

        if target_idx == selector_idx {
            return Err("You cannot choose yourself!".to_string());
        }

        match action.as_str() {
            "SwapStamps" => {
                self.handle_stamp_swap(selector_idx, Some(target_idx));
            },
            "DiceDuel" => {
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
            "BlockNextDouble" => {
                self.players[target_idx].double_blocked = true;
                let selector_name = self.players[selector_idx].name.clone();
                let target_name_str = self.players[target_idx].name.clone();
                self.log_action(Some(selector_idx), format!("{} blocked next double for {}", selector_name, target_name_str));
            }
            _ => return Err("Acțiune necunoscută".to_string())
        }

        let is_duel = matches!(self.step, GameStep::WaitingForDiceDuel { .. });

        if !is_duel {
            let action_by_current_player = selector_idx == self.current_player_idx;

            if action_by_current_player {
                if let Some(prev) = self.previous_step.take() {
                    self.step = *prev;
                } else {
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
                }
            } else {
                if self.pending_reroll.is_some() {
                    self.step = GameStep::WaitingForRerollDice { player_idx: self.current_player_idx };
                } else if let Some(prev) = self.previous_step.take() {
                    self.step = *prev;
                } else if self.has_rolled_this_turn && self.players[self.current_player_idx].consecutive_doubles == 0 {
                    self.step = GameStep::WaitingForRoll;
                    self.end_turn();
                } else {
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
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let (challenger_idx, target_idx, mut c_roll, mut t_roll) = match &self.step {
            GameStep::WaitingForDiceDuel { challenger_idx, target_idx, challenger_roll, target_roll } =>
                (*challenger_idx, *target_idx, *challenger_roll, *target_roll),
            _ => return Err("You are not in a dice duel!".to_string()),
        };



        // Roll TWO dice
        let d1 = rand::thread_rng().gen_range(1..=6);
        let d2 = rand::thread_rng().gen_range(1..=6);


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
                die1: t1, die2: t2,
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
            _ => return Err("There is no duel to finalize!".to_string()),
        };

        let cr_sum = c1 + c2;
        let tr_sum = t1 + t2;

        if cr_sum > tr_sum {
            self.players[target_idx].pay_money(100);
            self.players[challenger_idx].add_money(100);
            self.history.push(GameAction::Payment { is_tax: false,
                from: Some(target_idx),
                to: Some(challenger_idx),
                amount: 100,
                initiator: None
            });
            let d_loser = self.players[target_idx].name.clone();
            let d_winner = self.players[challenger_idx].name.clone();
            self.log_action(Some(target_idx), format!("{} paid $100 to {}", d_loser, d_winner));
        } else if tr_sum > cr_sum {
            self.players[challenger_idx].pay_money(100);
            self.players[target_idx].add_money(100);
            self.history.push(GameAction::Payment { is_tax: false,
                from: Some(challenger_idx),
                to: Some(target_idx),
                amount: 100,
                initiator: None
            });
            let d_loser2 = self.players[challenger_idx].name.clone();
            let d_winner2 = self.players[target_idx].name.clone();
            self.log_action(Some(challenger_idx), format!("{} paid $100 to {}", d_loser2, d_winner2));
        } else {
        }

        self.step = GameStep::WaitingForRoll;
        self.end_turn();
        self.update_all_reactive_statuses();
        Ok(())
    }

    pub fn use_here_and_now_card(&mut self, player_idx: usize, card_id: String) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let card_id_inner = card_id.clone();
        let card = self.players[player_idx].here_and_now_cards.iter().find(|c| c.id == card_id_inner)
            .ok_or_else(|| "You do not hold this card!".to_string())?.clone();

        let old_step = self.step.clone();
        // Save current step before executing action, so we can restore it if action triggers a sub-step
        self.previous_step = Some(Box::new(old_step.clone()));

        self.execute_here_and_now_action(player_idx, card.action.clone())?;

        if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| c.id == card_id) {
            self.players[player_idx].here_and_now_cards.remove(pos);
        }

        let player_name = self.players[player_idx].name.clone();
        self.log_action(Some(player_idx), format!("{} used treasure card: {}", player_name, card.description));
        self.here_and_now_deck.discard(card);
        self.update_all_reactive_statuses();

        let step_changed = self.step != old_step;
        if !step_changed {
            self.previous_step = None;
        }

        let current_step_is_jail = matches!(self.step, GameStep::WaitingForJailDecision);
        if player_idx != self.current_player_idx && self.players[self.current_player_idx].in_jail && !current_step_is_jail {
            self.step = GameStep::WaitingForJailDecision;
        }

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
            GameStep::WaitingForStampSelection { .. } => false,
            _ => {
                false
            }
        };

        Ok(TurnResult {
            die1: 0,
            die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[self.current_player_idx].position as u8,
            went_to_jail: false,
            turn_ends,
            current_player_index: self.current_player_idx as u8,
        })
    }

    fn execute_here_and_now_action(&mut self, player_idx: usize, action: HereAndNowCardAction) -> Result<(), String> {
        match action {
            HereAndNowCardAction::MoveSteps(steps) => {
                if player_idx != self.current_player_idx {
                    return Err("You can only use this card on your turn!".to_string());
                }
                if self.players[player_idx].in_jail {
                    return Err("You cannot use this card while in prison!".to_string());
                }
                let board_size = self.board.spaces.len();
                if self.players[player_idx].move_by(steps, board_size) {
                    self.players[player_idx].add_money(200);
                }
                self.handle_landing(player_idx);
            }
            HereAndNowCardAction::MoveAnywhere => {
                if player_idx != self.current_player_idx {
                    return Err("You can only use this card on your turn!".to_string());
                }
                if self.players[player_idx].in_jail {
                    return Err("You cannot use this card while in prison!".to_string());
                }
                self.step = GameStep::WaitingForAirportDestination { buyer_idx: player_idx };
            }
            HereAndNowCardAction::GetOutOfJailFree => {
                if !self.players[player_idx].in_jail {
                    return Err("You are not in prison!".to_string());
                }
                self.players[player_idx].release_from_jail();
                self.step = GameStep::WaitingForRoll;
                self.previous_step = None;
            }
            HereAndNowCardAction::SwapStamps => {
                if self.players[player_idx].passport.stamp_count() == 0 {
                    return Err("You have no stamps to offer for swap!".to_string());
                }
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
                    return Err("No other player has stamps!".to_string());
                }

                for i in 0..self.players.len() {
                    if i != player_idx {
                        if let Some(stamp) = self.players[i].passport.remove_last_stamp() {

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
                    return Err("There are no other players!".to_string());
                }

                let max_stamps = opponents.iter()
                    .map(|&i| self.players[i].passport.stamp_count())
                    .max()
                    .unwrap_or(0);

                if max_stamps == 0 {
                    return Err("No opponent has stamps!".to_string());
                }

                let my_stamps = self.players[player_idx].passport.stamp_count();
                if max_stamps <= my_stamps {
                    return Err("No one has more stamps than you!".to_string());
                }

                let mut richest: Vec<usize> = opponents.into_iter()
                    .filter(|&i| self.players[i].passport.stamp_count() == max_stamps)
                    .collect();

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
                        self.history.push(GameAction::Payment { is_tax: false,
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
                    return Err("No recent purchase to intercept!".to_string());
                }

                if let Some(record) = purchase_data {
                    if record.buyer_idx == player_idx {
                        return Err("You cannot intercept your own purchase!".to_string());
                    }

                    let price = record.price;
                    let stamp_name = record.name.clone();

                    let mut current_owner_idx: Option<usize> = None;
                    for (idx, p) in self.players.iter().enumerate() {
                        if p.passport.find_stamp_index(&stamp_name).is_some() {
                            current_owner_idx = Some(idx);
                            break;
                        }
                    }

                    let old_buyer_idx = if let Some(owner_idx) = current_owner_idx {
                        owner_idx
                    } else {
                        return Err("Property no longer exists in the game!".to_string());
                    };

                    if self.players[player_idx].pay_money(price) {
                        self.players[old_buyer_idx].add_money(price);

                        if let Some(pos) = self.players[old_buyer_idx].passport.find_stamp_index(&stamp_name) {
                            if let Some(stamp) = self.players[old_buyer_idx].passport.remove_stamp_at(pos) {
                                let s_id = format!("{}", stamp.destination_id.unwrap_or(0));
                                let is_fc = stamp.destination_id.is_none();

                                self.add_stamp_with_checks(player_idx, stamp);

                                self.history.push(GameAction::Payment { is_tax: false, from: Some(player_idx), to: None, amount: price, initiator: None });
                                self.history.push(GameAction::Payment { is_tax: false, from: None, to: Some(old_buyer_idx), amount: price, initiator: None });

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
                        return Err("Property could not be transferred.".to_string());
                    } else {
                        return Err("You do not have enough money for interception!".to_string());
                    }
                } else {
                    return Err("No recent purchase to intercept.".to_string());
                }
            },
            HereAndNowCardAction::SayNo => {
                let window_size = 15;
                let found_revert: Option<(usize, GameAction)> = {
                    let recent: Vec<(usize, &GameAction)> = self.history.iter()
                        .enumerate()
                        .rev()
                        .take(window_size)
                        .collect();

                    let mut result = None;
                    for (i, action) in &recent {
                        let mut resolved = false;
                        match action {
                            GameAction::Payment { from, to, initiator, .. } => {
                                // Say No valid only when paying to another player (to: Some(player)), not bank (to: None)
                                if *from == Some(player_idx) && *initiator != Some(player_idx) && *to != None {
                                    result = Some((*i, (*action).clone()));
                                    resolved = true;
                                } else if *initiator == Some(player_idx) {
                                    resolved = true;
                                }
                            },
                            GameAction::StampTransfer { from, initiator, .. } => {
                                if *from == Some(player_idx) && *initiator != Some(player_idx) {
                                    result = Some((*i, (*action).clone()));
                                    resolved = true;
                                } else if *initiator == Some(player_idx) {
                                    resolved = true;
                                }
                            },
                            GameAction::GoToJail { player_idx: pj } => {
                                if *pj == player_idx { resolved = true; }
                            },
                            GameAction::Move { player_idx: pm, .. } => {
                                if *pm == player_idx { resolved = true; }
                            }
                        }

                        if resolved {
                            break;
                        }
                    }
                    result
                };

                if let Some((idx_in_history, action)) = found_revert {
                    self.players[player_idx].say_no_cards += 1;

                    match action {
                        GameAction::Payment { is_tax: _, from, to, amount, .. } => {
                            if let (Some(me), Some(creditor)) = (from, to) {
                                if me == player_idx {
                                    if self.players[creditor].pay_money(amount) {
                                        self.players[me].add_money(amount);
                                        self.log_action(Some(player_idx), format!("Just Say No! Payment of M{} cancelled", amount));
                                        self.history.push(GameAction::Payment { is_tax: false,
                                            from: Some(creditor),
                                            to: Some(me),
                                            amount,
                                            initiator: Some(player_idx)
                                        });
                                    }
                                }
                            }
                        },

                        // ----------------------------------------------------------
                        // STAMP TRANSFER revert
                        // ----------------------------------------------------------
                        GameAction::StampTransfer { from, to, stamp_name, stamp_id: _, is_first_class: _, initiator } => {
                            let mut swap_pair: Option<GameAction> = None;

                            if idx_in_history > 0 {
                                if let Some(GameAction::StampTransfer { initiator: prev_init, from: pf, to: pt, .. }) = self.history.get(idx_in_history - 1) {
                                    if *prev_init == initiator {
                                        let same_people = match (&from, &to, pf, pt) {
                                            (Some(a), Some(b), Some(c), Some(d)) => (*a == *c && *b == *d) || (*a == *d && *b == *c),
                                            _ => false,
                                        };
                                        if same_people {
                                            swap_pair = Some(self.history[idx_in_history - 1].clone());
                                        }
                                    }
                                }
                            }

                            if swap_pair.is_none() && idx_in_history + 1 < self.history.len() {
                                if let Some(GameAction::StampTransfer { initiator: next_init, from: nf, to: nt, .. }) = self.history.get(idx_in_history + 1) {
                                    if *next_init == initiator {
                                        let same_people = match (&from, &to, nf, nt) {
                                            (Some(a), Some(b), Some(c), Some(d)) => (*a == *c && *b == *d) || (*a == *d && *b == *c),
                                            _ => false,
                                        };
                                        if same_people {
                                            swap_pair = Some(self.history[idx_in_history + 1].clone());
                                        }
                                    }
                                }
                            }

                            if let Some(other_transfer) = swap_pair {
                                // ============================================================
                                // SWAP REVERT ATOMIC
                                // ============================================================
                                if let GameAction::StampTransfer {
                                    stamp_name: other_stamp_name,
                                    from: other_from,
                                    to: other_to,
                                    ..
                                } = other_transfer
                                {
                                    if let (Some(h1), Some(o1), Some(h2), Some(o2)) = (to, from, other_to, other_from) {
                                        let sn  = stamp_name.clone();
                                        let osn = other_stamp_name.clone();

                                        let pos1 = self.players[h1].passport.find_stamp_index(&sn);
                                        let pos2 = self.players[h2].passport.find_stamp_index(&osn);

                                        if let (Some(p1), Some(p2)) = (pos1, pos2) {
                                            let sa = self.players[h1].passport.remove_stamp_at(p1);
                                            let sb = self.players[h2].passport.remove_stamp_at(p2);

                                            if let (Some(sa), Some(sb)) = (sa, sb) {
                                                let sa_name = sa.name.clone();
                                                let sb_name = sb.name.clone();
                                                let sa_id   = sa.destination_id;
                                                let sb_id   = sb.destination_id;

                                                self.add_stamp_with_checks(o1, sa);
                                                self.add_stamp_with_checks(o2, sb);

                                                self.log_action(Some(player_idx), format!(
                                                    "Just Say No! Swap {} <-> {} reversed atomically",
                                                    sa_name, sb_name
                                                ));

                                                self.history.push(GameAction::StampTransfer {
                                                    from: Some(h1),
                                                    to: Some(o1),
                                                    stamp_name: sa_name,
                                                    stamp_id: format!("{}", sa_id.unwrap_or(0)),
                                                    is_first_class: sa_id.is_none(),
                                                    initiator: Some(player_idx),
                                                });
                                                self.history.push(GameAction::StampTransfer {
                                                    from: Some(h2),
                                                    to: Some(o2),
                                                    stamp_name: sb_name,
                                                    stamp_id: format!("{}", sb_id.unwrap_or(0)),
                                                    is_first_class: sb_id.is_none(),
                                                    initiator: Some(player_idx),
                                                });
                                            } else {
                                                return Err("Could not find both stamps from the swap!".to_string());
                                            }
                                        } else {
                                            return Err("Could not locate the swap stamps in passports!".to_string());
                                        }
                                    } else {
                                        return Err("Invalid swap data in history!".to_string());
                                    }
                                }

                            } else {
                                if from == Some(player_idx) {
                                    match to {
                                        Some(tgt) => {
                                            if let Some(pos) = self.players[tgt].passport.find_stamp_index(&stamp_name) {
                                                if let Some(s) = self.players[tgt].passport.remove_stamp_at(pos) {
                                                    let s_name = s.name.clone();
                                                    let s_id   = s.destination_id;
                                                    self.add_stamp_with_checks(player_idx, s);
                                                    self.log_action(Some(player_idx), format!("Just Say No! Stamp {} recovered", stamp_name));
                                                    self.history.push(GameAction::StampTransfer {
                                                        from: Some(tgt),
                                                        to: Some(player_idx),
                                                        stamp_name: s_name,
                                                        stamp_id: format!("{}", s_id.unwrap_or(0)),
                                                        is_first_class: s_id.is_none(),
                                                        initiator: Some(player_idx),
                                                    });

                                                    if idx_in_history > 0 {
                                                        if let Some(GameAction::Payment { from: pf, to: pt, amount: pa, initiator: pi, .. })
                                                            = self.history.get(idx_in_history - 1).cloned()
                                                        {
                                                            if pf == Some(tgt) && pt == Some(player_idx) && pi == initiator {
                                                                if self.players[player_idx].pay_money(pa) {
                                                                    self.players[tgt].add_money(pa);
                                                                    self.history.push(GameAction::Payment { is_tax: false,
                                                                        from: Some(player_idx),
                                                                        to: Some(tgt),
                                                                        amount: pa,
                                                                        initiator: Some(player_idx),
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    return Err(format!("Stamp {} could not be extracted!", stamp_name));
                                                }
                                            } else {
                                                return Err(format!("Stamp {} is no longer with the target player!", stamp_name));
                                            }
                                        },
                                        None => {
                                            let stamp_obj = if stamp_name == "First Class" {
                                                Stamp::first_class()
                                            } else {
                                                match self.board.find_destination_by_name(&stamp_name) {
                                                    Some(d) => Stamp::from_destination(d),
                                                    None => return Err(format!("Destination {} not found on the board!", stamp_name)),
                                                }
                                            };
                                            let s_name = stamp_obj.name.clone();
                                            let s_id   = stamp_obj.destination_id;
                                            self.add_stamp_with_checks(player_idx, stamp_obj);
                                            self.log_action(Some(player_idx), format!("Just Say No! Stamp {} recovered from board", stamp_name));
                                            self.history.push(GameAction::StampTransfer {
                                                from: None,
                                                to: Some(player_idx),
                                                stamp_name: s_name,
                                                stamp_id: format!("{}", s_id.unwrap_or(0)),
                                                is_first_class: s_id.is_none(),
                                                initiator: Some(player_idx),
                                            });
                                        }
                                    }
                                } else if to == Some(player_idx) {
                                    if let Some(src) = from {
                                        if let Some(pos) = self.players[player_idx].passport.find_stamp_index(&stamp_name) {
                                            if let Some(s) = self.players[player_idx].passport.remove_stamp_at(pos) {
                                                let s_name = s.name.clone();
                                                let s_id   = s.destination_id;
                                                self.add_stamp_with_checks(src, s);
                                                self.log_action(Some(player_idx), format!("Just Say No! Stamp {} returned to original owner", stamp_name));
                                                self.history.push(GameAction::StampTransfer {
                                                    from: Some(player_idx),
                                                    to: Some(src),
                                                    stamp_name: s_name,
                                                    stamp_id: format!("{}", s_id.unwrap_or(0)),
                                                    is_first_class: s_id.is_none(),
                                                    initiator: Some(player_idx),
                                                });
                                            }
                                        } else {
                                            return Err(format!("Stamp {} is no longer with you!", stamp_name));
                                        }
                                    }
                                } else {
                                    return Err("This type of transfer cannot be cancelled with Just Say No!".to_string());
                                }
                            }
                        },

                        _ => {}
                    }
                } else {
                    return Err("There is no recent action against you that you can cancel!".to_string());
                }
            },
            HereAndNowCardAction::DiscountPurchase => {
                self.players[player_idx].discount_purchase_ready = true;

                if let GameStep::WaitingForPurchaseDecision { dest_id, price, buyer_idx } = &self.step {
                    let (dest_id, price, buyer_idx) = (*dest_id, *price, *buyer_idx);
                    if buyer_idx == player_idx && price > 100 {
                        self.step = GameStep::WaitingForPurchaseDecision {
                            dest_id,
                            price: 100,
                            buyer_idx
                        };
                        self.players[player_idx].discount_purchase_ready = false;
                        return Ok(());
                    }
                }

                let refund_data = self.history.iter().rev().take(10).find_map(|action| {
                    if let GameAction::Payment { is_tax: _, from, to: None, amount, .. } = action {
                        if *from == Some(player_idx) && *amount > 100 {
                            return Some(*amount - 100);
                        }
                    }
                    None
                });

                if let Some(refund) = refund_data {
                    self.players[player_idx].add_money(refund);
                    self.players[player_idx].discount_purchase_ready = false;
                }
            },
            HereAndNowCardAction::CollectTax => {
                let found_revert = {
                    let mut result = None;
                    for action in self.history.iter().rev().take(5) {
                        match action {
                            GameAction::Move { .. } => break,
                            GameAction::Payment { is_tax: true, from, to: Some(receiver_idx), amount, .. } => {
                                if *from == Some(player_idx) {
                                    result = Some((*receiver_idx, *amount));
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                    result
                };

                if found_revert.is_none() {
                    return Err("You cannot use this card — you have not paid any tax this turn!".to_string());
                }

                self.players[player_idx].collect_tax_ready = true;

                if let Some((receiver_idx, amount)) = found_revert {
                    let total = amount * 2;
                    if self.players[receiver_idx].pay_money(total) {
                        self.players[player_idx].add_money(total);
                        self.history.push(GameAction::Payment { is_tax: false,
                            from: Some(receiver_idx),
                            to: Some(player_idx),
                            amount: total,
                            initiator: None
                        });
                    } else if self.players[receiver_idx].pay_money(amount) {
                        self.players[player_idx].add_money(amount);
                        self.history.push(GameAction::Payment { is_tax: false,
                            from: Some(receiver_idx),
                            to: Some(player_idx),
                            amount,
                            initiator: None
                        });
                    } else {
                    }
                    self.players[player_idx].collect_tax_ready = false;
                }
            },
            HereAndNowCardAction::StampAmnesty => {
                if self.players[player_idx].passport.stamp_count() == 0 {
                    return Err("You have no stamps to sell!".to_string());
                }
                self.step = GameStep::WaitingForStampSelection {
                    player_idx,
                    action: "StampAmnesty".to_string(),
                };
            },
            HereAndNowCardAction::BlockNextDouble => {
                let has_targets = self.players.iter().enumerate()
                    .any(|(i, _)| i != player_idx);

                if !has_targets {
                    return Err("There are no other players!".to_string());
                }

                self.step = GameStep::WaitingForTargetSelection {
                    action: "BlockNextDouble".to_string(),
                    card_id: None,
                    selector_idx: player_idx,
                };
            },
            HereAndNowCardAction::StealFirstClass => {
                let mut target_idx: Option<usize> = None;
                let mut amount = 0;

                for (idx, p) in self.players.iter().enumerate() {
                    for stamp in p.passport.left_column.iter().chain(p.passport.right_column.iter()) {
                        if stamp.destination_id.is_none() { // First Class stamp
                            if idx != player_idx {
                                target_idx = Some(idx);
                                if let Some(record) = &self.last_purchase {
                                    amount = record.price;
                                }
                                break;
                            }
                        }
                    }
                    if target_idx.is_some() {
                        break;
                    }
                }

                let target_idx = match target_idx {
                    Some(idx) => idx,
                    None => return Err("No First Class stamp to steal from other players.".to_string()),
                };

                if let Some(stamp) = self.players[target_idx].passport.remove_last_stamp() {

                    // Refund the owner
                    self.players[target_idx].add_money(amount);

                    let s_name = stamp.name.clone();
                    let s_id = stamp.destination_id.map(|id| format!("{}", id)).unwrap_or_else(|| "first_class".to_string());
                    let is_fc = true; // By definition for this card

                    self.history.push(GameAction::StampTransfer {
                        from: Some(target_idx),
                        to: Some(player_idx),
                        stamp_name: s_name,
                        stamp_id: s_id,
                        is_first_class: is_fc,
                        initiator: Some(player_idx),
                    });

                    self.add_stamp_with_checks(player_idx, stamp);
                    self.log_action(Some(player_idx), format!("Steal First Class used against {}", self.players[target_idx].name));
                } else {
                    return Err("Target player no longer has the stamp!".to_string());
                }
            }
        }
        Ok(())
    }

    fn end_turn(&mut self) {
        let in_jail = self.players[self.current_player_idx].in_jail;
        let consecutive_doubles = self.players[self.current_player_idx].consecutive_doubles;

        if consecutive_doubles == 0 || in_jail {
            if self.current_player_idx == self.players.len() - 1 {
                self.turn_number += 1;
            }

            let p = &mut self.players[self.current_player_idx];
            p.collect_tax_ready = false;
            p.discount_purchase_ready = false;
            p.intercept_purchase_ready = false;
            p.steal_first_class_ready = false;

            self.has_rolled_this_turn = false;

            self.current_player_idx = (self.current_player_idx + 1) % self.players.len();

            if self.players[self.current_player_idx].skip_next_turn {
                self.players[self.current_player_idx].skip_next_turn = false;
                let name = self.players[self.current_player_idx].name.clone();
                self.log_action(Some(self.current_player_idx), format!("{} skipped turn (Free Parking)", name));
                self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
            }

            if self.players[self.current_player_idx].in_jail {
                self.step = GameStep::WaitingForJailDecision;
            } else {
                self.step = GameStep::WaitingForRoll;
            }

        }
    }


    fn roll_dice_internal() -> DiceResult {
        let mut rng = rand::thread_rng();
        let d1 = rng.gen_range(1..=6);
        let d2 = rng.gen_range(1..=6);

        if d1 == 1 {
            return DiceResult::BusinessDeal(d2);
        }

        if d1 == d2 {
            DiceResult::Double(d1)
        } else {
            DiceResult::Normal(d1, d2)
        }
    }




    fn move_player(&mut self, steps: i32) {
        let board_size = self.board.total_spaces();
        let player_idx = self.current_player_idx;
        let old_position = self.players[player_idx].position;

        let passed_start = self.players[player_idx].move_by(steps, board_size);


        if passed_start {
            self.players[player_idx].add_money(200);

            self.history.push(GameAction::Payment { is_tax: false,
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

    fn check_and_handle_win(&mut self, player_idx: usize) -> bool {
        if self.players[player_idx].passport.is_full() {
            self.game_over = true;
            return true;
        }
        false
    }

    pub fn use_chance_card(&mut self, player_idx: usize, card_id: String) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let card_idx = self.players[player_idx].chance_cards.iter().position(|c| c.id == card_id)
            .ok_or_else(|| "Nu deții acest cartonaș!".to_string())?;

        let card = self.players[player_idx].chance_cards.remove(card_idx);

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
                self.history.push(GameAction::Payment { is_tax: false,
                    from: None,
                    to: Some(idx),
                    amount: amount as i32,
                    initiator: None
                });
            }

            ChanceCardAction::PayHospital => {
                let amount = 200;
                if !self.players[idx].pay_money(amount) {
                    self.handle_bankruptcy(idx, None);
                } else {
                    self.history.push(GameAction::Payment { is_tax: false,
                        from: Some(idx),
                        to: None,
                        amount,
                        initiator: Some(idx)
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
                self.history.push(GameAction::Payment { is_tax: false,
                    from: None,
                    to: Some(idx),
                    amount: gain as i32,
                    initiator: None
                });
            }

            ChanceCardAction::CollectFromEachPlayer(amount) => {
                for i in 0..self.players.len() {
                    if i != idx {
                        if self.players[i].pay_money(amount as i32) {
                            self.players[idx].add_money(amount as i32);
                            self.history.push(GameAction::Payment { is_tax: false,
                                from: Some(i),
                                to: Some(idx),
                                amount: amount as i32,
                                initiator: None
                            });
                        } else {
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

                self.history.push(GameAction::Move {
                    player_idx: idx,
                    from: old_pos as u8,
                    to: 0
                });
                self.history.push(GameAction::Payment { is_tax: false,
                    from: None,
                    to: Some(idx),
                    amount: 200,
                    initiator: None
                });
            }

            ChanceCardAction::GoToJail => {
                self.players[idx].send_to_jail();
                self.history.push(GameAction::GoToJail { player_idx: idx });
            }

            ChanceCardAction::GetOutOfJailFree => {
                if !self.players[idx].in_jail {
                    return Err("You are not in prison!".to_string());
                }
                self.players[idx].release_from_jail();
                let name = self.players[idx].name.clone();
                self.log_action(Some(idx), format!("{} used chance card: Get Out Of Jail Free", name));
                self.step = GameStep::WaitingForRoll;
                self.previous_step = None;
            }

            ChanceCardAction::RerollOneDice => {
                self.step = GameStep::WaitingForRerollDice { player_idx: idx };
                self.pending_reroll = Some(PendingReroll { player_idx: idx });
            }

            ChanceCardAction::DiceChallenge => {
                self.step = GameStep::WaitingForTargetSelection {
                    action: "DiceDuel".to_string(),
                    card_id: None,
                    selector_idx: idx
                };
            }
            ChanceCardAction::SwapTwoPlayersStamps => {
                let mut eligible: Vec<usize> = self.players.iter().enumerate()
                    .filter(|(i, p)| *i != idx && p.passport.stamp_count() > 0)
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

                            self.log_action(None, format!("♻️ Sneaky Swap: {} and {} swapped stamps ('{}' ↔ '{}')!",
                                self.players[p1_idx].name, self.players[p2_idx].name, s1.name, s2.name));

                            self.add_stamp_with_checks(p1_idx, s2);
                            self.add_stamp_with_checks(p2_idx, s1);

                            if self.check_and_handle_win(p1_idx) { return Ok(()); }
                            if self.check_and_handle_win(p2_idx) { return Ok(()); }
                        }
                    }
                } else {
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
                }
            }
            ChanceCardAction::GoToFreeParking => {
                let old_pos = self.players[idx].position;
                self.players[idx].move_to(20);

                self.history.push(GameAction::Move {
                    player_idx: idx,
                    from: old_pos as u8,
                    to: 20,
                });

                let last_3_taxes: Vec<i32> = self.history.iter()
                    .filter(|a| matches!(a, GameAction::Payment { is_tax: true, from: Some(f), .. } if *f == idx))
                    .rev()
                    .take(3)
                    .map(|a| if let GameAction::Payment { amount, .. } = a { *amount } else { 0 })
                    .collect();

                let total_refund: i32 = last_3_taxes.iter().sum();
                if total_refund > 0 {
                    self.players[idx].add_money(total_refund);
                    self.history.push(GameAction::Payment {
                        is_tax: false,
                        from: None,
                        to: Some(idx),
                        amount: total_refund,
                        initiator: Some(idx),
                    });
                } else {
                }

                self.players[idx].skip_next_turn = true;
                self.players[idx].consecutive_doubles = 0;
                let name = self.players[idx].name.clone();
                self.log_action(Some(idx), format!("{} went to Free Parking, skips next turn, recovered M{} in taxes", name, total_refund));
            }
        }
        Ok(())
    }
    fn handle_landing(&mut self, player_idx: usize) {
        let position = self.players[player_idx].position;
        let space = self.board.get_space(position).clone();


        match space {
            Space::Start => {
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
            }
            Space::GoToJail => {
                let name = self.players[player_idx].name.clone();
                let player = &mut self.players[player_idx];
                player.send_to_jail();
                self.history.push(GameAction::GoToJail { player_idx });
                self.log_action(Some(player_idx), format!("{} got into prison", name));
            }
            Space::JustVisiting => {
                let name = self.players[player_idx].name.clone();
                self.log_action(Some(player_idx), format!("{} arrived at jail but is just visiting", name));
            }
        }
    }

    fn handle_destination(&mut self, dest: Destination, player_idx: usize) {

        let owner_idx = self.find_destination_owner(dest.id);

        if let Some(owner_idx) = owner_idx {
            if owner_idx == player_idx {
            } else {
                let mut tax = dest.tourist_tax;

                if let Some(color_set) = self.board.color_sets.get(&dest.color) {
                    if self.players[owner_idx].has_color_set(color_set) {
                        tax *= 2;
                    }
                }

                if self.players[owner_idx].double_rent_active {
                    tax *= 2;
                    self.players[owner_idx].double_rent_active = false;
                }

                if self.players[player_idx].collect_tax_ready {

                    if self.players[owner_idx].pay_money(tax as i32) {
                        self.players[player_idx].add_money(tax as i32);
                        self.history.push(GameAction::Payment { is_tax: false,
                            from: Some(owner_idx),
                            to: Some(player_idx),
                            amount: tax as i32,
                            initiator: None
                        });
                    } else {
                    }

                    self.players[player_idx].collect_tax_ready = false;

                    if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::CollectTax)) {
                        let card = self.players[player_idx].here_and_now_cards.remove(pos);
                        self.here_and_now_deck.discard(card);
                    }

                    return;
                }


                if self.players[player_idx].pay_money(tax as i32) {
                    self.players[owner_idx].add_money(tax as i32);
                    self.history.push(GameAction::Payment {
                        is_tax: true,
                        from: Some(player_idx),
                        to: Some(owner_idx),
                        amount: tax as i32,
                        initiator: None
                    });
                    let payer = self.players[player_idx].name.clone();
                    let receiver = self.players[owner_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} paid ${} to {}", payer, tax, receiver));
                } else {
                    self.handle_bankruptcy(player_idx, Some(owner_idx));
                }
            }
        } else {

            let mut final_price = dest.price;
            let buyer_idx = player_idx;

            if self.players[player_idx].discount_purchase_ready {
                final_price = 100;

                self.players[player_idx].discount_purchase_ready = false;

                if let Some(pos) = self.players[player_idx].here_and_now_cards.iter().position(|c| matches!(c.action, HereAndNowCardAction::DiscountPurchase)) {
                    let card = self.players[player_idx].here_and_now_cards.remove(pos);
                    self.here_and_now_deck.discard(card);
                }
            }


            if self.players[buyer_idx].money >= final_price as i32 {
                self.step = GameStep::WaitingForPurchaseDecision {
                    dest_id: dest.id,
                    price: final_price as i32,
                    buyer_idx: buyer_idx
                };
            } else {
                self.step = GameStep::WaitingForAuction {
                    dest_id: dest.id,
                    current_bid: 20,
                    highest_bidder: None,
                };
            }
        }
    }

    fn handle_first_class(&mut self, player_idx: usize) {

        if self.first_class_stamps_available == 0 {
            return;
        }

        if self.players[player_idx].money >= 100 {
            self.step = GameStep::WaitingForFirstClassDecision { buyer_idx: player_idx };
        } else {
        }
    }

    fn handle_airport(&mut self, player_idx: usize) {
        if self.players[player_idx].money >= 100 {
            self.step = GameStep::WaitingForAirportDecision { buyer_idx: player_idx };
        } else {
        }
    }

    fn handle_here_and_now(&mut self, player_idx: usize) {

        let card = self.here_and_now_deck.draw();

        let h_name = self.players[player_idx].name.clone();
        self.log_action(Some(player_idx), format!("{} got a treasure card", h_name));

        self.players[player_idx].here_and_now_cards.push(card.clone());

        match card.action {
            HereAndNowCardAction::SayNo => {
            }
            HereAndNowCardAction::InterceptPurchase => {
            }
            HereAndNowCardAction::DiscountPurchase => {
            }
            HereAndNowCardAction::CollectTax => {
            }
            HereAndNowCardAction::StealFirstClass => {
            }
            _ => {
            }
        }
    }

    fn handle_chance(&mut self, player_idx: usize) {

        let card = self.chance_deck.draw();

        let c_name = self.players[player_idx].name.clone();
        let c_desc = card.description.clone();
        self.log_action(Some(player_idx), format!("{} got a chance card: {}", c_name, c_desc));

        if card.can_keep {
            self.players[player_idx].chance_cards.push(card);
        } else {
            let action = card.action.clone();
            let _ = self.execute_chance_action(player_idx, action);
            self.chance_deck.discard(card);
        }
    }

    fn handle_business_deal(&mut self, player_idx: usize, target_idx: Option<usize>) {

        self.last_purchase = None;

        if self.players[player_idx].passport.stamp_count() == 0 {
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
            if let Some(my_stamp) = self.players[player_idx].passport.remove_last_stamp() {
                if let Some(opp_stamp) = self.players[opp_idx].passport.remove_last_stamp() {

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

                    let my_name = self.players[player_idx].name.clone();
                    let opp_name = self.players[opp_idx].name.clone();
                    self.log_action(Some(player_idx), format!("{} swapped {} for {} with {}", my_name, my_s_name, opp_s_name, opp_name));

                    self.add_stamp_with_checks(player_idx, opp_stamp);
                    self.add_stamp_with_checks(opp_idx, my_stamp);

                    if self.check_and_handle_win(player_idx) {
                        return;
                    }
                    if self.check_and_handle_win(opp_idx) {
                        return;
                    }
                }
            }
        } else {
        }
    }

    fn handle_stamp_swap(&mut self, player_idx: usize, target_idx: Option<usize>) {

        self.last_purchase = None;

        if self.players[player_idx].passport.stamp_count() == 0 {
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

                if self.check_and_handle_win(player_idx) { return; }
                if self.check_and_handle_win(opp_idx) { return; }

                let p_name = self.players[player_idx].name.clone();
                let o_name = self.players[opp_idx].name.clone();
                self.log_action(Some(player_idx), format!("{} swapped {} for {} with {}", p_name, stamp1.name, stamp2.name, o_name));
            }
        } else {
        }
    }

    fn handle_steal_stamp_and_pay(&mut self, player_idx: usize, target_idx: usize) {
        if let Some(stamp) = self.players[target_idx].passport.remove_last_stamp() {
            let price = if stamp.destination_id.is_none() {
                100 // First Class - G100
            } else {
                self.board.find_destination_by_name(&stamp.name)
                    .map(|d| d.price)
                    .unwrap_or(200)
            };

            if self.players[player_idx].pay_money(price as i32) {
                self.players[target_idx].add_money(price as i32);
                self.history.push(GameAction::Payment { is_tax: false,
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
                self.log_action(Some(player_idx), format!("{} stole {} from {} for M{}", self.players[player_idx].name, stamp_name, self.players[target_idx].name, price));
            } else {
                self.players[target_idx].passport.add_stamp(stamp);
            }
        } else {
        }
    }

    pub fn place_bid(&mut self, bidder_idx: usize, bid_amount: u32) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
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

    pub fn resolve_stamp_amnesty(&mut self, stamp_name: String) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let (player_idx, action) = match &self.step {
            GameStep::WaitingForStampSelection { player_idx, action } => (*player_idx, action.clone()),
            _ => return Err("You are not in the stage of choosing a stamp!".to_string()),
        };

        if action != "StampAmnesty" {
            return Err("Current action is not Stamp Amnesty!".to_string());
        }

        let stamp_pos = self.players[player_idx].passport.find_stamp_index(&stamp_name)
            .ok_or_else(|| format!("You do not have the stamp '{}' in your passport!", stamp_name))?;

        let stamp = self.players[player_idx].passport.remove_stamp_at(stamp_pos)
            .ok_or_else(|| "Could not remove the stamp!".to_string())?;

        let payout = if stamp.destination_id.is_none() {
            // First Class — fixed price 100, so 150
            150
        } else {
            let original_price = self.board.find_destination_by_name(&stamp.name)
                .map(|d| d.price)
                .unwrap_or(0);
            (original_price as f32 * 1.5) as i32
        };

        self.players[player_idx].add_money(payout);


        self.history.push(GameAction::StampTransfer {
            from: Some(player_idx),
            to: None,
            stamp_name: stamp_name.clone(),
            stamp_id: format!("{}", stamp.destination_id.unwrap_or(0)),
            is_first_class: stamp.destination_id.is_none(),
            initiator: Some(player_idx),
        });
        self.history.push(GameAction::Payment { is_tax: false,
            from: None,
            to: Some(player_idx),
            amount: payout,
            initiator: Some(player_idx),
        });

        let p_name = self.players[player_idx].name.clone();
        self.log_action(Some(player_idx), format!("{} sold {} via Stamp Amnesty for M{}", p_name, stamp_name, payout));

        // Restore previous step
        let had_prev = self.previous_step.is_some();
        if let Some(prev) = self.previous_step.take() {
            self.step = *prev;
        } else {
            self.step = GameStep::WaitingForRoll;
        }

        self.update_all_reactive_statuses();

        Ok(TurnResult {
            die1: 0, die2: 0,
            is_double: false,
            is_forced_deal: false,
            new_position: self.players[self.current_player_idx].position as u8,
            went_to_jail: false,
            turn_ends: !had_prev,
            current_player_index: self.current_player_idx as u8,
        })
    }

    pub fn resolve_auction(&mut self) -> Result<TurnResult, String> {
        if self.game_over {
            return Err("Game is over!".to_string());
        }
        let (dest_id, current_bid, highest_bidder) = match &self.step {
            GameStep::WaitingForAuction { dest_id, current_bid, highest_bidder } => (*dest_id, *current_bid, *highest_bidder),
            _ => return Err("No auction active".to_string()),
        };

        if let Some(winner_idx) = highest_bidder {
            // Winner pays and gets the property
            if let Some(dest) = self.board.find_destination_by_id(dest_id) {
                let dest = dest.clone();
                if self.players[winner_idx].pay_money(current_bid) {
                    self.history.push(GameAction::Payment { is_tax: false,
                        from: Some(winner_idx),
                        to: None,
                        amount: current_bid,
                        initiator: None
                    });
                    let a_name = self.players[winner_idx].name.clone();
                    let a_dest_name = dest.name.clone();
                    self.log_action(Some(winner_idx), format!("{} won auction for {} at ${}", a_name, a_dest_name, current_bid));
                    self.acquire_property(winner_idx, &dest);
                } else {
                }
            }
        } else {
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

    fn acquire_property(&mut self, player_idx: usize, dest: &Destination) -> bool {
        let stamp = Stamp::from_destination(dest);

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

    fn add_stamp_with_checks(&mut self, player_idx: usize, stamp: Stamp) -> bool {
        let dest_id = stamp.destination_id;
        self.players[player_idx].passport.add_stamp(stamp);

        if let Some(id) = dest_id {
            if let Some(dest) = self.board.find_destination_by_id(id) {
                if let Some(color_set) = self.board.color_sets.get(&dest.color) {
                    if self.players[player_idx].has_color_set(color_set) {
                        if !self.players[player_idx].completed_color_sets.contains(&dest.color) {
                            self.players[player_idx].completed_color_sets.insert(dest.color.clone());
                            self.give_first_class_stamp(player_idx, true);
                        }
                    }
                }
            }
        }

        self.check_and_handle_win(player_idx)
    }

    fn give_first_class_stamp(&mut self, player_idx: usize, free: bool) {
        if self.first_class_stamps_available == 0 {
            return;
        }

        let stamp = Stamp::first_class();
        if self.players[player_idx].passport.add_stamp(stamp) {
            self.first_class_stamps_available -= 1;

            self.last_purchase = Some(PurchaseRecord {
                dest_id: 0,
                buyer_idx: player_idx,
                price: if free { 0 } else { 100 },
                name: "First Class".to_string(),
                is_first_class: true,
            });

            if free {
            } else {
            }

            self.check_and_handle_win(player_idx);
        } else {
            if !free {
                self.players[player_idx].add_money(100);
            }
        }
    }

    fn handle_bankruptcy(&mut self, player_idx: usize, creditor_idx: Option<usize>) {

        if let Some(stamp) = self.players[player_idx].passport.remove_last_stamp() {
            if let Some(creditor) = creditor_idx {
                let bankrupt_name = self.players[player_idx].name.clone();
                let creditor_name = self.players[creditor].name.clone();
                self.log_action(Some(player_idx), format!("{} went bankrupt and gave their last stamp {} to {}", bankrupt_name, stamp.name, creditor_name));
                self.add_stamp_with_checks(creditor, stamp);
            } else {
                let bankrupt_name = self.players[player_idx].name.clone();
                self.log_action(Some(player_idx), format!("{} went bankrupt and their last stamp {} returned to the board", bankrupt_name, stamp.name));
            }
        } else {
        }

    }

    fn find_destination_owner(&self, dest_id: u8) -> Option<usize> {
        for (i, player) in self.players.iter().enumerate() {
            if player.passport.get_destination_ids().contains(&dest_id) {
                return Some(i);
            }
        }
        None
    }

    fn can_anyone_swap(&self, player_idx: usize) -> bool {
        if self.players[player_idx].passport.stamp_count() == 0 {
            return false;
        }
        self.players.iter().enumerate().any(|(i, p)| i != player_idx && p.passport.stamp_count() > 0)
    }
}