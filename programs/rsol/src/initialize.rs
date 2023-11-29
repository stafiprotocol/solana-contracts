use anchor_lang::{prelude::*, solana_program::program_pack::Pack};
use anchor_spl::stake::StakeAccount;
use anchor_spl::token::{spl_token, Mint};
use std::collections::BTreeMap;

use crate::{PoolInfo, StakeManager};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED,
            &[0]
        ],
        bump,
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::FEE_RECIPIENT_SEED,
        ],
        bump,
    )]
    pub fee_recipient: SystemAccount<'info>,

    pub rsol_mint: Box<Account<'info, Mint>>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeData {
    pub admin: Pubkey,
    pub rsol_mint: Pubkey,
    pub validator: Pubkey,
    pub bond: u64,
    pub unbond: u64,
    pub active: u64,
    pub latest_era: u64,
    pub rate: u64,
    pub total_rsol_supply: u64,
    pub total_protocol_fee: u64,
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
        initialize_data: InitializeData,
        pool_seed_bump: u8,
        fee_recipient_seed_bump: u8,
    ) -> Result<()> {
        require_keys_neq!(self.stake_manager.key(), self.stake_pool.key());

        let rent_exempt_for_token_acc = self.rent.minimum_balance(spl_token::state::Account::LEN);
        require_eq!(self.fee_recipient.lamports(), rent_exempt_for_token_acc);

        let mut bonded_pools = BTreeMap::new();
        bonded_pools.insert(
            self.stake_pool.key(),
            PoolInfo {
                seed_index: 0,
                seed_bump: pool_seed_bump,
                bond: initialize_data.bond,
                unbond: initialize_data.unbond,
                active: initialize_data.active,
                validator: vec![initialize_data.validator],
                stake_accounts: vec![],
                split_accounts: vec![],
            },
        );

        self.stake_manager.set_inner(StakeManager {
            admin: initialize_data.admin,
            rsol_mint: initialize_data.rsol_mint,
            fee_recipient: self.fee_recipient.key(),
            fee_recipient_seed_bump,
            latest_pool_seed_index: 1,
            min_stake_amount: StakeManager::DEFAULT_MIN_STAKE_AMOUNT,
            unstake_fee_commission: StakeManager::DEFAULT_UNSTAKE_FEE_COMMISSION,
            protocol_fee_commission: StakeManager::DEFAULT_PROTOCOL_FEE_COMMISSION,
            rate_change_limit: StakeManager::DEFAULT_RATE_CHANGE_LIMIT,
            unbonding_duration: StakeManager::DEFAULT_UNBONDING_DURATION,
            latest_era: initialize_data.latest_era,
            rate: StakeManager::DEFAULT_RATE,
            total_rsol_supply: initialize_data.total_rsol_supply,
            total_protocol_fee: initialize_data.total_protocol_fee,
            bonded_pools,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MigrateStakeAccount<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    #[account(mut)]
    pub stake_account: Box<Account<'info, StakeAccount>>,
    pub stake_authority: Signer<'info>,
}

impl<'info> MigrateStakeAccount<'info> {
    pub fn process(&mut self) -> Result<()> {
        Ok(())
    }
}
