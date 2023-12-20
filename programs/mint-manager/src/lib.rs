use anchor_lang::{prelude::*, Bumps};

pub mod admin;
pub mod errors;
pub mod initialize;
pub mod mint;
pub mod states;

pub use crate::admin::*;
pub use crate::errors::Errors;
pub use crate::initialize::*;
pub use crate::mint::*;
pub use crate::states::*;

declare_id!("9akpBZZyVdyp4BGcsftn5dBNKEWSnKa9tZorKk4extwB");

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
pub mod mint_manager_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, ext_mint_authorities: Vec<Pubkey>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts
            .process(ext_mint_authorities, ctx.bumps.mint_authority)?;

        Ok(())
    }

    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(new_admin)?;

        Ok(())
    }

    pub fn set_ext_mint_authorities(
        ctx: Context<SetExtMintAuthorities>,
        ext_mint_authorities: Vec<Pubkey>,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(ext_mint_authorities)?;

        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, mint_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(mint_amount)?;

        Ok(())
    }
}
