use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_spl::stake::StakeAccount;

#[derive(Accounts)]
pub struct EraUpdateActive<'info> {
    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub stake_account: Account<'info, StakeAccount>,
}

#[event]
pub struct EventEraUpdateActive {
    pub era: u64,
    pub stake_account: Pubkey,
    pub stake_amount: u64,
}

impl<'info> EraUpdateActive<'info> {
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

        // require stake is active (deactivation_epoch == u64::MAX)
        require_eq!(
            delegation.deactivation_epoch,
            std::u64::MAX,
            Errors::StakeAccountNotActive
        );

        self.stake_manager
            .era_process_data
            .pending_stake_accounts
            .retain(|&e| e != self.stake_account.key());

        self.stake_manager.era_process_data.new_active += delegation.stake;

        emit!(EventEraUpdateActive {
            era: self.stake_manager.latest_era,
            stake_account: self.stake_account.key(),
            stake_amount: delegation.stake
        });
        Ok(())
    }
}
