use anchor_lang::{prelude::*, Bumps};

pub mod admin;
pub mod era_bond;
pub mod era_merge;
pub mod era_new;
pub mod era_unbond;
pub mod era_update_active;
pub mod era_update_rate;
pub mod era_withdraw;
pub mod errors;
pub mod initialize;
pub mod staker_stake;
pub mod staker_unstake;
pub mod staker_withdraw;
pub mod states;

pub use crate::admin::*;
pub use crate::era_bond::*;
pub use crate::era_merge::*;
pub use crate::era_new::*;
pub use crate::era_unbond::*;
pub use crate::era_update_active::*;
pub use crate::era_update_rate::*;
pub use crate::era_withdraw::*;
pub use crate::errors::Errors;
pub use crate::initialize::*;
pub use crate::staker_stake::*;
pub use crate::staker_unstake::*;
pub use crate::staker_withdraw::*;
pub use crate::states::*;

declare_id!("7KguRVBwBAbnfPhkTEnw3n2SA9YiesuebuBAX9AhB7eH");

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
pub mod stake_manager_program {

    use super::*;

    // initialize

    pub fn initialize(ctx: Context<Initialize>, initialize_data: InitializeData) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts
            .process(initialize_data, ctx.bumps.stake_pool)?;

        Ok(())
    }

    pub fn migrate_stake_account(ctx: Context<MigrateStakeAccount>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    // admin

    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(new_admin)?;

        Ok(())
    }

    pub fn set_min_stake_amount(ctx: Context<SetMinStakeAmount>, amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(amount)?;

        Ok(())
    }

    pub fn set_unbonding_duration(ctx: Context<SetUnbondingDuration>, duration: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(duration)?;

        Ok(())
    }

    pub fn set_rate_change_limit(
        ctx: Context<SetRateChangeLimit>,
        rate_change_limit: u64,
    ) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(rate_change_limit)?;

        Ok(())
    }

    pub fn add_validator(ctx: Context<AddValidator>, new_validator: Pubkey) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(new_validator)?;

        Ok(())
    }

    pub fn remove_validator(ctx: Context<RemoveValidator>, remove_validator: Pubkey) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(remove_validator)?;

        Ok(())
    }

    pub fn redelegate(ctx: Context<Redelegate>, redelegate_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(redelegate_amount)?;

        Ok(())
    }

    // staker

    pub fn stake(ctx: Context<Stake>, stake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(stake_amount)?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, unstake_amount: u64) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process(unstake_amount)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    // era

    pub fn era_new(ctx: Context<EraNew>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_bond(ctx: Context<EraBond>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_unbond(ctx: Context<EraUnbond>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_update_active(ctx: Context<EraUpdateActive>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_update_rate(ctx: Context<EraUpdateRate>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_merge(ctx: Context<EraMerge>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }

    pub fn era_withdraw(ctx: Context<EraWithdraw>) -> Result<()> {
        check_context(&ctx)?;

        ctx.accounts.process()?;

        Ok(())
    }
}
