use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use near_sdk::{
    env, near_bindgen, require, AccountId, Balance
};

const nano: u128 = 1000000000;
const ROI: u128 = 1000000000000000000000000;

#[derive(Deserialize, Serialize, Debug)]
pub struct Account {
    pub wNear_time_tracker: u64,
    pub wNear_deposited_amount: Balance,
}

impl Account {
    pub fn new() -> Self{
        Self {
            wNear_time_tracker: 0u64,
            wNear_deposited_amount: 0
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub accounts: UnorderedMap<AccountId, Account>,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("This contract shuld be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {

    #[init]
    pub fn new() -> Self {
        Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
        }
    }

    pub fn calcInterestForAccount(&self, account: &Account) -> Balance {
        let timestamp = account.wNear_time_tracker;
        let current_timestamp = env::block_timestamp();
        let diff = (current_timestamp - timestamp) as u128 / (nano * 3600u128);

        let mut deposited_amount = account.wNear_deposited_amount;
        let interest = (diff * ROI + deposited_amount).into();
        interest
    }

    pub fn depositWNear(&self, amount: U128) {
        let amount: u128 = amount.into();
        let account_id = env::predecessor_account_id();
        let mut account = self.accounts.get(&account_id).unwrap_or_else(|| Account::new());
        let new_balance = self.calcInterestForAccount(&account) + amount;

        account.wNear_time_tracker = env::block_timestamp();
        account.wNear_deposited_amount = new_balance;
        self.accounts.insert(&account_id, &account);

        env::log_str("Deposit success!");
        // env::log_str(&(new_balance.to_string()));
    }

    pub fn withdrawWNear(&self, amount: U128) {
        let amount = amount.into();
        let account_id = env::predecessor_account_id();
        let mut account = self.accounts.get(&account_id).unwrap_or_else(|| Account::new());
        let mut balance = self.calcInterestForAccount(&account);

        require!(balance >= amount, "The amount exceed current balance.");

        balance -= amount;
        account.wNear_deposited_amount = amount;
        account.wNear_time_tracker = env::block_timestamp();
        
        env::log_str("Withdraw success!");
        // env::log_str(&(balance.to_string()));
    }

    pub fn getWNearBalance(&self) -> Balance {
        let account_id = env::predecessor_account_id();
        let mut account = self.accounts.get(&account_id).unwrap();
        let balance = self.calcInterestForAccount(&account);
        balance
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
/*
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    #[test]
    fn debug_get_hash() {
        // Basic set up for a unit test
        testing_env!(VMContextBuilder::new().build());

        // Using a unit test to rapidly debug and iterate
        let debug_solution = "near nomicon ref finance";
        let debug_hash_bytes = env::sha256(debug_solution.as_bytes());
        let debug_hash_string = hex::encode(debug_hash_bytes);
        println!("Let's debug: {:?}", debug_hash_string);
    }

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn check_guess_solution() {
        // Get Alice as an account ID
        let alice = AccountId::new_unchecked("alice.testnet".to_string());
        // Set up the testing context and unit test environment
        let context = get_context(alice);
        testing_env!(context.build());

        // Set up contract object and call the new method
        let contract = Contract::new(
            "69c2feb084439956193f4c21936025f14a5a5a78979d67ae34762e18a7206a0f".to_string(),
        );
        let guess_result = contract.guess_solution();
        println!("BlockTimeStamp: {}", guess_result)
        // assert!(!guess_result, "Expected a failure from the wrong guess");
        // assert_eq!(get_logs(), ["Try again."], "Expected a failure log.");
        // guess_result = contract.guess_solution("near nomicon ref finance".to_string());
        // assert!(guess_result, "Expected the correct answer to return true.");
        // assert_eq!(
        //     get_logs(),
        //     ["Try again.", "You guessed right!"],
        //     "Expected a successful log after the previous failed log."
        // );
    }
}*/