use crate::{Errors, StakeManager, UnstakeAccount};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    #[account(
        mut,
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED,
        ],
        bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(
        mut,
        close = recipient,
    )]
    pub unstake_account: Account<'info, UnstakeAccount>,

    #[account(
        mut,
        address = unstake_account.recipient @ Errors::UnstakeRecipientNotMatch
    )]
    pub recipient: SystemAccount<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn process(&mut self) -> Result<()> {
        require_keys_eq!(
            self.unstake_account.stake_manager,
            self.stake_manager.key(),
            Errors::InvalidUnstakeAccount
        );

        require_gt!(
            self.unstake_account.amount,
            0,
            Errors::UnstakeAccountAmountZero
        );

        require_gte!(
            self.clock.epoch,
            self.unstake_account.created_epoch + self.stake_manager.unbonding_duration,
            Errors::UnstakeAccountNotClaimable
        );

        let pool_balance = self.stake_pool.lamports();
        let withdraw_amount = self.unstake_account.amount;

        let available_for_withdraw = pool_balance - self.stake_manager.rent_exempt_for_pool_acc;
        if withdraw_amount > available_for_withdraw {
            return err!(Errors::PoolBalanceNotEnough);
        }

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.stake_pool.to_account_info(),
                    to: self.recipient.to_account_info(),
                },
                &[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManager::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ),
            withdraw_amount,
        )?;

        msg!(
            "Withdraw: staker: {} unstake account: {} amount: {}",
            self.recipient.key().to_string(),
            self.unstake_account.key().to_string(),
            withdraw_amount,
        );

        Ok(())
    }
}
