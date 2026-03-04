use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Serialize, Deserialize};

// ============================================================================
// CHANCE CARDS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChanceCardAction {
    StealStampAndPay,                    // Take a player's last stamp, but pay them its full value
    PayHospital,                         // Pay G200 for hospitalization
    FirstClassBonus,                     // Collect G40 for each First Class stamp (x2)
    CollectFromEachPlayer(u32),          // All players pay you G40
    RerollOneDice,                        // Reroll one of the dice and move
    GoToJail,                            // Go to Jail
    AdvanceToStart,                      // Advance to START
    DiceChallenge,                       // Choose a player; both roll a die (x2)
    CollectMoney(u32),                   // Collect G100 from bank
    MoveSteps(i32),                      // Advance 5 spaces
    SwapTwoPlayersStamps,                // Two players swap their last stamps
    GetOutOfJailFree,                    // Get out of jail free
    GoToFreeParking,                     // Go to Free Parking (pos 20), skip turn, collect last 3 taxes paid
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChanceCard {
    pub id: String,
    pub description: String,
    pub action: ChanceCardAction,
    pub can_keep: bool, // Only for "get out of jail"
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
                description: "Pay a hospital tax of G200.".to_string(),
                action: ChanceCardAction::PayHospital,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_first_class_1".to_string(),
                description: "Bonus for \"First Class\"! Collect G40 for each \"First Class\" stamp in your passport.".to_string(),
                action: ChanceCardAction::FirstClassBonus,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_first_class_2".to_string(),
                description: "Bonus for \"First Class\"! Collect G40 for each \"First Class\" stamp in your passport.".to_string(),
                action: ChanceCardAction::FirstClassBonus,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_collect_each".to_string(),
                description: "Each player pays you G40.".to_string(),
                action: ChanceCardAction::CollectFromEachPlayer(40),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_reroll".to_string(),
                description: "Reroll one of the dice and move.".to_string(),
                action: ChanceCardAction::RerollOneDice,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_jail".to_string(),
                description: "Go to Jail! Go directly to Jail. Do not pass START. Do not collect G200.".to_string(),
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
                description: "Choose another player; both roll a die. The player with the higher roll pays the other G100.".to_string(),
                action: ChanceCardAction::DiceChallenge,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_dice_challenge_2".to_string(),
                description: "Choose another player; both roll a die. The player with the higher roll pays the other G100.".to_string(),
                action: ChanceCardAction::DiceChallenge,
                can_keep: false,
            },
            ChanceCard {
                id: "chance_collect_100".to_string(),
                description: "Collect G100 from the bank.".to_string(),
                action: ChanceCardAction::CollectMoney(100),
                can_keep: false,
            },
            ChanceCard {
                id: "chance_collect_100_2".to_string(),
                description: "Collect G100 from the bank.".to_string(),
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
            ChanceCard {
                id: "chance_free_parking".to_string(),
                description: "Go to Free Parking. Skip your next turn. Do not collect G200. Collect your last 3 tourist taxes paid.".to_string(),
                action: ChanceCardAction::GoToFreeParking,
                can_keep: false,
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

        // ✅ if still empty (all cards are in players' hands), rebuild the deck
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
    InterceptPurchase,                   // When another player buys a space, you buy it
    SayNo,                               // Counters a player's actions against you (x3)
    MoveSteps(i32),                      // Advance 5 spaces
    GetOutOfJailFree,                    // Get out of jail free
    SwapStamps,                          // Swap your last stamp with another player's last stamp (x2)
    TakeAllLastStamps,                   // Take the last stamp from ALL players
    DiscountPurchase,                    // When landing on an unowned space, pay only G100
    CollectFromRichest,                  // The player with the most stamps pays you G200
    StealFirstClass,                     // Steal another player's First Class stamp when they receive it
    MoveAnywhere,                        // Go to any space on the board
    CollectTax,                          // Collect a tourist tax instead of paying
    StampAmnesty,                        // Sell any stamp from your passport to the bank for 150% of original price
    BlockNextDouble,                     // Block a chosen player: on next double they move but their turn ends
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
                description: "When you land on an unowned space, pay only G100 for that space.".to_string(),
                action: HereAndNowCardAction::DiscountPurchase,
            },
            HereAndNowCard {
                id: "hn_collect_richest".to_string(),
                description: "The player with the most stamps pays you G200.".to_string(),
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
            HereAndNowCard {
                id: "hn_stamp_amnesty".to_string(),
                description: "Stamp Amnesty! Sell any stamp from your passport back to the bank for 150% of its original price.".to_string(),
                action: HereAndNowCardAction::StampAmnesty,
            },
            HereAndNowCard {
                id: "hn_block_double".to_string(),
                description: "Block someone's next double dice roll".to_string(),
                action: HereAndNowCardAction::BlockNextDouble,
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

        if self.cards.is_empty() {
            *self = HereAndNowDeck::new();
        }

        self.cards.pop().expect("Here & Now deck is empty!")
    }

    pub fn deal_initial(&mut self) -> (HereAndNowCard, HereAndNowCard) {
        let card1 = self.draw();
        let card2 = self.draw();
        (card1, card2)
    }

    pub fn discard(&mut self, card: HereAndNowCard) {
        self.discard_pile.push(card);
    }
}