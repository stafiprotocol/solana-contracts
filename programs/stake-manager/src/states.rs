use anchor_lang::prelude::*;

pub use crate::errors::Errors;

#[account]
#[derive(Debug)]
pub struct StakeManagerOld {
    pub admin: Pubkey,
    pub rsol_mint: Pubkey,
    pub fee_recipient: Pubkey,
    pub pool_seed_bump: u8,
    pub rent_exempt_for_pool_acc: u64,

    pub min_stake_amount: u64,
    pub unstake_fee_commission: u64,  // decimals 9
    pub protocol_fee_commission: u64, // decimals 9
    pub rate_change_limit: u64,       // decimals 9
    pub stake_accounts_len_limit: u64,
    pub split_accounts_len_limit: u64,
    pub unbonding_duration: u64,

    pub latest_era: u64,
    pub rate: u64, // decimals 9
    pub era_bond: u64,
    pub era_unbond: u64,
    pub active: u64,
    pub total_rsol_supply: u64,
    pub total_protocol_fee: u64,
    pub validators: Vec<Pubkey>,
    pub stake_accounts: Vec<Pubkey>,
    pub split_accounts: Vec<Pubkey>,
    pub era_process_data: EraProcessData,
}

#[account]
#[derive(Debug)]
pub struct StakeManager {
    pub admin: Pubkey,
    pub balancer: Pubkey,
    pub rsol_mint: Pubkey,
    pub fee_recipient: Pubkey,
    pub pool_seed_bump: u8,
    pub rent_exempt_for_pool_acc: u64,

    pub min_stake_amount: u64,
    pub unstake_fee_commission: u64,  // decimals 9
    pub protocol_fee_commission: u64, // decimals 9
    pub rate_change_limit: u64,       // decimals 9
    pub stake_accounts_len_limit: u64,
    pub split_accounts_len_limit: u64,
    pub unbonding_duration: u64,

    pub latest_era: u64,
    pub rate: u64, // decimals 9
    pub era_bond: u64,
    pub era_unbond: u64,
    pub active: u64,
    pub total_rsol_supply: u64,
    pub total_protocol_fee: u64,
    pub validators: Vec<Pubkey>,
    pub stake_accounts: Vec<Pubkey>,
    pub split_accounts: Vec<Pubkey>,
    pub era_process_data: EraProcessData,
}

#[derive(Clone, Debug, Default, AnchorSerialize, AnchorDeserialize)]
pub struct EraProcessData {
    pub need_bond: u64,
    pub need_unbond: u64,
    pub old_active: u64,
    pub new_active: u64,
    pub pending_stake_accounts: Vec<Pubkey>,
}

impl EraProcessData {
    pub fn is_empty(&self) -> bool {
        return self.need_bond == 0
            && self.need_unbond == 0
            && self.old_active == 0
            && self.new_active == 0
            && self.pending_stake_accounts.is_empty();
    }

    pub fn need_bond(&self) -> bool {
        return self.need_bond > 0;
    }

    pub fn need_unbond(&self) -> bool {
        return self.need_unbond > 0;
    }

    pub fn need_update_active(&self) -> bool {
        return self.need_bond == 0
            || self.need_unbond == 0 && !self.pending_stake_accounts.is_empty();
    }

    pub fn need_update_rate(&self) -> bool {
        return self.need_bond == 0
            && self.need_unbond == 0
            && self.pending_stake_accounts.is_empty()
            && self.old_active != 0
            && self.new_active != 0;
    }
}

impl StakeManager {
    pub const POOL_SEED: &'static [u8] = b"pool_seed";

    pub const DEFAULT_UNBONDING_DURATION: u64 = 2;
    pub const CAL_BASE: u64 = 1_000_000_000;
    pub const DEFAULT_MIN_STAKE_AMOUNT: u64 = 1_000_000;
    pub const DEFAULT_UNSTAKE_FEE_COMMISSION: u64 = 0;
    pub const DEFAULT_PROTOCOL_FEE_COMMISSION: u64 = 100_000_000;
    pub const DEFAULT_RATE_CHANGE_LIMIT: u64 = 500_000;
    pub const DEFAULT_STAKE_ACCOUNT_LEN_LIMIT: u64 = 100;
    pub const DEFAULT_SPLIT_ACCOUNT_LEN_LIMIT: u64 = 20;

    pub fn calc_rsol_amount(&self, sol_amount: u64) -> Result<u64> {
        u64::try_from((sol_amount as u128) * (StakeManager::CAL_BASE as u128) / (self.rate as u128))
            .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_sol_amount(&self, rsol_amount: u64) -> Result<u64> {
        u64::try_from(
            (rsol_amount as u128) * (self.rate as u128) / (StakeManager::CAL_BASE as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_unstake_fee(&self, rsol_amount: u64) -> Result<u64> {
        u64::try_from(
            (rsol_amount as u128) * (self.unstake_fee_commission as u128)
                / (StakeManager::CAL_BASE as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_protocol_fee(&self, reward_sol: u64) -> Result<u64> {
        u64::try_from(
            (reward_sol as u128) * (self.protocol_fee_commission as u128) / (self.rate as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_rate(&self, sol_amount: u64, rsol_amount: u64) -> Result<u64> {
        if sol_amount == 0 || rsol_amount == 0 {
            return Ok(StakeManager::CAL_BASE);
        }

        u64::try_from(
            (sol_amount as u128) * (StakeManager::CAL_BASE as u128) / (rsol_amount as u128),
        )
        .map_err(|_| error!(Errors::CalculationFail))
    }

    pub fn calc_rate_change(&self, old_rate: u64, new_rate: u64) -> Result<u64> {
        if old_rate == 0 {
            return Ok(0);
        }
        let diff = if old_rate > new_rate {
            old_rate - new_rate
        } else {
            new_rate - old_rate
        };

        u64::try_from((diff as u128) * (StakeManager::CAL_BASE as u128) / (old_rate as u128))
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
