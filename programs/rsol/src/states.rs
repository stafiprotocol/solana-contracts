use anchor_lang::prelude::*;
use std::collections::BTreeMap;
use std::convert::Into;

#[account]
#[derive(Debug)]
pub struct StakeManager {
    pub admin: Pubkey,
    pub rsol_mint: Pubkey,
    pub fee_recipient: Pubkey,
    pub fee_recipient_seed_bump: u8,
    pub latest_pool_seed_index: u8,
    pub min_stake_amount: u64,
    pub unstake_fee_commission: u64,  // decimals 9
    pub protocol_fee_commission: u64, // decimals 9
    pub rate_change_limit: u64,       // decimals 9
    pub unbonding_duration: u64,
    pub latest_era: u64,
    pub rate: u64, // decimals 9
    pub total_rsol_supply: u64,
    pub total_protocol_fee: u64,
    pub bonded_pools: BTreeMap<Pubkey, PoolInfo>, // pda account
}

#[derive(Clone, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct PoolInfo {
    pub seed_index: u8,
    pub seed_bump: u8,
    pub bond: u64,
    pub unbond: u64,
    pub active: u64,
    pub validator: Vec<Pubkey>,
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
}
