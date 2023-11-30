use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke, stake, stake::state::StakeAuthorize, system_program,
};

#[derive(Accounts)]
pub struct StakeWithPool<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    #[account(
        mut,
        owner = system_program::ID
    )]
    pub transfer_from: Signer<'info>,
}

impl<'info> StakeWithPool<'info> {
    pub fn process(&mut self, target_pool: Pubkey, stake_amount: u64) -> Result<()> {
        require_gte!(
            stake_amount,
            self.stake_manager.min_stake_amount,
            Errors::StakeAmountTooLow
        );

        let pool = self
            .stake_manager
            .bonded_pools
            .get_mut(&target_pool)
            .ok_or_else(|| error!(Errors::PoolNotExist))?;

        Ok(())
    }
}
