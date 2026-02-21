use crate::passport::Passport;
use crate::cards::HereAndNowCard;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub money: i32,
    pub position: usize,
    pub passport: Passport,
    pub in_jail: bool,
    pub jail_turns: u8,
    pub get_out_of_jail_free: bool, // (Șansă) - păstrabil (kept for compatibility)

    pub here_and_now_cards: Vec<HereAndNowCard>, // în mână, păstrabile oricând
    pub chance_cards: Vec<crate::cards::ChanceCard>, // în mână, păstrabile (ex: Ieșire închisoare)

    // Efecte Here&Now "armate" / păstrate ca stare (simplificare console)
    pub say_no_cards: u8,
    pub intercept_purchase_ready: bool,
    pub discount_purchase_ready: bool,
    pub collect_tax_ready: bool,
    pub steal_first_class_ready: bool,

    pub consecutive_doubles: u8, // Tracks consecutive doubles for jail rule

    pub double_rent_active: bool, // păstrat din vechiul engine (nu strică)
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            name,
            money: 1500,
            position: 0,
            passport: Passport::new(),
            in_jail: false,
            jail_turns: 0,
            get_out_of_jail_free: false,
            here_and_now_cards: Vec::new(),
            chance_cards: Vec::new(),

            say_no_cards: 0,
            intercept_purchase_ready: false,
            discount_purchase_ready: false,
            collect_tax_ready: false,
            steal_first_class_ready: false,

            consecutive_doubles: 0,
            double_rent_active: false,
        }
    }

    pub fn add_money(&mut self, amount: i32) {
        self.money += amount;
    }

    pub fn pay_money(&mut self, amount: i32) -> bool {
        if self.money >= amount {
            self.money -= amount;
            true
        } else {
            false
        }
    }

    pub fn move_to(&mut self, position: usize) {
        self.position = position;
    }

    pub fn move_by(&mut self, steps: i32, board_size: usize) -> bool {
        let old_position = self.position;

        if steps >= 0 {
            self.position = (self.position + steps as usize) % board_size;
        } else {
            let abs_steps = steps.abs() as usize;
            if abs_steps > self.position {
                self.position = board_size - (abs_steps - self.position);
            } else {
                self.position -= abs_steps;
            }
        }

        old_position > self.position && steps > 0
    }

    pub fn send_to_jail(&mut self) {
        self.in_jail = true;
        self.jail_turns = 0;
        self.position = 10;
        self.consecutive_doubles = 0;
    }

    pub fn release_from_jail(&mut self) {
        self.in_jail = false;
        self.jail_turns = 0;
    }

    pub fn display_status(&self) {
        println!("👤 {}", self.name);
        println!("  💰 Bani: M{}", self.money);
        println!("  📍 Poziție: {}", self.position);

        if self.in_jail {
            println!("  🔒 ÎN ÎNCHISOARE (tura {})", self.jail_turns + 1);
        }
        if self.get_out_of_jail_free {
            println!("  🎫 Deține cartonaș 'Ieșire Gratuită din Închisoare' (Șansă)");
        }

        if self.say_no_cards > 0 {
            println!("  🛑 'Spune nu!' disponibile: {}", self.say_no_cards);
        }
        if self.intercept_purchase_ready {
            println!("  🎯 Intercept Purchase: ACTIV");
        }
        if self.discount_purchase_ready {
            println!("  💸 Discount Purchase: ACTIV");
        }
        if self.collect_tax_ready {
            println!("  🧾 Collect Tax: ACTIV");
        }
        if self.steal_first_class_ready {
            println!("  ✈️ Steal First Class: ACTIV");
        }

        if !self.here_and_now_cards.is_empty() {
            println!("  🎴 Cartonașe Here&Now în mână: {}", self.here_and_now_cards.len());
        }

        self.passport.display();
        println!();
    }

    pub fn has_color_set(&self, color_set: &[u8]) -> bool {
        let owned_ids = self.passport.get_destination_ids();
        color_set.iter().all(|id| owned_ids.contains(id))
    }
}