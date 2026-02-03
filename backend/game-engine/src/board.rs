use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Yellow,
    Red,
    Green,
    DarkBlue,
}

#[derive(Debug, Clone)]
pub struct Destination {
    pub id: u8,
    pub name: String,
    pub price: u32,
    pub tourist_tax: u32,
    pub color: Color,
    pub stamp_diameter: f32, // în cm
}

#[derive(Debug, Clone)]
pub enum Space {
    Start,
    Destination(Destination),
    FirstClass,
    Airport,
    HereAndNow,
    Chance,
    FreeParking,
    GoToJail,
    JustVisiting,
}

pub struct Board {
    pub spaces: Vec<Space>,
    pub color_sets: HashMap<Color, Vec<u8>>, // culoare -> lista de ID-uri destinații
}

impl Board {
    pub fn new() -> Self {
        let mut spaces = Vec::new();

        // Poziția 0: START
        spaces.push(Space::Start);

        // Vom construi tabla în sensul acelor de ceasornic
        // 40 spații total (ca în Monopoly clasic)

        // Destinații conform documentului
        let destinations = vec![
            Destination {
                id: 22,
                name: "Madrid".to_string(),
                price: 60,
                tourist_tax: 40,
                color: Color::Brown,
                stamp_diameter: 1.5,
            },
            Destination {
                id: 21,
                name: "Giethoorn".to_string(),
                price: 60,
                tourist_tax: 40,
                color: Color::Brown,
                stamp_diameter: 1.5,
            },
            Destination {
                id: 20,
                name: "Taipei".to_string(),
                price: 100,
                tourist_tax: 60,
                color: Color::LightBlue,
                stamp_diameter: 1.6,
            },
            Destination {
                id: 19,
                name: "Cape Town".to_string(),
                price: 100,
                tourist_tax: 60,
                color: Color::LightBlue,
                stamp_diameter: 1.6,
            },
            Destination {
                id: 18,
                name: "Queenstown".to_string(),
                price: 100,
                tourist_tax: 60,
                color: Color::LightBlue,
                stamp_diameter: 1.6,
            },
            Destination {
                id: 17,
                name: "Sydney".to_string(),
                price: 160,
                tourist_tax: 100,
                color: Color::Pink,
                stamp_diameter: 1.8,
            },
            Destination {
                id: 16,
                name: "Amsterdam".to_string(),
                price: 160,
                tourist_tax: 100,
                color: Color::Pink,
                stamp_diameter: 1.8,
            },
            Destination {
                id: 15,
                name: "New York".to_string(),
                price: 160,
                tourist_tax: 100,
                color: Color::Pink,
                stamp_diameter: 1.8,
            },
            Destination {
                id: 14,
                name: "Tokyo".to_string(),
                price: 200,
                tourist_tax: 120,
                color: Color::Orange,
                stamp_diameter: 1.85,
            },
            Destination {
                id: 13,
                name: "Moscova".to_string(),
                price: 200,
                tourist_tax: 120,
                color: Color::Orange,
                stamp_diameter: 1.85,
            },
            Destination {
                id: 12,
                name: "Londra".to_string(),
                price: 200,
                tourist_tax: 120,
                color: Color::Orange,
                stamp_diameter: 1.85,
            },
            Destination {
                id: 11,
                name: "Belgrad".to_string(),
                price: 260,
                tourist_tax: 140,
                color: Color::Red,
                stamp_diameter: 2.1,
            },
            Destination {
                id: 10,
                name: "Atena".to_string(),
                price: 260,
                tourist_tax: 140,
                color: Color::Red,
                stamp_diameter: 2.1,
            },
            Destination {
                id: 9,
                name: "Belfast".to_string(),
                price: 260,
                tourist_tax: 140,
                color: Color::Red,
                stamp_diameter: 2.1,
            },
            Destination {
                id: 8,
                name: "Santiago".to_string(),
                price: 300,
                tourist_tax: 180,
                color: Color::Yellow,
                stamp_diameter: 2.2,
            },
            Destination {
                id: 7,
                name: "Mexico City".to_string(),
                price: 300,
                tourist_tax: 180,
                color: Color::Yellow,
                stamp_diameter: 2.2,
            },
            Destination {
                id: 6,
                name: "Varsovia".to_string(),
                price: 300,
                tourist_tax: 180,
                color: Color::Yellow,
                stamp_diameter: 2.2,
            },
            Destination {
                id: 5,
                name: "Istanbul".to_string(),
                price: 360,
                tourist_tax: 200,
                color: Color::Green,
                stamp_diameter: 2.45,
            },
            Destination {
                id: 4,
                name: "Lisabona".to_string(),
                price: 360,
                tourist_tax: 200,
                color: Color::Green,
                stamp_diameter: 2.45,
            },
            Destination {
                id: 3,
                name: "Riga".to_string(),
                price: 360,
                tourist_tax: 200,
                color: Color::Green,
                stamp_diameter: 2.45,
            },
            Destination {
                id: 2,
                name: "Hong Kong".to_string(),
                price: 400,
                tourist_tax: 240,
                color: Color::DarkBlue,
                stamp_diameter: 2.5,
            },
            Destination {
                id: 1,
                name: "Lima".to_string(),
                price: 400,
                tourist_tax: 240,
                color: Color::DarkBlue,
                stamp_diameter: 2.5,
            },
        ];

        // Construim tabla cu 40 spații (similar Monopoly clasic)
        // Poziționăm destinațiile și spațiile speciale strategic

        spaces.push(Space::Destination(destinations[0].clone())); // Madrid
        spaces.push(Space::Chance);
        spaces.push(Space::Destination(destinations[1].clone())); // Giethoorn
        spaces.push(Space::Airport);
        spaces.push(Space::HereAndNow);

        spaces.push(Space::Destination(destinations[2].clone())); // Taipei
        spaces.push(Space::FirstClass);
        spaces.push(Space::Destination(destinations[3].clone())); // Cape Town
        spaces.push(Space::Destination(destinations[4].clone())); // Queenstown

        spaces.push(Space::JustVisiting); // Poziția 10

        spaces.push(Space::Destination(destinations[5].clone())); // Sydney
        spaces.push(Space::Chance);
        spaces.push(Space::Destination(destinations[6].clone())); // Amsterdam
        spaces.push(Space::Destination(destinations[7].clone())); // New York
        spaces.push(Space::HereAndNow);

        spaces.push(Space::Destination(destinations[8].clone())); // Tokyo
        spaces.push(Space::FirstClass);
        spaces.push(Space::Destination(destinations[9].clone())); // Moscova
        spaces.push(Space::Destination(destinations[10].clone())); // Londra

        spaces.push(Space::FreeParking); // Poziția 20

        spaces.push(Space::Destination(destinations[11].clone())); // Belgrad
        spaces.push(Space::Chance);
        spaces.push(Space::Destination(destinations[12].clone())); // Atena
        spaces.push(Space::Destination(destinations[13].clone())); // Belfast
        spaces.push(Space::HereAndNow);

        spaces.push(Space::Destination(destinations[14].clone())); // Santiago
        spaces.push(Space::Destination(destinations[15].clone())); // Mexico City
        spaces.push(Space::FirstClass);
        spaces.push(Space::Destination(destinations[16].clone())); // Varsovia

        spaces.push(Space::GoToJail); // Poziția 30

        spaces.push(Space::Destination(destinations[17].clone())); // Istanbul
        spaces.push(Space::Destination(destinations[18].clone())); // Lisabona
        spaces.push(Space::Chance);
        spaces.push(Space::Destination(destinations[19].clone())); // Riga
        spaces.push(Space::HereAndNow);

        spaces.push(Space::Airport);
        spaces.push(Space::Destination(destinations[20].clone())); // Hong Kong
        spaces.push(Space::FirstClass);
        spaces.push(Space::Destination(destinations[21].clone())); // Lima

        // Construim seturile de culori pentru bonus-uri
        let mut color_sets = HashMap::new();
        for dest in &destinations {
            color_sets.entry(dest.color.clone())
                .or_insert_with(Vec::new)
                .push(dest.id);
        }

        Board { spaces, color_sets }
    }

    pub fn get_space(&self, position: usize) -> &Space {
        &self.spaces[position % self.spaces.len()]
    }

    pub fn total_spaces(&self) -> usize {
        self.spaces.len()
    }

    /// Găsește o destinație după ID
    pub fn find_destination_by_id(&self, dest_id: u8) -> Option<&Destination> {
        for space in &self.spaces {
            if let Space::Destination(dest) = space {
                if dest.id == dest_id {
                    return Some(dest);
                }
            }
        }
        None
    }
}