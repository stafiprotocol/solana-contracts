use anchor_lang::prelude::*;

use crate::{Errors, StakeManager};

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(mut,has_one = admin @ Errors::AdminNotMatch)]
    pub stake_manager: Account<'info, StakeManager>,
    pub admin: Signer<'info>,
}

impl<'info> TransferAdmin<'info> {
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.stake_manager.admin = new_admin;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetMinStakeAmount<'info> {
    #[account(mut,has_one = admin @ Errors::AdminNotMatch)]
    pub stake_manager: Account<'info, StakeManager>,
    pub admin: Signer<'info>,
}

impl<'info> SetMinStakeAmount<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        self.stake_manager.min_stake_amount = amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetUnbondingDuration<'info> {
    #[account(mut,has_one = admin @ Errors::AdminNotMatch)]
    pub stake_manager: Account<'info, StakeManager>,
    pub admin: Signer<'info>,
}

impl<'info> SetUnbondingDuration<'info> {
    pub fn process(&mut self, duration: u64) -> Result<()> {
        self.stake_manager.unbonding_duration = duration;
        Ok(())
    }
}
