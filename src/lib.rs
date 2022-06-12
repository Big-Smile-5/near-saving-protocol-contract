use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap};
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
pub const GAS_FOR_FT_TRANSFER: Gas = 30_000_000_000_000;
pub const GAS_FOR_FT_REGISTER_TRANSFER: Gas = 50_000_000_000_000;

#[derive(Deserialize, Serialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenData {
    pub related_token_id: AccountId,
    pub reward_rate: u128,
}

impl TokenData {
    pub fn new(token_id: AccountId, rate: u128) -> Self{
        Self {
            related_token_id: token_id,
            reward_rate: rate
        }
    }
}

#[derive(Deserialize, Serialize, BorshDeserialize, BorshSerialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct DepositDataDetail {
    pub token_time_tracker: u64,
    pub token_deposited_amount: Balance,
}

impl DepositDataDetail {
    pub fn new() -> Self{
        Self {
            token_time_tracker: 0u64,
            token_deposited_amount: 0u128
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DepositData {
    pub tokens: LookupMap<AccountId, DepositDataDetail>,
}

impl DepositData {
    pub fn new() -> Self{
        Self {
            tokens: LookupMap::new(b"d".to_vec()),
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub owner_id: AccountId,
    pub total_balance: Balance,
    pub accounts: LookupMap<AccountId, DepositData>,
    pub token_info: LookupMap<AccountId, TokenData>,
    pub reserve_token_info: LookupMap<AccountId, TokenData>,
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
            accounts: LookupMap::new(b"a".to_vec()),
            reserve_token_info: LookupMap::new(b"r".to_vec()),
        };
        let wnear_id = AccountId::from("wrap.testnet".to_string());
        let wnear_reserve_id = AccountId::from("reservetoken.testnet".to_string());
        this.token_info.insert(&wnear_id, &TokenData::new(wnear_reserve_id.clone(), 1u128));
        this.reserve_token_info.insert(&wnear_reserve_id, &TokenData::new(wnear_id.clone(), 1u128));
        this
    }

    pub fn calc_interest_for_account(&self, account: &DepositDataDetail) -> Balance {
        let timestamp = account.token_time_tracker;
        let mut interest = 0u128;
        if timestamp > 0 && account.token_deposited_amount > 0 {
            let current_timestamp = env::block_timestamp();
            let diff = (current_timestamp - timestamp) as u128 / (NANO * TIME_DEVISOR);
            let deposited_amount = account.token_deposited_amount;
            interest = (diff * ROI * deposited_amount / DIVISOR).into();
            env::log(format!("{}", diff).as_bytes());
            env::log(format!("{}", deposited_amount).as_bytes());
            env::log(format!("{}", diff * ROI * deposited_amount / DIVISOR).as_bytes());
        }
        interest
    }

    fn transfer_checker(&mut self, account_id: &AccountId, token_id: &AccountId, amount: Balance) {
        if self.token_info.contains_key(token_id) == true {
            env::log(format!("Trying to deposit the token").as_bytes());
            self.deposit_token(account_id, token_id, amount.clone());
        }
        else if self.reserve_token_info.contains_key(token_id) == true {
            env::log(format!("Trying to withdraw the token").as_bytes());
            self.withdraw_token(account_id, token_id, amount.clone());
        }
        else {
            env::panic(b"This contract is not registered to this saving protocol.");
        }
    }

    #[allow(unused_variables)]
    fn deposit_token(&mut self, sender: &AccountId, token_id: &AccountId, amount: Balance) {
        let mut deposit_data: DepositData;
        let mut deposit_data_detail: DepositDataDetail ;

        if self.accounts.contains_key(sender) {
            deposit_data = self.accounts.get(sender).unwrap();
        }
        else {
            deposit_data = DepositData::new();
        }

        if deposit_data.tokens.contains_key(token_id) {
            deposit_data_detail = deposit_data.tokens.get(token_id).unwrap();
        }
        else {
            deposit_data_detail = DepositDataDetail::new();
        }

        
        let interest = self.calc_interest_for_account(&deposit_data_detail);
        let new_balance = deposit_data_detail.token_deposited_amount + interest + amount;

        deposit_data_detail.token_time_tracker = env::block_timestamp();
        deposit_data_detail.token_deposited_amount = new_balance;
        deposit_data.tokens.insert(token_id, &deposit_data_detail);
        self.accounts.insert(sender, &deposit_data);
        // self.total_balance += amount;

        let _token_info: TokenData = self.token_info.get(token_id).unwrap();
        let _reserve_token_id = _token_info.related_token_id;
        ext_reserve_token::check_and_transfer(
            sender.to_string(),
            (amount + interest).into(),
            &_reserve_token_id,
            1,
            GAS_FOR_FT_REGISTER_TRANSFER,
        );
    }

    #[allow(unused_variables)]
    fn withdraw_token(&mut self, sender: &AccountId, token_id: &AccountId, amount: Balance) {
        let amount: Balance = amount.into();
        assert!(amount > 0u128, "The amount must be greater than zero.");
        // assert!(self.total_balance >= amount, "Insufficient balance.");

        let _token_info: TokenData = self.reserve_token_info.get(token_id).unwrap();
        let _reserve_token_id = _token_info.related_token_id;
        let mut deposit_data = self.accounts.get(sender).unwrap();
        let mut deposit_data_detail = deposit_data.tokens.get(&_reserve_token_id).unwrap();

        let interest = self.calc_interest_for_account(&deposit_data_detail);
        let mut balance = deposit_data_detail.token_deposited_amount + interest;
        assert!(balance >= amount, "The amount exceed current balance.");

        balance -= amount;
        deposit_data_detail.token_deposited_amount = balance;
        deposit_data_detail.token_time_tracker = env::block_timestamp();
        if deposit_data_detail.token_deposited_amount == 0u128 {
            deposit_data_detail.token_time_tracker = 0u64;
        }
        deposit_data.tokens.insert(&_reserve_token_id, &deposit_data_detail);
        // self.total_balance -= amount;
        self.accounts.insert(sender, &deposit_data);

        if interest >= amount {
            ext_ft::ft_transfer(
                sender.to_string(),
                (interest - amount).into(),
                Some("Reward transfer".to_string()),
                token_id,
                1,
                GAS_FOR_FT_TRANSFER,
            );
        }

        ext_ft::ft_transfer(
            sender.to_string(),
            amount.into(),
            Some("WNear withdraw".to_string()),
            &_reserve_token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        );
        env::log(format!("Withdrawed -> {}", amount).as_bytes());
        env::log(format!("Previous balance -> {}", balance).as_bytes());
    }

    pub fn get_deposit_balance(&self, account_id: AccountId, token_id: AccountId) -> Balance {
        let deposit_data = self.accounts.get(&account_id).unwrap();
        let deposit_data_detail = deposit_data.tokens.get(&token_id).unwrap();
        deposit_data_detail.token_deposited_amount
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