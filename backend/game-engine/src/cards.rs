use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Serialize, Deserialize};

// ============================================================================
// CHANCE CARDS 
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChanceCardAction {
    StealStampAndPay,                    // Ia ultima ștampilă a unui jucător, dar plătește-i valoarea integrală
    PayHospital,                         // Plătește M200 pentru spitalizare
    FirstClassBonus,                     // Colectează M40 pentru fiecare ștampilă First Class (x2)
    CollectFromEachPlayer(u32),          // Toți jucătorii îți plătesc M40
    RerollOneDie,                        // Aruncă din nou unul din zaruri și mută
    GoToJail,                            // Mergi la închisoare
    AdvanceToStart,                      // Avansează la START
    DiceChallenge,                       // Alege un jucător; dați amândoi cu zarul (x2)
    CollectMoney(u32),                   // Colectează M100 de la bancă
    MoveSteps(i32),                      // Înaintează cu 5 spații
    SwapTwoPlayersStamps,                // Doi jucători fac schimb între ultimele lor ștampile
    GetOutOfJailFree,                    // Ieșire gratis din închisoare
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceCard {
    pub id: String,
    pub description: String,
    pub action: ChanceCardAction,
    pub can_keep: bool, // Doar "Get Out of Jail Free" poate fi păstrat
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceDeck {
    pub cards: Vec<ChanceCard>,
    pub discard_pile: Vec<ChanceCard>,
}

impl ChanceDeck {
    pub fn new() -> Self {
        let mut cards = vec![
            ChanceCard {
                id: "chance_steal".to_string(),
                description: "Steal another player's last stamp, but pay them its full value.".to_string(),
                action: ChanceCardAction::StealStampAndPay,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_hospital".to_string(),
                description: "Pay a hospital tax of M200.".to_string(),
                action: ChanceCardAction::PayHospital,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_first_class_1".to_string(),
                description: "Bonus for \"First Class\"! Collect M40 for each \"First Class\" stamp in your passport.".to_string(),
                action: ChanceCardAction::FirstClassBonus,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_first_class_2".to_string(),
                description: "Bonus for \"First Class\"! Collect M40 for each \"First Class\" stamp in your passport.".to_string(),
                action: ChanceCardAction::FirstClassBonus,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_collect_each".to_string(),
                description: "Each player pays you M40.".to_string(),
                action: ChanceCardAction::CollectFromEachPlayer(40),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_reroll".to_string(),
                description: "Reroll one of the dice and move.".to_string(),
                action: ChanceCardAction::RerollOneDie,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_jail".to_string(),
                description: "Go to Jail! Go directly to Jail. Do not pass START. Do not collect M200.".to_string(),
                action: ChanceCardAction::GoToJail,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_start".to_string(),
                description: "Advance to START.".to_string(),
                action: ChanceCardAction::AdvanceToStart,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_dice_challenge_1".to_string(),
                description: "Choose another player; both roll a die. The player with the higher roll pays the other M100.".to_string(),
                action: ChanceCardAction::DiceChallenge,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_dice_challenge_2".to_string(),
                description: "Choose another player; both roll a die. The player with the higher roll pays the other M100.".to_string(),
                action: ChanceCardAction::DiceChallenge,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_collect_100".to_string(),
                description: "Collect M100 from the bank.".to_string(),
                action: ChanceCardAction::CollectMoney(100),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_move_5".to_string(),
                description: "Advance 5 spaces.".to_string(),
                action: ChanceCardAction::MoveSteps(5),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_swap_stamps".to_string(),
                description: "Two players must swap their last stamps (preferably not you).".to_string(),
                action: ChanceCardAction::SwapTwoPlayersStamps,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_get_out_jail".to_string(),
                description: "Get Out of Jail Free!".to_string(),
                action: ChanceCardAction::GetOutOfJailFree,
                can_keep: true,
            },
        ];

        let mut rng = thread_rng();
        cards.shuffle(&mut rng);

        if let Some(pos) = cards.iter().position(|c| c.id == "chance_reroll") {
            let card = cards.remove(pos);
            cards.push(card);
        }

        ChanceDeck {
            cards,
            discard_pile: Vec::new(),
        }
    }

    pub fn draw(&mut self) -> ChanceCard {
        if self.cards.is_empty() {
            self.cards = self.discard_pile.drain(..).collect();
            let mut rng = thread_rng();
            self.cards.shuffle(&mut rng);
        }

        // ✅ dacă încă e gol (toate cărțile sunt în mâinile jucătorilor), refacem pachetul
        if self.cards.is_empty() {
            *self = ChanceDeck::new();
        }

        self.cards.pop().expect("Chance deck is empty!")
    }

    pub fn discard(&mut self, card: ChanceCard) {
        if !card.can_keep {
            self.discard_pile.push(card);
        }
    }
}

// ============================================================================
// HERE & NOW CARDS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HereAndNowCardAction {
    InterceptPurchase,                   // Când un alt jucător cumpără un loc, cumpără-l tu
    SayNo,                               // Contracărează acțiunile unui jucător împotriva ta (x3)
    MoveSteps(i32),                      // Înaintează cu 5 spații
    GetOutOfJailFree,                    // Ieșire gratis din închisoare
    SwapStamps,                          // Schimbă ultima ta ștampilă cu ultima ștampilă a altui jucător (x2)
    TakeAllLastStamps,                   // Ia ultima ștampilă de la TOȚI jucătorii
    DiscountPurchase,                    // Când aterizezi pe un loc nedeținut, plătești doar M100
    CollectFromRichest,                  // Jucătorul cu cele mai multe ștampile îți plătește M200
    StealFirstClass,                     // Fură ștampila First Class a altui jucător când o primește
    MoveAnywhere,                        // Mergi în orice loc de pe tablă
    CollectTax,                          // Colectează o taxă de turist în loc să plătești
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereAndNowCard {
    pub id: String,
    pub description: String,
    pub action: HereAndNowCardAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HereAndNowDeck {
    pub cards: Vec<HereAndNowCard>,
    pub discard_pile: Vec<HereAndNowCard>,
}

impl HereAndNowDeck {
    pub fn new() -> Self {
        let mut cards = vec![
            HereAndNowCard {
                id: "hn_intercept".to_string(),
                description: "Intercept the last property purchased by another player. You pay the price, and they are refunded.".to_string(),
                action: HereAndNowCardAction::InterceptPurchase,
            },
            HereAndNowCard {
                id: "hn_say_no_1".to_string(),
                description: "Just Say No! Counter another player's action against you.".to_string(),
                action: HereAndNowCardAction::SayNo,
            },
            HereAndNowCard {
                id: "hn_say_no_2".to_string(),
                description: "Just Say No! Counter another player's action against you.".to_string(),
                action: HereAndNowCardAction::SayNo,
            },
            HereAndNowCard {
                id: "hn_say_no_3".to_string(),
                description: "Just Say No! Counter another player's action against you.".to_string(),
                action: HereAndNowCardAction::SayNo,
            },
            HereAndNowCard {
                id: "hn_move_5".to_string(),
                description: "Advance 5 spaces.".to_string(),
                action: HereAndNowCardAction::MoveSteps(5),
            },
            HereAndNowCard {
                id: "hn_get_out_jail".to_string(),
                description: "Get Out of Jail Free.".to_string(),
                action: HereAndNowCardAction::GetOutOfJailFree,
            },
            HereAndNowCard {
                id: "hn_swap_stamps_1".to_string(),
                description: "Swap your last stamp with another player's last stamp.".to_string(),
                action: HereAndNowCardAction::SwapStamps,
            },
            HereAndNowCard {
                id: "hn_swap_stamps_2".to_string(),
                description: "Swap your last stamp with another player's last stamp.".to_string(),
                action: HereAndNowCardAction::SwapStamps,
            },
            HereAndNowCard {
                id: "hn_take_all_last".to_string(),
                description: "Take the last stamp from ALL players and put them back on the board.".to_string(),
                action: HereAndNowCardAction::TakeAllLastStamps,
            },
            HereAndNowCard {
                id: "hn_discount".to_string(),
                description: "When you land on an unowned space, pay only M100 for that space.".to_string(),
                action: HereAndNowCardAction::DiscountPurchase,
            },
            HereAndNowCard {
                id: "hn_collect_richest".to_string(),
                description: "The player with the most stamps pays you M200.".to_string(),
                action: HereAndNowCardAction::CollectFromRichest,
            },
            HereAndNowCard {
                id: "hn_steal_first_class".to_string(),
                description: "Steal another player's \"First Class\" stamp when they receive it.".to_string(),
                action: HereAndNowCardAction::StealFirstClass,
            },
            HereAndNowCard {
                id: "hn_move_anywhere".to_string(),
                description: "Go to any space on the board.".to_string(),
                action: HereAndNowCardAction::MoveAnywhere,
            },
            HereAndNowCard {
                id: "hn_collect_tax".to_string(),
                description: "Collect a tourist tax instead of paying for it.".to_string(),
                action: HereAndNowCardAction::CollectTax,
            },
        ];

        let mut rng = thread_rng();
        cards.shuffle(&mut rng);

        HereAndNowDeck {
            cards,
            discard_pile: Vec::new(),
        }
    }

    pub fn draw(&mut self) -> HereAndNowCard {
        if self.cards.is_empty() {
            self.cards = self.discard_pile.drain(..).collect();
            let mut rng = thread_rng();
            self.cards.shuffle(&mut rng);
        }

        // ✅ dacă încă e gol (toate cărțile sunt în mâinile jucătorilor), refacem pachetul
        if self.cards.is_empty() {
            *self = HereAndNowDeck::new();
        }

        self.cards.pop().expect("Here & Now deck is empty!")
    }

    /// Distribuie 2 cărți inițiale fiecărui jucător
    pub fn deal_initial(&mut self) -> (HereAndNowCard, HereAndNowCard) {
        let card1 = self.draw();
        let card2 = self.draw();
        (card1, card2)
    }

    pub fn discard(&mut self, card: HereAndNowCard) {
        self.discard_pile.push(card);
    }
}