use crate::board::{Board, Space, Destination};
use crate::player::Player;
use crate::cards::{ChanceDeck, HereAndNowDeck, ChanceCardAction, HereAndNowCardAction};
use crate::passport::Stamp;
use colored::*;
use rand::Rng;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiceResult {
    Normal(u8, u8),
    Double(u8),
    BusinessDeal
}

pub struct Game {
    pub board: Board,
    pub players: Vec<Player>,
    pub current_player_idx: usize,
    pub chance_deck: ChanceDeck,
    pub here_and_now_deck: HereAndNowDeck,
    pub first_class_stamps_available: u8,
    pub turn_number: u32,
    pub game_over: bool
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
            game_over: false
        }
    }

    pub fn play(&mut self) {
        while !self.game_over {
            // Afișăm numărul turei doar la începutul fiecărui ciclu (când joacă primul jucător)
            if self.current_player_idx == 0 {
                println!("\n{}", "=".repeat(60).on_bright_blue().white());
                println!("{}  ÎNCEPE TURA GLOBALĂ #{}  {}", "🌍".yellow(), self.turn_number, "🌍".yellow());
                println!("{}", "=".repeat(60).on_bright_blue().white());
            }

            let player_name = self.players[self.current_player_idx].name.clone();
            let in_jail = self.players[self.current_player_idx].in_jail;

            println!("\n{} {}", "Rândul jucătorului:".green().bold(), player_name.yellow().bold());

            self.display_current_player_status();

            // Verificare câștig
            if self.check_and_handle_win(self.current_player_idx) {
                break;
            }

            if in_jail {
                self.handle_jail_turn();
            } else {
                self.take_turn();
            }

            if self.game_over {
                break;
            }

            self.wait_for_enter();

            // Logica de incrementare a turei:
            // Dacă jucătorul curent este ultimul din listă, înseamnă că tura globală s-a terminat.
            if self.current_player_idx == self.players.len() - 1 {
                self.turn_number += 1;
            }

            // Trecem la următorul jucător
            self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
        }
    }

    fn take_turn(&mut self) {
        let mut doubles_count: u8 = 0;
        let mut roll_number: u8 = 0;

        loop {
            roll_number += 1;

            if roll_number == 1 {
                println!("\n{}", "Aruncă zarurile...".cyan());
            } else {
                println!("\n{}", "Aruncă din nou zarurile (după dublă)...".cyan());
            }

            let dice_result = Self::roll_dice();
            self.display_dice_result(&dice_result);

            match dice_result {
                DiceResult::BusinessDeal => {
                    println!(
                        "\n{}",
                        "💼 AFACERE FORȚATĂ! Poți schimba ultima stampilă cu a unui adversar!"
                            .bright_magenta()
                    );
                    self.handle_business_deal();

                    if self.game_over {
                        return;
                    }
                    break; // tura se termină după BusinessDeal
                }

                DiceResult::Normal(d1, d2) => {
                    let total = d1 + d2;
                    self.move_player(total as i32);
                    self.handle_landing();

                    if self.game_over {
                        return;
                    }
                    break; // fără dublă => tura se termină
                }

                DiceResult::Double(value) => {
                    doubles_count += 1;

                    // A treia dublă => direct la închisoare, fără mutare
                    if doubles_count >= 3 {
                        println!(
                            "\n{}",
                            "🚫 A TREIA DUBLĂ! Mergi direct la ÎNCHISOARE! (tura se termină)"
                                .red()
                                .bold()
                        );
                        self.players[self.current_player_idx].send_to_jail();
                        break;
                    }

                    let total = value * 2;
                    println!(
                        "\n{}",
                        format!(
                            "Dublă #{}! Mută {} spații și mai arunci o dată!",
                            doubles_count, total
                        )
                            .bright_green()
                    );

                    self.move_player(total as i32);
                    self.handle_landing();

                    // dacă a câștigat după mutare, nu mai aruncă
                    if self.game_over {
                        return;
                    }

                    // dacă a ajuns la închisoare din mutare (ex: Go To Jail), tura se oprește
                    if self.players[self.current_player_idx].in_jail {
                        println!(
                            "{}",
                            "Tura se încheie deoarece ai ajuns la ÎNCHISOARE.".yellow()
                        );
                        break;
                    }

                    // altfel, continuă bucla pentru încă o aruncare
                    continue;
                }
            }
        }
    }

    fn handle_jail_turn(&mut self) {
        let player = &mut self.players[self.current_player_idx];
        player.jail_turns += 1;

        println!("\n{}", "🔒 Ești în ÎNCHISOARE!".red().bold());
        println!("Opțiuni:");
        println!("1. Plătește M100 și ieși");
        if player.get_out_of_jail_free {
            println!("2. Folosește cartonașul 'Ieșire Gratuită din Închisoare'");
        }
        println!("3. Încearcă să dai dublă ({}/ 3 încercări)", player.jail_turns);

        if player.jail_turns >= 3 {
            println!("\n{}", "Ai stat 3 ture în închisoare! Trebuie să plătești M100.".yellow());
            if player.pay_money(100) {
                println!("{}", "Ai plătit M100. Ești liber!".green());
                player.release_from_jail();
                self.take_turn();
                if self.game_over {
                    return;
                }
            } else {
                println!("{}", "Nu ai suficienți bani! Dai faliment...".red());
                self.handle_bankruptcy(self.current_player_idx, None);
            }
            return;
        }

        // Simplificat: aruncăm zarurile automat pentru a încerca dublă
        println!("\n{}", "Încerci să dai dublă...".cyan());
        let dice_result = Self::roll_dice();

        if let DiceResult::Double(_) = dice_result {
            println!("\n{}", "🎉 Ai dat dublă! Ești liber!".bright_green());
            player.release_from_jail();
            self.display_dice_result(&dice_result);

            if let DiceResult::Double(value) = dice_result {
                self.move_player((value * 2) as i32);
                self.handle_landing();
            }
        } else {
            println!("\n{}", "Nu ai dat dublă. Rămâi în închisoare.".yellow());
        }
    }

    fn roll_dice() -> DiceResult {
        let mut rng = rand::thread_rng();
        let business_deal_chance = rng.gen_range(1..=12);
        if business_deal_chance == 1 {
            return DiceResult::BusinessDeal;
        }
        let d1 = rng.gen_range(1..=6);
        let d2 = rng.gen_range(1..=6);
        if d1 == d2 { DiceResult::Double(d1) } else { DiceResult::Normal(d1, d2) }
    }

    fn display_dice_result(&self, result: &DiceResult) {
        match result {
            DiceResult::Normal(d1, d2) => {
                println!("🎲 {} + {} = {}", d1, d2, d1 + d2);
            }
            DiceResult::Double(value) => {
                println!("🎲🎲 DUBLĂ! {} + {} = {}", value, value, value * 2);
            }
            DiceResult::BusinessDeal => {
                println!("💼 AFACERE FORȚATĂ!");
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
                    self.players[self.current_player_idx].here_and_now_cards.retain(|c| {
                        !matches!(c.action, HereAndNowCardAction::CollectTax)
                    });

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
                self.players[self.current_player_idx].here_and_now_cards.retain(|c| {
                    !matches!(c.action, HereAndNowCardAction::DiscountPurchase)
                });
            }

            // ✅ INTERCEPT PURCHASE: alt jucător interceptează cumpărarea
            for i in 0..self.players.len() {
                if i != self.current_player_idx && self.players[i].intercept_purchase_ready {
                    if self.players[i].money >= final_price {
                        println!("{}", format!("🎯 {} INTERCEPTEAZĂ cumpărarea cu cardul Here&Now!", self.players[i].name).bright_magenta());
                        buyer_idx = i;

                        self.players[i].intercept_purchase_ready = false;

                        // Găsim și ștergem cardul din mână
                        self.players[i].here_and_now_cards.retain(|c| {
                            !matches!(c.action, HereAndNowCardAction::InterceptPurchase)
                        });

                        break;
                    }
                }
            }

            println!("Vrei să o cumperi pentru M{}? (y/n)", final_price);

            // Simplificat: cumpărăm automat dacă avem bani
            if self.players[buyer_idx].money >= final_price {
                println!("Auto: Da, {} cumpără!", self.players[buyer_idx].name);

                if self.players[buyer_idx].pay_money(final_price) {
                    let stamp = Stamp::from_destination(&dest);

                    // ✅ ștampila se pune ORICUM
                    self.players[buyer_idx].passport.add_stamp(stamp);
                    println!("{}", format!("✅ {} a cumpărat destinația și a primit stampila!", self.players[buyer_idx].name).green());

                    // Verifică câștig imediat după cumpărare
                    if self.check_and_handle_win(buyer_idx) {
                        return;
                    }

                    // ✅ verificăm setul de culoare (DOAR dacă jocul continuă)
                    if let Some(color_set) = self.board.color_sets.get(&dest.color) {
                        if self.players[buyer_idx].has_color_set(color_set) {
                            println!("{}", "🎉 Ai completat setul de culoare! Primești o stampilă Clasa Întâi GRATIS!".bright_green());
                            self.give_first_class_stamp(buyer_idx, true);
                            if self.game_over {
                                return;
                            }
                        }
                    }
                }
            } else {
                println!("Auto: Nu am destui bani. Se organizează licitație!");
                self.handle_auction(&dest);
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

        if self.players[idx].pay_money(100) {
            let stamp = Stamp::first_class();

            // ✅ STEAL FIRST CLASS: alt jucător fură ștampila
            let mut stealer_idx: Option<usize> = None;
            for i in 0..self.players.len() {
                if i != idx && self.players[i].steal_first_class_ready {
                    println!("{}", format!("✈️ {} FURĂ ștampila First Class cu cardul Here&Now!", self.players[i].name).bright_magenta());
                    stealer_idx = Some(i);

                    self.players[i].steal_first_class_ready = false;

                    // Găsim și ștergem cardul din mână
                    self.players[i].here_and_now_cards.retain(|c| {
                        !matches!(c.action, HereAndNowCardAction::StealFirstClass)
                    });

                    break;
                }
            }

            let final_idx = stealer_idx.unwrap_or(idx);

            self.players[final_idx].passport.add_stamp(stamp);
            self.first_class_stamps_available -= 1;

            if stealer_idx.is_some() {
                println!("{}", format!("✅ {} a furat stampila Clasa Întâi!", self.players[final_idx].name).bright_green());
            } else {
                println!("{}", "✅ Ai cumpărat stampila Clasa Întâi!".green());
            }

            self.check_and_handle_win(final_idx);
        }
    }

    fn handle_airport(&mut self) {
        println!("✈️ AEROPORT - Poți zbura oriunde pentru M100");

        // Simplificat: nu zburăm automat
        println!("Auto: Rămân aici.");
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

        match card.action {
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
                // numărăm ștampilele FirstClass din pașaport
                let count = self.players[idx]
                    .passport
                    .all_stamps()
                    .iter()
                    .filter(|s| s.name == "FIRST CLASS")
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
                            // dacă nu poate plăti, îl lăsăm simplificat (ca restul engine-ului)
                            println!("{}", format!("{} nu are bani suficienți!", self.players[i].name).yellow());
                        }
                    }
                }
            }

            ChanceCardAction::MoveSteps(steps) => {
                self.move_player(steps);
                self.handle_landing();
                if self.game_over { return; }
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
                self.players[idx].get_out_of_jail_free = true;
                println!("{}", "Primești cartonașul 'Ieșire gratis din închisoare' (Șansă) și îl păstrezi!".bright_green());
                // NU discard
                return;
            }

            ChanceCardAction::RerollOneDie => {
                // simplificare: reroll 1 die => mutăm cu 1..6
                let mut rng = rand::thread_rng();
                let d = rng.gen_range(1..=6);
                println!("{}", format!("Arunci încă un zar: {}. Te muți {} spații.", d, d).cyan());
                self.move_player(d);
                self.handle_landing();
                if self.game_over { return; }
            }

            ChanceCardAction::DiceChallenge => {
                // simplificat: alegem primul adversar disponibil
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
                        println!("{}", "Tu plătești M100 adversarului.".yellow());
                        if self.players[idx].pay_money(100) {
                            self.players[opp].add_money(100);
                        } else {
                            self.handle_bankruptcy(idx, Some(opp));
                        }
                    } else {
                        println!("{}", "Egalitate! Nimeni nu plătește.".yellow());
                    }
                }
            }

            ChanceCardAction::StealStampAndPay => {
                // alegem un adversar cu ștampile
                let opp = self.players.iter().enumerate()
                    .find(|(i, p)| *i != idx && p.passport.stamp_count() > 0)
                    .map(|(i, _)| i);

                if let Some(opp_idx) = opp {
                    if let Some(stolen) = self.players[opp_idx].passport.remove_last_stamp() {
                        // "valoarea integrală" = price-ul destinației, iar FirstClass = 100
                        let price = if stolen.name == "FIRST CLASS" {
                            100
                        } else if let Some(dest_id) = stolen.destination_id {
                            self.board
                                .find_destination_by_id(dest_id)
                                .map(|d| d.price)
                                .unwrap_or(100)
                        } else {
                            100
                        };

                        println!("{}", format!("Furi '{}' de la {} și plătești M{}.", stolen.name, self.players[opp_idx].name, price).bright_magenta());

                        if self.players[idx].pay_money(price) {
                            self.players[opp_idx].add_money(price);
                            self.players[idx].passport.add_stamp(stolen);
                            self.check_and_handle_win(idx);
                            if self.game_over { return; }
                        } else {
                            println!("{}", "Nu ai bani să plătești valoarea! Acțiunea eșuează.".red());
                            // punem ștampila la loc
                            self.players[opp_idx].passport.add_stamp(stolen);
                        }
                    }
                } else {
                    println!("{}", "Niciun adversar nu are ștampile.".yellow());
                }
            }

            ChanceCardAction::SwapTwoPlayersStamps => {
                // simplificat: alegem primii 2 jucători diferiți de tine dacă există
                let mut candidates: Vec<usize> = (0..self.players.len()).filter(|&i| i != idx && self.players[i].passport.stamp_count() > 0).collect();
                if candidates.len() >= 2 {
                    let a = candidates.remove(0);
                    let b = candidates.remove(0);

                    let sa = self.players[a].passport.remove_last_stamp();
                    let sb = self.players[b].passport.remove_last_stamp();

                    if let (Some(sa), Some(sb)) = (sa, sb) {
                        println!("{}", format!("{} și {} își schimbă ultimele ștampile: '{}' <-> '{}'",
                                               self.players[a].name, self.players[b].name, sa.name, sb.name).bright_magenta());

                        self.players[a].passport.add_stamp(sb);
                        self.players[b].passport.add_stamp(sa);

                        if self.check_and_handle_win(a) { return; }
                        if self.check_and_handle_win(b) { return; }
                    }
                } else {
                    println!("{}", "Nu există 2 jucători eligibili pentru schimb.".yellow());
                }
            }
        }

        self.chance_deck.discard(card);
    }

    fn handle_business_deal(&mut self) {
        // Schimbăm ultima stampilă cu a unui adversar
        let current_player_idx = self.current_player_idx;

        if self.players[current_player_idx].passport.stamp_count() == 0 {
            println!("{}", "Nu ai stampile de schimbat!".yellow());
            return;
        }

        // Găsim un adversar cu stampile
        let opponent_idx = self.players.iter()
            .enumerate()
            .find(|(i, p)| *i != current_player_idx && p.passport.stamp_count() > 0)
            .map(|(i, _)| i);

        if let Some(opp_idx) = opponent_idx {
            if let Some(my_stamp) = self.players[current_player_idx].passport.remove_last_stamp() {
                if let Some(opp_stamp) = self.players[opp_idx].passport.remove_last_stamp() {
                    println!("{}", format!("Schimbi '{}' cu '{}' de la {}",
                                           my_stamp.name, opp_stamp.name, self.players[opp_idx].name).bright_magenta());

                    self.players[current_player_idx].passport.add_stamp(opp_stamp);
                    self.players[opp_idx].passport.add_stamp(my_stamp);

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
                let stamp = Stamp::from_destination(dest);
                self.players[i].passport.add_stamp(stamp);

                // Verifică câștig după licitație
                self.check_and_handle_win(i);
                return;
            }
        }

        println!("{}", "Nimeni nu vrea destinația!".yellow());
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

            // Verificare imediată de câștig
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
                println!("{}", format!("Ultima stampilă '{}' merge la {}",
                                       stamp.name, self.players[creditor].name).red());
                self.players[creditor].passport.add_stamp(stamp);
            } else {
                println!("{}", format!("Ultima stampilă '{}' revine pe tablă", stamp.name).red());
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