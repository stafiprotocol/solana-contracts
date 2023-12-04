use anchor_lang::{prelude::*, Bumps};

pub mod admin;
pub mod errors;
pub mod initialize;
pub mod staker;
pub mod states;

pub use crate::admin::*;
pub use crate::errors::Errors;
pub use crate::initialize::*;
pub use crate::staker::*;
pub use crate::states::*;

declare_id!("47pM7t6NrHmmrkrnnpr1FfVYNHCohVsStaAsdaqYsxEV");

fn check_context<T: Bumps>(ctx: &Context<T>) -> Result<()> {
    if !check_id(ctx.program_id) {
        return err!(Errors::ProgramIdNotMatch);
    }

    if !ctx.remaining_accounts.is_empty() {
        return err!(Errors::RemainingAccountsNotMatch);
    }

    Ok(())
}

#[program]
pub mod rsol {

    use super::*;

    // initialize

    pub fn initialize(ctx: Context<Initialize>, initialize_data: InitializeData) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(
            initialize_data,
            ctx.bumps.stake_pool,
            ctx.bumps.ext_mint_authority,
        )?;

        Ok(())
    }

    pub fn migrate_stake_account(
        ctx: Context<MigrateStakeAccount>,
        target_pool: Pubkey,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(target_pool)?;

        Ok(())
    }

    // admin

    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(new_admin)?;

        Ok(())
    }

    // staker

    pub fn stake(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(stake_amount)?;

        Ok(())
    }

    pub fn un_stake(ctx: Context<Unstake>, unstake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(unstake_amount)?;

        Ok(())
    }
}
