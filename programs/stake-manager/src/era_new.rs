use crate::{EraProcessData, Errors, StakeManager};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EraNew<'info> {
    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub clock: Sysvar<'info, Clock>,
}

#[event]
pub struct EventEraNew {
    pub new_era: u64,
    pub need_bond: u64,
    pub need_unbond: u64,
    pub active: u64,
}

impl<'info> EraNew<'info> {
    pub fn process(&mut self) -> Result<()> {
        let new_era = self.stake_manager.latest_era + 1;

        require_gte!(self.clock.epoch, new_era, Errors::EraIsLatest);
        require!(
            self.stake_manager.era_process_data.is_empty(),
            Errors::EraIsProcessing
        );

        let (need_bond, need_unbond) =
            if self.stake_manager.era_bond > self.stake_manager.era_unbond {
                (
                    self.stake_manager.era_bond - self.stake_manager.era_unbond,
                    0,
                )
            } else {
                (
                    0,
                    self.stake_manager.era_unbond - self.stake_manager.era_bond,
                )
            };

        self.stake_manager.latest_era = new_era;
        self.stake_manager.era_bond = 0;
        self.stake_manager.era_unbond = 0;

        self.stake_manager.era_process_data = EraProcessData {
            need_bond,
            need_unbond,
            old_active: self.stake_manager.active,
            new_active: 0,
            pending_stake_accounts: self.stake_manager.stake_accounts.clone(),
        };

        emit!(EventEraNew {
            new_era,
            need_bond,
            need_unbond,
            active: self.stake_manager.active
        });
        Ok(())
    }
}
