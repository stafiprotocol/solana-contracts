use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use std::collections::BTreeMap;

use crate::{PoolInfo, StakeManager};

pub fn current_era(current_epoch: u64, era_epochs: u64) -> u64 {
    if era_epochs == 0 {
        return current_epoch;
    } else {
        return current_epoch / era_epochs;
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::STAKE_POOL_BASE_SEED,
            &[0x00]
        ],
        bump,
    )]
    pub stake_pool: SystemAccount<'info>,

    pub rsol_mint: Box<Account<'info, Mint>>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn stake_manager(&self) -> &StakeManager {
        &self.stake_manager
    }

    pub fn stake_manager_address(&self) -> &Pubkey {
        self.stake_manager.to_account_info().key
    }

    pub fn process(
        &mut self,
        admin: Pubkey,
        rsol_mint: Pubkey,
        validator: Pubkey,
        min_stake_amount: u64,
        unstake_fee_commission: u64,
        protocol_fee_commission: u64,
        rate_change_limit: u64,
        stake_pool_bump: u8,
    ) -> Result<()> {
        require_keys_neq!(self.stake_manager.key(), self.stake_pool.key());

        let mut bonded_pools = BTreeMap::new();
        bonded_pools.insert(
            self.stake_pool.key(),
            PoolInfo {
                bond: 0,
                unbond: 0,
                active: 0,
                validator,
                bump: stake_pool_bump,
                stake_accounts: vec![],
                split_accounts: vec![],
            },
        );
        let latest_era = current_era(self.clock.epoch, StakeManager::DEFAULT_ERA_EPOCHS);

        self.stake_manager.set_inner(StakeManager {
            admin,
            rsol_mint,
            min_stake_amount,
            unstake_fee_commission,
            protocol_fee_commission,
            rate_change_limit,
            era_epochs: StakeManager::DEFAULT_ERA_EPOCHS,
            unbonding_duration: StakeManager::DEFAULT_UNBONDING_DURATION,
            latest_era,
            rate: StakeManager::DEFAULT_TATE,
            total_lsd_token_supply: 0,
            total_protocol_fee: 0,
            bonded_pools,
        });

        Ok(())
    }
}
