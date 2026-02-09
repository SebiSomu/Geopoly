use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Serialize, Deserialize};

// ============================================================================
// CHANCE CARDS (Șansă)
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
                description: "Ia ultima ștampilă a unui jucător, dar plătește-i valoarea integrală a acesteia.".to_string(),
                action: ChanceCardAction::StealStampAndPay,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_hospital".to_string(),
                description: "Plătește o taxă de spitalizare de M200.".to_string(),
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
                description: "Toți jucătorii îți plătesc M40.".to_string(),
                action: ChanceCardAction::CollectFromEachPlayer(40),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_reroll".to_string(),
                description: "Aruncă din nou unul din zaruri și mută.".to_string(),
                action: ChanceCardAction::RerollOneDie,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_jail".to_string(),
                description: "Mergi la Închisoare! Nu treci pe la START. Nu colectezi M200.".to_string(),
                action: ChanceCardAction::GoToJail,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_start".to_string(),
                description: "Avansează la START.".to_string(),
                action: ChanceCardAction::AdvanceToStart,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_dice_challenge_1".to_string(),
                description: "Alege un alt jucător; dați amândoi cu zarul. Jucătorul care dă zarul cel mai mare îi plătește celuilalt M100.".to_string(),
                action: ChanceCardAction::DiceChallenge,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_dice_challenge_2".to_string(),
                description: "Alege un alt jucător; dați amândoi cu zarul. Jucătorul care dă zarul cel mai mare îi plătește celuilalt M100.".to_string(),
                action: ChanceCardAction::DiceChallenge,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_collect_100".to_string(),
                description: "Colectează M100 de la bancă.".to_string(),
                action: ChanceCardAction::CollectMoney(100),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_move_5".to_string(),
                description: "Înaintează cu 5 spații.".to_string(),
                action: ChanceCardAction::MoveSteps(5),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_swap_stamps".to_string(),
                description: "Doi jucători trebuie să facă schimb între ultimele lor ștampile (de preferat, să nu fii tu acel jucător).".to_string(),
                action: ChanceCardAction::SwapTwoPlayersStamps,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_get_out_jail".to_string(),
                description: "Ieșire gratis din închisoare!".to_string(),
                action: ChanceCardAction::GetOutOfJailFree,
                can_keep: true, // POATE FI PĂSTRAT
            },
        ];

        let mut rng = thread_rng();
        cards.shuffle(&mut rng);

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

        self.cards.pop().expect("Deck-ul Here&Now este gol!")
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
                description: "Atunci când un alt jucător este pe cale să cumpere un loc, cumpără-l tu.".to_string(),
                action: HereAndNowCardAction::InterceptPurchase,
            },
            HereAndNowCard {
                id: "hn_say_no_1".to_string(),
                description: "Spune nu! Contracărează acțiunile unui jucător împotriva ta.".to_string(),
                action: HereAndNowCardAction::SayNo,
            },
            HereAndNowCard {
                id: "hn_say_no_2".to_string(),
                description: "Spune nu! Contracărează acțiunile unui jucător împotriva ta.".to_string(),
                action: HereAndNowCardAction::SayNo,
            },
            HereAndNowCard {
                id: "hn_say_no_3".to_string(),
                description: "Spune nu! Contracărează acțiunile unui jucător împotriva ta.".to_string(),
                action: HereAndNowCardAction::SayNo,
            },
            HereAndNowCard {
                id: "hn_move_5".to_string(),
                description: "Înaintează cu 5 spații.".to_string(),
                action: HereAndNowCardAction::MoveSteps(5),
            },
            HereAndNowCard {
                id: "hn_get_out_jail".to_string(),
                description: "Ieșire gratis din închisoare.".to_string(),
                action: HereAndNowCardAction::GetOutOfJailFree,
            },
            HereAndNowCard {
                id: "hn_swap_stamps_1".to_string(),
                description: "Schimbă ultima ta ștampilă cu ultima ștampilă a altui jucător.".to_string(),
                action: HereAndNowCardAction::SwapStamps,
            },
            HereAndNowCard {
                id: "hn_swap_stamps_2".to_string(),
                description: "Schimbă ultima ta ștampilă cu ultima ștampilă a altui jucător.".to_string(),
                action: HereAndNowCardAction::SwapStamps,
            },
            HereAndNowCard {
                id: "hn_take_all_last".to_string(),
                description: "Ia ultima ștampilă de la TOȚI jucătorii și pune-le din nou pe tablă.".to_string(),
                action: HereAndNowCardAction::TakeAllLastStamps,
            },
            HereAndNowCard {
                id: "hn_discount".to_string(),
                description: "Când aterizezi pe un loc care nu este deținut de nimeni, plătești doar M100 pentru acel loc.".to_string(),
                action: HereAndNowCardAction::DiscountPurchase,
            },
            HereAndNowCard {
                id: "hn_collect_richest".to_string(),
                description: "Jucătorul cu cele mai multe ștampile îți plătește M200.".to_string(),
                action: HereAndNowCardAction::CollectFromRichest,
            },
            HereAndNowCard {
                id: "hn_steal_first_class".to_string(),
                description: "Steal another player's \"First Class\" stamp when they receive it.".to_string(),
                action: HereAndNowCardAction::StealFirstClass,
            },
            HereAndNowCard {
                id: "hn_move_anywhere".to_string(),
                description: "Mergi în orice loc de pe tablă.".to_string(),
                action: HereAndNowCardAction::MoveAnywhere,
            },
            HereAndNowCard {
                id: "hn_collect_tax".to_string(),
                description: "Colectează o taxă de turist în loc să plătești pentru ea.".to_string(),
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

        self.cards.pop().expect("Deck-ul Here&Now este gol!")
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