use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_spl::stake::StakeAccount;

#[derive(Accounts)]
pub struct UpdateActive<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
}

impl<'info> UpdateActive<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_process_data.need_update_active(),
            Errors::EraNoNeedUpdateActive
        );

        require!(
            self.stake_manager
                .era_process_data
                .pending_stake_accounts
                .contains(&self.stake_account.key()),
            Errors::StakeAccountNotExist
        );

        let delegation = self
            .stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        self.stake_manager
            .era_process_data
            .pending_stake_accounts
            .retain(|&e| e != self.stake_account.key());

        self.stake_manager.era_process_data.new_active += delegation.stake;

        Ok(())
    }
}
