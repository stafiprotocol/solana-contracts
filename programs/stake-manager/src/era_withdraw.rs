use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::stake_history;
use anchor_spl::stake::{withdraw, Stake, StakeAccount, Withdraw};

#[derive(Accounts)]
pub struct EraWithdraw<'info> {
    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

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

    pub clock: Sysvar<'info, Clock>,
    /// CHECK: stake history
    #[account(address = stake_history::ID)]
    pub stake_history: UncheckedAccount<'info>,
    pub stake_program: Program<'info, Stake>,
}

#[event]
pub struct EventEraWithdraw {
    pub era: u64,
    pub stake_account: Pubkey,
    pub withdraw_amount: u64,
}

impl<'info> EraWithdraw<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager
                .split_accounts
                .contains(&self.stake_account.key()),
            Errors::StakeAccountNotExist
        );

        let delegation = self
            .stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        require_neq!(
            delegation.deactivation_epoch,
            std::u64::MAX,
            Errors::StakeAccountActive
        );

        let withdraw_amount = self.stake_account.get_lamports();
        withdraw(
            CpiContext::new_with_signer(
                self.stake_program.to_account_info(),
                Withdraw {
                    stake: self.stake_account.to_account_info(),
                    withdrawer: self.stake_pool.to_account_info(),
                    to: self.stake_pool.to_account_info(),
                    clock: self.clock.to_account_info(),
                    stake_history: self.stake_history.to_account_info(),
                },
                &[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManager::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ),
            withdraw_amount,
            None,
        )?;

        self.stake_manager
            .split_accounts
            .retain(|&e| e != self.stake_account.key());

        emit!(EventEraWithdraw {
            era: self.stake_manager.latest_era,
            stake_account: self.stake_account.key(),
            withdraw_amount
        });
        Ok(())
    }
}
