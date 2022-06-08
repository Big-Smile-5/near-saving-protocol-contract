use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{U128};
use near_sdk::{
    assert_one_yocto, env, near_bindgen, require, AccountId, Balance, PromiseOrValue, Gas, ext_contract
};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

const NANO: u128 = 1000000000;
const ROI: u128 = 1_000_000_000_000_000_000_000_000;
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);


#[derive(Deserialize, Serialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    pub wnear_time_tracker: u64,
    pub wnear_deposited_amount: Balance,
}

impl Account {
    pub fn new() -> Self{
        Self {
            wnear_time_tracker: 0u64,
            wnear_deposited_amount: 0
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
impl FungibleTokenReceiver for Contract {
    /**
    Callback on receiving tokens by this contract.
    Returns zero.
    Panics when account is not registered. */
    #[allow(unused_variables)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // env::log_str("Ok! Deposited!");
        self.deposit_wnear(&sender_id, amount.into());
        return PromiseOrValue::Value(U128(0));
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

    pub fn calc_interest_for_account(&self, account: &Account) -> Balance {
        let timestamp = account.wnear_time_tracker;
        let current_timestamp = env::block_timestamp();
        let diff = (current_timestamp - timestamp) as u128 / (NANO * 3600u128);

        let deposited_amount = account.wnear_deposited_amount;
        let interest = (diff * ROI + deposited_amount).into();
        interest
    }

    pub fn deposit_wnear(&mut self, account_id: &AccountId, amount: Balance) {
        let mut account = self.accounts.get(account_id).unwrap_or_else(|| Account::new());
        let new_balance = self.calc_interest_for_account(&account) + amount;

        account.wnear_time_tracker = env::block_timestamp();
        account.wnear_deposited_amount = new_balance;
        self.accounts.insert(account_id, &account);

        env::log_str("Deposit success!");
        env::log_str(&(new_balance.to_string()));
    }

    #[payable]
    pub fn withdraw_wnear(&mut self, amount: U128) {
        assert_one_yocto();

        let amount = amount.into();
        let recipient = env::predecessor_account_id();
        let mut account = self.accounts.get(&recipient).unwrap_or_else(|| Account::new());
        let mut balance = self.calc_interest_for_account(&account);

        require!(balance >= amount, "The amount exceed current balance.");

        balance -= amount;
        account.wnear_deposited_amount = balance;
        account.wnear_time_tracker = env::block_timestamp();
        self.accounts.insert(&recipient, &account);

        let _token = AccountId::new_unchecked("usdn.testnet".to_string());

        env::log_str("Withdraw success!");
        env::log_str(&(balance.to_string()));

        ext_ft::ft_transfer(
            recipient,
            amount.into(),
            Some("WNear withdraw".to_string()),
            &_token,
            1, // required 1yNEAR for transfers
            GAS_FOR_FT_TRANSFER,
        );
    }

    pub fn get_wnear_balance(&self) -> Balance {
        let account_id = env::predecessor_account_id();
        let account = self.accounts.get(&account_id).unwrap();
        let balance = self.calc_interest_for_account(&account);
        balance
    }
}

#[ext_contract(ext_ft)]
trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    fn ft_total_supply(&self) -> U128;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
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

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn deposit_wnear() {
        let alice = AccountId::new_unchecked("alice.testnet".to_string());      // Get Alice as an account ID
        let context = get_context(alice);       // Set up the testing context and unit test environment
        testing_env!(context.build());

        let mut contract = Contract::new();
        let amount = 1_000_000_000_000_000u128;
        contract.deposit_wnear(amount.into());
    }

    #[test]
    fn withdraw_wnear() {
        let alice = AccountId::new_unchecked("alice.testnet".to_string());
        let context = get_context(alice);
        testing_env!(context.build());

        let mut contract = Contract::new();
        let deposit_amount = 1_000_000_000_000_000u128;
        let withdraw_amount = 1_000_000_000_000u128;
        contract.deposit_wnear(deposit_amount.into());
        contract.withdraw_wnear(withdraw_amount.into());
        // assert!(!guess_result, "Expected a failure from the wrong guess");
        // assert_eq!(get_logs(), ["Try again."], "Expected a failure log.");
        // assert_eq!(
        //     get_logs(),
        //     ["Try again.", "You guessed right!"],
        //     "Expected a successful log after the previous failed log."
        // );
    }
}*/