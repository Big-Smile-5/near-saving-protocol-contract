use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, PromiseOrValue, Gas, ext_contract
};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

const ROI: u128 = 1;
const DIVISOR: u128 = 1000;
const NANO: u128 = 1000000000;
const TIME_DEVISOR: u128 = 1;
pub const GAS_FOR_FT_TRANSFER: Gas = 50_000_000_000_000;


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
            wnear_deposited_amount: 0u128
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub owner_id: AccountId,
    pub total_balance: Balance,
    pub accounts: UnorderedMap<AccountId, Account>,
    pub token_info: LookupMap<AccountId, u128>,
    pub reserve_token_info: LookupMap<AccountId, bool>,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic(b"This contract shuld be initialized before usage")
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {

    #[allow(unused_variables)]
    fn ft_on_transfer(&mut self, sender_id: ValidAccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        env::log(format!("{}", env::predecessor_account_id()).as_bytes());

        let sender_id = AccountId::from(sender_id);
        let token_id = env::predecessor_account_id();
        self.transfer_checker(&sender_id, &token_id, amount.into());
        return PromiseOrValue::Value(U128(0));
    }
}

#[near_bindgen]
impl Contract {

    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");

        let sender = env::predecessor_account_id();
        let mut this = Self {
            owner_id: sender,
            total_balance: 0,
            token_info: LookupMap::new(b"t".to_vec()),
            accounts: UnorderedMap::new(b"a".to_vec()),
            reserve_token_info: LookupMap::new(b"r".to_vec()),
        };
        this.token_info.insert(&AccountId::from("wrap.testnet".to_string()), &1u128);
        this.reserve_token_info.insert(&AccountId::from("reservetoken.testnet".to_string()), &true);
        this
    }

    pub fn calc_interest_for_account(&self, account: &Account) -> Balance {
        let timestamp = account.wnear_time_tracker;
        let mut interest = 0u128;
        if timestamp > 0 {
            let current_timestamp = env::block_timestamp();
            let diff = (current_timestamp - timestamp) as u128 / (NANO * TIME_DEVISOR);

            let deposited_amount = account.wnear_deposited_amount;
            env::log(format!("{}", diff).as_bytes());
            env::log(format!("{}", diff * ROI * deposited_amount / DIVISOR).as_bytes());
            interest = (diff * ROI * deposited_amount / DIVISOR + deposited_amount).into();
        }
        interest
    }

    fn transfer_checker(&mut self, account_id: &AccountId, token_id: &AccountId, amount: Balance) {
        if self.token_info.contains_key(token_id) == true {
            self.deposit_token(account_id, token_id, amount.clone());
        }
        else if self.reserve_token_info.contains_key(token_id) == true {
            self.withdraw_token(account_id, token_id, amount.clone())
        }
        else {
            env::panic(b"This contract is not registered to this saving protocol.");
        }
    }

    #[allow(unused_variables)]
    fn deposit_token(&mut self, sender: &AccountId, token_id: &AccountId, amount: Balance) {
        let mut account = self.accounts.get(sender).unwrap_or_else(|| Account::new());
        let new_balance = self.calc_interest_for_account(&account) + amount;

        account.wnear_time_tracker = env::block_timestamp();
        account.wnear_deposited_amount = new_balance;
        self.accounts.insert(sender, &account);
        self.total_balance += amount;

        let _token = AccountId::from("reservetoken.testnet".to_string());
        ext_reserve_token::check_and_transfer(
            sender.to_string(),
            amount.into(),
            &_token,
            1,
            GAS_FOR_FT_TRANSFER,
        );
    }

    #[allow(unused_variables)]
    fn withdraw_token(&mut self, sender: &AccountId, token_id: &AccountId, amount: Balance) {
        let amount: Balance = amount.into();
        assert!(amount > 0u128, "The amount must be greater than zero.");
        assert!(self.total_balance >= amount, "Insufficient balance.");

        let mut account = self.accounts.get(sender).unwrap_or_else(|| Account::new());
        let mut balance = self.calc_interest_for_account(&account);
        assert!(balance >= amount, "The amount exceed current balance.");

        balance -= amount;
        account.wnear_deposited_amount = balance;
        account.wnear_time_tracker = env::block_timestamp();
        
        self.total_balance -= amount;
        if account.wnear_deposited_amount == 0u128 {
            account.wnear_time_tracker = 0u64;
        }
        self.accounts.insert(sender, &account);

        let _token = AccountId::from("wrap.testnet".to_string());
        ext_ft::ft_transfer(
            sender.to_string(),
            amount.into(),
            Some("WNear withdraw".to_string()),
            &_token,
            1,
            GAS_FOR_FT_TRANSFER,
        );
        env::log(format!("Withdrawed -> {}", amount).as_bytes());
        env::log(format!("Previous balance -> {}", balance).as_bytes());
    }

    pub fn get_wnear_balance(&self, account_id: AccountId) -> Balance {
        let account = self.accounts.get(&account_id).unwrap_or_else(|| Account::new());
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

#[ext_contract(ext_reserve_token)]
trait ReserveToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_total_supply(&self) -> U128;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;

    fn check_and_transfer(&self, account_id: AccountId, amount: U128);
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext, Balance};
    use near_sdk::test_utils::{accounts};

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: String, storage_usage: u64) -> VMContext {
        VMContext {
            current_account_id: "alice.testnet".to_string(),
            signer_account_id: "jane.testnet".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn get_balance() {
        let context = get_context("Alice".to_string(), 0);
        testing_env!(context);

        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        let balance = contract.ft_balance_of(accounts(1).into());
        println!("{:?}", balance);
    }

    // #[test]
    // fn deposit_wnear() {
    //     let alice = AccountId::new_unchecked("alice.testnet".to_string());      // Get Alice as an account ID
    //     let context = get_context(alice);       // Set up the testing context and unit test environment
    //     testing_env!(context.build());

    //     let mut contract = Contract::new();
    //     let amount = 1_000_000_000_000_000u128;
    //     contract.deposit_wnear(amount.into());
    // }

    // #[test]
    // fn withdraw_wnear() {
    //     let alice = AccountId::new_unchecked("alice.testnet".to_string());
    //     let context = get_context(alice);
    //     testing_env!(context.build());

    //     let mut contract = Contract::new();
    //     let deposit_amount = 1_000_000_000_000_000u128;
    //     let withdraw_amount = 1_000_000_000_000u128;
    //     contract.deposit_wnear(deposit_amount.into());
    //     contract.withdraw_wnear(withdraw_amount.into());
    //     // assert!(!guess_result, "Expected a failure from the wrong guess");
    //     // assert_eq!(get_logs(), ["Try again."], "Expected a failure log.");
    //     // assert_eq!(
    //     //     get_logs(),
    //     //     ["Try again.", "You guessed right!"],
    //     //     "Expected a successful log after the previous failed log."
    //     // );
    // }
}*/