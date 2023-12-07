use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::{
    solana_program::{
        program::invoke_signed,
        stake::{self, state::StakeState},
    },
    system_program,
};
use anchor_spl::stake::{
    deactivate_stake as solana_deactivate_stake, DeactivateStake as SolanaDeactivateStake, Stake,
    StakeAccount,
};

#[derive(Accounts)]
pub struct UnBond<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    #[account(
        mut,
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED
            ],
            bump = stake_manager.pool_seed_bump
        )]
    pub stake_pool: SystemAccount<'info>,

    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
            init,
            payer = rent_payer,
            space = std::mem::size_of::<StakeState>(),
            owner = stake::program::ID,
        )]
    pub split_stake_account: Account<'info, StakeAccount>,

    /// CHECK: validator account
    #[account(mut)]
    pub validator: UncheckedAccount<'info>,

    #[account(
        mut,
        owner = system_program::ID
    )]
    pub rent_payer: Signer<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,

    pub stake_program: Program<'info, Stake>,
    pub system_program: Program<'info, System>,
}

impl<'info> UnBond<'info> {
    pub fn process(&mut self, unbond_amount: u64) -> Result<()> {
        require!(
            self.stake_manager.era_process_data.need_unbond(),
            Errors::EraNoNeedUnBond
        );

        require_gte!(
            self.stake_manager.era_process_data.need_unbond,
            unbond_amount,
            Errors::AmountUnmatch
        );

        require!(
            self.stake_manager.validators.contains(self.validator.key),
            Errors::ValidatorNotExist
        );

        require!(
            self.stake_manager
                .stake_accounts
                .contains(&self.stake_account.key()),
            Errors::StakeAccountNotExist
        );

        require!(
            !self
                .stake_manager
                .split_accounts
                .contains(&self.split_stake_account.key()),
            Errors::SplitStakeAccountAlreadyExist
        );

        let delegation = self
            .stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        require_keys_eq!(
            delegation.voter_pubkey,
            self.validator.key(),
            Errors::ValidatorNotMatch
        );

        require_gte!(delegation.stake, unbond_amount, Errors::BalanceNotEnough);

        if delegation.stake == unbond_amount {
            solana_deactivate_stake(CpiContext::new_with_signer(
                self.stake_program.to_account_info(),
                SolanaDeactivateStake {
                    stake: self.stake_account.to_account_info(),
                    staker: self.stake_pool.to_account_info(),
                    clock: self.clock.to_account_info(),
                },
                &[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManager::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ))?;

            self.stake_manager
                .stake_accounts
                .retain(|&e| e != self.stake_account.key());

            self.stake_manager
                .era_process_data
                .pending_stake_accounts
                .retain(|&e| e != self.stake_account.key());
        } else {
            let split_instruction = stake::instruction::split(
                self.stake_account.to_account_info().key,
                self.stake_pool.key,
                unbond_amount,
                &self.split_stake_account.key(),
            )
            .last()
            .unwrap()
            .clone();

            invoke_signed(
                &split_instruction,
                &[
                    self.stake_program.to_account_info(),
                    self.stake_account.to_account_info(),
                    self.split_stake_account.to_account_info(),
                    self.stake_pool.to_account_info(),
                ],
                &[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManager::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]],
            )?;

            solana_deactivate_stake(CpiContext::new_with_signer(
                self.stake_program.to_account_info(),
                SolanaDeactivateStake {
                    stake: self.split_stake_account.to_account_info(),
                    staker: self.stake_pool.to_account_info(),
                    clock: self.clock.to_account_info(),
                },
                &[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManager::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ))?;

            self.stake_manager
                .split_accounts
                .push(self.split_stake_account.key());
        }

        self.stake_manager.era_process_data.need_unbond -= unbond_amount;

        Ok(())
    }
}
