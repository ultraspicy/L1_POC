pub mod pos;

use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: String,
}

impl Block {
    pub fn new(
        index: u64, 
        transactions: Vec<Transaction>, 
        previous_hash: String, 
        validator: String
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let hash = Block::calculate_hash(index, timestamp, &transactions, &previous_hash, &validator);
        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
            validator,
        }
    }

    pub fn calculate_hash(
        index: u64, 
        timestamp: i64, 
        transactions: &Vec<Transaction>, 
        previous_hash: &String, 
        validator: &String
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(index.to_string());
        hasher.update(timestamp.to_string());
        hasher.update(serde_json::to_string(transactions).unwrap());
        hasher.update(previous_hash);
        hasher.update(validator);
        format!("{:x}", hasher.finalize())
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub unconfirmed_transactions: Vec<Transaction>,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            unconfirmed_transactions: Vec::new(),
        };
        let genesis_block = Block::new(0, Vec::new(), "0".to_string(), "genesis".to_string());
        blockchain.chain.push(genesis_block);
        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.unconfirmed_transactions.push(transaction);
    }

    pub fn add_block(&mut self, validator: String) {
        let last_block = self.chain.last().unwrap();
        let new_block = Block::new(
            last_block.index + 1,
            self.unconfirmed_transactions.clone(),
            last_block.hash.clone(),
            validator,
        );
        self.chain.push(new_block);
        self.unconfirmed_transactions.clear();
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != Block::calculate_hash(
                current_block.index,
                current_block.timestamp,
                &current_block.transactions,
                &current_block.previous_hash,
                &current_block.validator,
            ) {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }
}