use anchor_lang::prelude::*;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{Errors, StakeManager};

#[derive(Accounts)]
pub struct NewEra<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

impl<'info> NewEra<'info> {
    pub fn process(&mut self) -> Result<()> {
        let new_era = self.stake_manager.latest_era + 1;

        require_gte!(self.clock.epoch, new_era, Errors::EraIsLatest);

        Ok(())
    }
}
