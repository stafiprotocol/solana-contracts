use anchor_lang::prelude::*;

use crate::{Errors, StakeManager};

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        mut,
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Account<'info, StakeManager>,
    pub admin: Signer<'info>,
}

impl<'info> TransferAdmin<'info> {
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.stake_manager.admin = new_admin;
        Ok(())
    }
}
