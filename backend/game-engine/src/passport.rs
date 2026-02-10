use crate::board::Destination;
use serde::{Serialize, Deserialize};

pub const LEFT_COLUMN_HEIGHT: f32 = 8.0; // cm
pub const RIGHT_COLUMN_HEIGHT: f32 = 7.0; // cm
pub const FIRST_CLASS_DIAMETER: f32 = 1.4; // cm
pub const STAMP_OVERLAP_CM: f32 = 0.125; // 5px at 40px/cm

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stamp {
    pub destination_id: Option<u8>, // None pentru Clasa Întâi
    pub diameter: f32,
    pub name: String,
}

impl Stamp {
    pub fn from_destination(dest: &Destination) -> Self {
        Stamp {
            destination_id: Some(dest.id),
            diameter: dest.stamp_diameter,
            name: dest.name.clone(),
        }
    }

    pub fn first_class() -> Self {
        Stamp {
            destination_id: None,
            diameter: FIRST_CLASS_DIAMETER,
            name: "First Class".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passport {
    pub left_column: Vec<Stamp>,
    pub right_column: Vec<Stamp>,
    pub left_height_used: f32,
    pub right_height_used: f32,
    pub overflowed: bool
}

impl Passport {
    pub fn new() -> Self {
        Passport {
            left_column: Vec::new(),
            right_column: Vec::new(),
            left_height_used: 0.0,
            right_height_used: 0.0,
            overflowed: false
        }
    }

    /// Încearcă să adauge o stampilă în pașaport
    /// Returnează true dacă a fost adăugată cu succes
    pub fn add_stamp(&mut self, stamp: Stamp) -> bool {
        // 1️⃣ Încercăm să punem în coloana STÂNGĂ DOAR dacă ÎNCAPE
        // Calculăm înălțimea efectivă adăugată (diametru minus overlap dacă nu e prima)
        let left_overlap = if self.left_column.is_empty() { 0.0 } else { STAMP_OVERLAP_CM };
        if self.left_height_used + stamp.diameter - left_overlap <= LEFT_COLUMN_HEIGHT {
            self.left_height_used += stamp.diameter - left_overlap;
            self.left_column.push(stamp);
            return true;
        }

        // 2️⃣ Altfel, se pune în coloana DREAPTĂ (chiar dacă depășește)
        let right_overlap = if self.right_column.is_empty() { 0.0 } else { STAMP_OVERLAP_CM };
        self.right_height_used += stamp.diameter - right_overlap;
        self.right_column.push(stamp);

        // 3️⃣ Dacă depășim coloana dreaptă → CÂȘTIG
        if self.right_height_used > RIGHT_COLUMN_HEIGHT {
            self.overflowed = true;
        }

        true
    }

    /// Verifică dacă pașaportul este plin (câștigător)
    /// Ultima stampilă trebuie să depășească linia de sus a coloanei drepte
    pub fn is_full(&self) -> bool {
        self.overflowed
    }

    /// Întoarce ultima stampilă adăugată (pentru returnare la bancă/schimb)
    pub fn remove_last_stamp(&mut self) -> Option<Stamp> {
        // Ultima stampilă e fie în dreapta (dacă există), fie în stânga
        if let Some(stamp) = self.right_column.pop() {
            let overlap = if self.right_column.is_empty() { 0.0 } else { STAMP_OVERLAP_CM };
            self.right_height_used -= stamp.diameter - overlap;

            if self.right_height_used <= RIGHT_COLUMN_HEIGHT {
                self.overflowed = false;
            }

            return Some(stamp);
        }

        if let Some(stamp) = self.left_column.pop() {
            let overlap = if self.left_column.is_empty() { 0.0 } else { STAMP_OVERLAP_CM };
            self.left_height_used -= stamp.diameter - overlap;
            return Some(stamp);
        }

        None
    }

    /// Returnează numărul total de stampile
    pub fn stamp_count(&self) -> usize {
        self.left_column.len() + self.right_column.len()
    }

    /// Returnează toate ID-urile de destinații deținute
    pub fn get_destination_ids(&self) -> Vec<u8> {
        let mut ids = Vec::new();

        for stamp in &self.left_column {
            if let Some(id) = stamp.destination_id {
                ids.push(id);
            }
        }

        for stamp in &self.right_column {
            if let Some(id) = stamp.destination_id {
                ids.push(id);
            }
        }

        ids
    }

    /// Returnează toate ștampilele din pașaport (pentru a număra FirstClass, etc.)
    pub fn all_stamps(&self) -> Vec<&Stamp> {
        let mut v = Vec::new();
        for s in &self.left_column { v.push(s); }
        for s in &self.right_column { v.push(s); }
        v
    }

    /// Caută o ștampilă după nume și returnează indexul ei în pașaport
    /// Returnează poziția sub formă de (column: 0=left, 1=right, index_in_column)
    pub fn find_stamp_index(&self, stamp_name: &str) -> Option<usize> {
        // Căutăm în ambele coloane, returnăm indexul global (simplu)
        for (i, stamp) in self.left_column.iter().enumerate() {
            if stamp.name == stamp_name {
                return Some(i); // index in left column
            }
        }
        for (i, stamp) in self.right_column.iter().enumerate() {
            if stamp.name == stamp_name {
                return Some(self.left_column.len() + i); // offset by left column size
            }
        }
        None
    }

    /// Șterge ștampila de la poziția specificată
    /// Poziția este indexul global (0..left.len() = left column, rest = right column)
    pub fn remove_stamp_at(&mut self, global_idx: usize) -> Option<Stamp> {
        if global_idx < self.left_column.len() {
            let stamp = self.left_column.remove(global_idx);
            // Recalculăm înălțimea coloanei stângi
            self.left_height_used = 0.0;
            for (i, s) in self.left_column.iter().enumerate() {
                let overlap = if i == 0 { 0.0 } else { STAMP_OVERLAP_CM };
                self.left_height_used += s.diameter - overlap;
            }
            Some(stamp)
        } else {
            let right_idx = global_idx - self.left_column.len();
            if right_idx < self.right_column.len() {
                let stamp = self.right_column.remove(right_idx);
                // Recalculăm înălțimea coloanei drepte
                self.right_height_used = 0.0;
                for (i, s) in self.right_column.iter().enumerate() {
                    let overlap = if i == 0 { 0.0 } else { STAMP_OVERLAP_CM };
                    self.right_height_used += s.diameter - overlap;
                }
                // Verificăm dacă am reintrat sub limită
                if self.right_height_used <= RIGHT_COLUMN_HEIGHT {
                    self.overflowed = false;
                }
                Some(stamp)
            } else {
                None
            }
        }
    }

    /// Afișare detaliată a pașaportului
    pub fn display(&self) {
        println!("  📘 PAȘAPORT:");
        println!("    Coloana Stângă (8cm): {:.2}cm umplut", self.left_height_used);
        for (i, stamp) in self.left_column.iter().enumerate() {
            println!("      {}. {} (⌀{:.1}cm)", i + 1, stamp.name, stamp.diameter);
        }

        println!("    Coloana Dreaptă (7cm): {:.2}cm umplut", self.right_height_used);
        for (i, stamp) in self.right_column.iter().enumerate() {
            println!("      {}. {} (⌀{:.1}cm)", i + 1, stamp.name, stamp.diameter);
        }

        if self.is_full() {
            println!("    🎉 PAȘAPORT PLIN! CÂȘTIGĂTOR!");
        } else {
            let left_remaining = LEFT_COLUMN_HEIGHT - self.left_height_used;
            let right_remaining = RIGHT_COLUMN_HEIGHT - self.right_height_used;
            println!("    Spațiu rămas: stânga {:.2}cm, dreapta {:.2}cm",
                     left_remaining, right_remaining);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passport_filling() {
        let mut passport = Passport::new();

        // Adăugăm stampile mici
        let stamp1 = Stamp {
            destination_id: Some(1),
            diameter: 1.5,
            name: "Test1".to_string(),
        };

        assert!(passport.add_stamp(stamp1.clone()));
        assert_eq!(passport.left_height_used, 1.5);

        // Umplem coloana stângă
        for _ in 0..4 {
            passport.add_stamp(stamp1.clone());
        }

        // Următoarea ar trebui să meargă în dreapta
        assert!(passport.add_stamp(stamp1.clone()));
        assert!(passport.right_height_used > 0.0);
    }

    #[test]
    fn test_passport_full() {
        let mut passport = Passport::new();

        let big_stamp = Stamp {
            destination_id: Some(1),
            diameter: 2.5,
            name: "Big".to_string(),
        };

        // umplem stânga
        passport.add_stamp(big_stamp.clone());
        passport.add_stamp(big_stamp.clone());
        passport.add_stamp(big_stamp.clone());

        // umplem dreapta
        passport.add_stamp(big_stamp.clone());
        passport.add_stamp(big_stamp.clone());

        // această ștampilă DEPĂȘEȘTE
        passport.add_stamp(big_stamp.clone());

        assert!(passport.is_full());
        assert!(passport.right_height_used > RIGHT_COLUMN_HEIGHT);
    }
}