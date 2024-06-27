use l1_poc::{Blockchain, Transaction};
use l1_poc::pos::PoS;

fn main() {
    let mut blockchain = Blockchain::new();
    let mut pos = PoS::new();

    // Example validators
    pos.update_stake("Alice".to_string(), 50);
    pos.update_stake("Bob".to_string(), 30);
    pos.update_stake("Charlie".to_string(), 20);

    // Add transactions
    blockchain.add_transaction(Transaction {
        sender: "Alice".to_string(), 
        receiver: "Bob".to_string(), 
        amount: 10 
    });
    blockchain.add_transaction(Transaction { 
        sender: "Bob".to_string(), 
        receiver: "Charlie".to_string(), 
        amount: 5 
    });

    // Select a validator and add a block
    let validator = pos.select_validator();
    blockchain.add_block(validator.clone());

    // Print the blockchain
    for block in blockchain.chain {
        println!("{:?}", block);
    }
}
