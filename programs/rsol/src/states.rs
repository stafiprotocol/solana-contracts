use anchor_lang::prelude::*;
use std::collections::BTreeMap;
use std::convert::Into;

#[account]
#[derive(Debug)]
pub struct StakeManager {
    pub admin: Pubkey,
    pub rsol_mint: Pubkey,
    pub min_stake_amount: u64,
    pub unstake_fee_commission: u64,  // decimals 9
    pub protocol_fee_commission: u64, // decimals 9
    pub rate_change_limit: u64,       // decimals 9
    pub era_epochs: u64,
    pub unbonding_duration: u64,
    pub latest_era: u64,
    pub rate: u64, // decimals 9
    pub total_lsd_token_supply: u64,
    pub total_protocol_fee: u64,
    pub bonded_pools: BTreeMap<Pubkey, PoolInfo>, // pda account
}

#[derive(Clone, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct PoolInfo {
    pub bond: u64,
    pub unbond: u64,
    pub active: u64,
    pub bump: u8,
    pub validator: Pubkey,
    pub stake_accounts: Vec<Pubkey>,
    pub split_accounts: Vec<Pubkey>,
}

impl StakeManager {
    pub const STAKE_POOL_BASE_SEED: &'static [u8] = b"stake_pool_base_seed";
    pub const DEFAULT_ERA_EPOCHS: u64 = 1;
    pub const DEFAULT_UNBONDING_DURATION: u64 = 1;
    pub const DEFAULT_TATE: u64 = 1_000_000_000;
}
