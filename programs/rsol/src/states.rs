use anchor_lang::prelude::*;

pub use crate::errors::Errors;

#[account]
#[derive(Debug)]
pub struct StakeManager {
    pub admin: Pubkey,
    pub rsol_mint: Pubkey,
    pub fee_recipient: Pubkey,
    pub pool_seed_bump: u8,
    pub rent_exempt_for_pool_acc: u64,
    pub min_stake_amount: u64,
    pub unstake_fee_commission: u64,  // decimals 9
    pub protocol_fee_commission: u64, // decimals 9
    pub rate_change_limit: u64,       // decimals 9
    pub unbonding_duration: u64,
    pub latest_era: u64,
    pub rate: u64, // decimals 9
    pub total_rsol_supply: u64,
    pub total_protocol_fee: u64,
    pub bond: u64,
    pub unbond: u64,
    pub active: u64,
    pub validators: Vec<Pubkey>,
    pub stake_accounts: Vec<Pubkey>,
    pub split_accounts: Vec<Pubkey>,
}

impl StakeManager {
    pub const POOL_SEED: &'static [u8] = b"pool_seed";
    pub const FEE_RECIPIENT_SEED: &'static [u8] = b"fee_recipient_seed";

    pub const DEFAULT_UNBONDING_DURATION: u64 = 3;
    pub const DEFAULT_RATE: u64 = 1_000_000_000;
    pub const DEFAULT_MIN_STAKE_AMOUNT: u64 = 1_000_000;
    pub const DEFAULT_UNSTAKE_FEE_COMMISSION: u64 = 100_000_000;
    pub const DEFAULT_PROTOCOL_FEE_COMMISSION: u64 = 100_000_000;
    pub const DEFAULT_RATE_CHANGE_LIMIT: u64 = 500_000;

    pub fn calc_rsol_amount(&self, sol_amount: u64) -> Result<u64> {
        u64::try_from(
            (sol_amount as u128) * (StakeManager::DEFAULT_RATE as u128) / (self.rate as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_sol_amount(&self, rsol_amount: u64) -> Result<u64> {
        u64::try_from(
            (rsol_amount as u128) * (self.rate as u128) / (StakeManager::DEFAULT_RATE as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }
}

#[account]
#[derive(Debug)]
pub struct UnstakeAccount {
    pub stake_manager: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub created_epoch: u64,
}
