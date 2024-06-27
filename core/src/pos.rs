use rand::seq::SliceRandom;
use std::collections::HashMap;
use rand::thread_rng;

pub struct PoS {
    pub stakes: HashMap<String, u64>,
}

impl PoS {
    pub fn new() -> Self {
        PoS {
            stakes: HashMap::new(),
        }
    }

    pub fn update_stake(&mut self, validator: String, amount: u64) {
        let stake = self.stakes.entry(validator).or_insert(0);
        *stake += amount;
    }

    pub fn select_validator(&self) -> String {
        let total_stake: u64 = self.stakes.values().sum();
        let mut rng = thread_rng();
        let validators: Vec<&String> = self.stakes.keys().collect();
        let selected_validator = validators.choose_weighted(&mut rng, |validator| *self.stakes.get(*validator).unwrap() as f64 / total_stake as f64).unwrap();
        selected_validator.to_string()
    }
}
