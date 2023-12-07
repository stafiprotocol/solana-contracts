use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::stake_history;
use anchor_lang::solana_program::{program::invoke_signed, stake};
use anchor_spl::stake::{Stake, StakeAccount};

#[derive(Accounts)]
pub struct Merge<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    #[account(mut)]
    pub src_stake_account: Box<Account<'info, StakeAccount>>,

    #[account(mut)]
    pub dst_stake_account: Box<Account<'info, StakeAccount>>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED
        ],
        bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    pub clock: Sysvar<'info, Clock>,

    /// CHECK: stake history
    #[account(address = stake_history::ID)]
    pub stake_history: UncheckedAccount<'info>,

    pub stake_program: Program<'info, Stake>,
}

impl<'info> Merge<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_process_data.is_empty(),
            Errors::EraIsProcessing
        );

        require!(
            self.stake_manager
                .stake_accounts
                .contains(&self.src_stake_account.key()),
            Errors::StakeAccountNotExist
        );

        require!(
            !self
                .stake_manager
                .stake_accounts
                .contains(&self.dst_stake_account.key()),
            Errors::StakeAccountNotExist
        );

        let src_delegation = self
            .src_stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        let dst_delegation = self
            .dst_stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        require_eq!(
            src_delegation.deactivation_epoch,
            std::u64::MAX,
            Errors::StakeAccountDeActive
        );

        require_eq!(
            dst_delegation.deactivation_epoch,
            std::u64::MAX,
            Errors::StakeAccountDeActive
        );

        require_keys_eq!(
            src_delegation.voter_pubkey,
            dst_delegation.voter_pubkey,
            Errors::ValidatorsNotEqual
        );

        invoke_signed(
            &stake::instruction::merge(
                self.dst_stake_account.to_account_info().key,
                self.src_stake_account.to_account_info().key,
                self.stake_pool.to_account_info().key,
            )[0],
            &[
                self.stake_program.to_account_info(),
                self.dst_stake_account.to_account_info(),
                self.src_stake_account.to_account_info(),
                self.clock.to_account_info(),
                self.stake_history.to_account_info(),
                self.stake_pool.to_account_info(),
            ],
            &[&[
                &self.stake_manager.key().to_bytes(),
                StakeManager::POOL_SEED,
                &[self.stake_manager.pool_seed_bump],
            ]],
        )?;

        self.stake_manager
            .stake_accounts
            .retain(|&e| e != self.src_stake_account.key());

        Ok(())
    }
}
