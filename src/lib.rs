#![no_std]

elrond_wasm::imports!();

/// This contract is meant to store 10% of the $ECITY token's supply, and unlock it over 5 years for the Team to use in the project's developement. Should not be upgradable
#[elrond_wasm::contract]
pub trait VestingLocker {
    #[init]
    fn init(&self) {
        self.last_unlock().set_if_empty(0);
        self.times_unlocked().set_if_empty(0);
    }

    #[storage_mapper("token")]
    fn token(&self) -> SingleValueMapper<Self::Api, TokenIdentifier>;

    #[storage_mapper("lockAmount")] // The amount of tokens locked
    fn lock_amount(&self) -> SingleValueMapper<Self::Api, BigUint>;

    #[storage_mapper("timesUnlocked")] // The amount of times the tokens have been unlocked, out of 5
    fn times_unlocked(&self) -> SingleValueMapper<Self::Api, u64>;

    #[storage_mapper("lastUnlock")] // The timestamp of the last unlock
    fn last_unlock(&self) -> SingleValueMapper<Self::Api, u64>;

    #[payable("*")]
    #[only_owner]
    #[endpoint(lockTokens)]
    fn lock_tokens(&self) {
        require!(self.last_unlock().get() == 0, "Tokens already locked");

        let payment = self.call_value().all_esdt_transfers().get(0);

        require!(&payment.amount > &0, "Cannot lock 0 tokens");

        self.token().set(&payment.token_identifier);

        self.last_unlock().set(self.blockchain().get_block_timestamp());

        self.lock_amount().set(&payment.amount);
    }

    #[only_owner]
    #[endpoint(unlockTokens)]
    fn unlock_tokens(&self) {
        require!(self.last_unlock().get() != 0, "Tokens not locked");

        let current_time = self.blockchain().get_block_timestamp();

        let year_in_seconds = 365 * 24 * 60 * 60;

        let time_since_last_unlock = current_time - self.last_unlock().get();

        require!(time_since_last_unlock > year_in_seconds, "Tokens can only be unlocked once a year");

        require!(self.times_unlocked().get() < 5, "Tokens have already been unlocked 5 times");

        let unlock_amount = &self.lock_amount().get() / &BigUint::from(5u64);

        self.last_unlock().set(current_time);

        self.times_unlocked().update(|x| *x += 1);

        self.send().direct_esdt(&self.blockchain().get_caller(), &self.token().get(), 0u64, &unlock_amount);
    }
}
