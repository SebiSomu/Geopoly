use crate::board::Destination;
use serde::{Serialize, Deserialize};

pub const LEFT_COLUMN_HEIGHT: f32 = 8.0; // cm
pub const RIGHT_COLUMN_HEIGHT: f32 = 7.0; // cm
pub const FIRST_CLASS_DIAMETER: f32 = 1.4; // cm
pub const COLUMN_WIDTH_CM: f32 = 2.5;     // 100px / 40px_per_cm
pub const TOUCH_OVERLAP_CM: f32 = 0.025;  // 1px / 40px_per_cm — slight overlap past tangent

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stamp {
    pub destination_id: Option<u8>, // None for First Class
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

    /// Compute the total stack height of a column using circle-tangent geometry.
    /// Stamps zig-zag: even indices flush right, odd indices flush left.
    /// Uses the same formula as the frontend visual positioning.
    fn column_height(stamps: &[Stamp]) -> f32 {
        if stamps.is_empty() {
            return 0.0;
        }

        let mut prev_x: f32 = 0.0;
        let mut prev_y: f32 = 0.0;
        let mut prev_d: f32 = 0.0;

        for (i, s) in stamps.iter().enumerate() {
            let d = s.diameter;
            let r = d / 2.0;

            // Zig-zag: even = flush right, odd = flush left
            let x = if i % 2 == 0 {
                (COLUMN_WIDTH_CM - d).max(0.0)
            } else {
                0.0
            };

            let y = if i == 0 {
                0.0
            } else {
                let prev_r = prev_d / 2.0;
                let prev_cx = prev_x + prev_r;
                let curr_cx = x + r;
                let dx = (curr_cx - prev_cx).abs();
                let sum_r = prev_r + r;

                let dy = if dx >= sum_r {
                    0.0
                } else {
                    (sum_r * sum_r - dx * dx).sqrt()
                };

                prev_y + prev_r + dy - r - TOUCH_OVERLAP_CM
            };

            prev_x = x;
            prev_y = y;
            prev_d = d;
        }

        // Total height = y of last stamp + its diameter
        prev_y + prev_d
    }

    pub fn add_stamp(&mut self, stamp: Stamp) -> bool {
        {
            let mut test_col = self.left_column.clone();
            test_col.push(stamp.clone());
            let new_height = Self::column_height(&test_col);
            if new_height <= LEFT_COLUMN_HEIGHT {
                self.left_column.push(stamp);
                self.left_height_used = new_height;
                return true;
            }
        }

        self.right_column.push(stamp);
        self.right_height_used = Self::column_height(&self.right_column);

        if self.right_height_used > RIGHT_COLUMN_HEIGHT + 0.005 {
            self.overflowed = true;
        }

        true
    }

    pub fn is_full(&self) -> bool {
        self.overflowed
    }

    pub fn remove_last_stamp(&mut self) -> Option<Stamp> {
        if let Some(stamp) = self.right_column.pop() {
            self.right_height_used = Self::column_height(&self.right_column);
            if self.right_height_used <= RIGHT_COLUMN_HEIGHT + 0.005 {
                self.overflowed = false;
            }
            return Some(stamp);
        }

        if let Some(stamp) = self.left_column.pop() {
            self.left_height_used = Self::column_height(&self.left_column);
            return Some(stamp);
        }

        None
    }

    pub fn stamp_count(&self) -> usize {
        self.left_column.len() + self.right_column.len()
    }

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

    pub fn all_stamps(&self) -> Vec<&Stamp> {
        let mut v = Vec::new();
        for s in &self.left_column { v.push(s); }
        for s in &self.right_column { v.push(s); }
        v
    }

    pub fn find_stamp_index(&self, stamp_name: &str) -> Option<usize> {
        for (i, stamp) in self.left_column.iter().enumerate() {
            if stamp.name == stamp_name {
                return Some(i);
            }
        }
        for (i, stamp) in self.right_column.iter().enumerate() {
            if stamp.name == stamp_name {
                return Some(self.left_column.len() + i);
            }
        }
        None
    }

    pub fn remove_stamp_at(&mut self, global_idx: usize) -> Option<Stamp> {
        if global_idx < self.left_column.len() {
            let stamp = self.left_column.remove(global_idx);
            self.left_height_used = Self::column_height(&self.left_column);
            Some(stamp)
        } else {
            let right_idx = global_idx - self.left_column.len();
            if right_idx < self.right_column.len() {
                let stamp = self.right_column.remove(right_idx);
                self.right_height_used = Self::column_height(&self.right_column);
                if self.right_height_used <= RIGHT_COLUMN_HEIGHT + 0.005 {
                    self.overflowed = false;
                }
                Some(stamp)
            } else {
                None
            }
        }
    }

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

        let stamp1 = Stamp {
            destination_id: Some(1),
            diameter: 1.5,
            name: "Test1".to_string(),
        };

        assert!(passport.add_stamp(stamp1.clone()));
        assert!((passport.left_height_used - 1.5).abs() < 0.001);

        for _ in 0..5 {
            passport.add_stamp(stamp1.clone());
        }

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

        passport.add_stamp(big_stamp.clone()); 
        passport.add_stamp(big_stamp.clone());
        passport.add_stamp(big_stamp.clone());

        passport.add_stamp(big_stamp.clone());
        passport.add_stamp(big_stamp.clone());

        passport.add_stamp(big_stamp.clone());

        assert!(passport.is_full());
        assert!(passport.right_height_used > RIGHT_COLUMN_HEIGHT + 0.005);
    }
}