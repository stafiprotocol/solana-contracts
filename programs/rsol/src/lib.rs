use anchor_lang::{prelude::*, Bumps};

pub mod admin;
pub mod errors;
pub mod initialize;
pub mod states;

pub use crate::errors::Error;
pub use crate::initialize::*;
pub use crate::states::*;

declare_id!("47pM7t6NrHmmrkrnnpr1FfVYNHCohVsStaAsdaqYsxEV");

fn check_context<T: Bumps>(ctx: &Context<T>) -> Result<()> {
    if !check_id(ctx.program_id) {
        return err!(Error::ProgramIdNotMatch);
    }

    if !ctx.remaining_accounts.is_empty() {
        return err!(Error::RemainingAccountsNotMatch);
    }

    Ok(())
}

#[program]
pub mod rsol {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        admin: Pubkey,
        rsol_mint: Pubkey,
        validator: Pubkey,
        min_stake_amount: u64,
        unstake_fee_commission: u64,
        protocol_fee_commission: u64,
        rate_change_limit: u64,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(
            admin,
            rsol_mint,
            validator,
            min_stake_amount,
            unstake_fee_commission,
            protocol_fee_commission,
            rate_change_limit,
            ctx.bumps.stake_pool,
        )?;

        Ok(())
    }
}
