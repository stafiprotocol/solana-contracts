use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct EraUpdateRate<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,
}

impl<'info> EraUpdateRate<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_process_data.need_update_rate(),
            Errors::EraNoNeedUpdateRate
        );

        let new_active = self.stake_manager.active + self.stake_manager.era_process_data.new_active
            - self.stake_manager.era_process_data.old_active;

        self.stake_manager.era_process_data.old_active = 0;
        self.stake_manager.era_process_data.new_active = 0;

        self.stake_manager.active = new_active;
        self.stake_manager.rate =
            new_active * StakeManager::RATE_BASE / self.stake_manager.total_rsol_supply;

        Ok(())
    }
}
