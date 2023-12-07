use crate::{EraProcessData, Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_lang::{
    solana_program::{
        program::{invoke, invoke_signed},
        stake::{
            self,
            state::{Authorized, Lockup, StakeState},
        },
        sysvar::stake_history,
    },
    system_program,
};
use anchor_spl::stake::{withdraw, Stake, StakeAccount, Withdraw};

#[derive(Accounts)]
pub struct NewEra<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    pub clock: Sysvar<'info, Clock>,
}

impl<'info> NewEra<'info> {
    pub fn process(&mut self) -> Result<()> {
        let new_era = self.stake_manager.latest_era + 1;

        require_gte!(self.clock.epoch, new_era, Errors::EraIsLatest);
        require!(
            self.stake_manager.era_process_data.is_empty(),
            Errors::EraIsProcessing
        );

        let (need_bond, need_unbond) = if self.stake_manager.bond > self.stake_manager.unbond {
            (self.stake_manager.bond - self.stake_manager.unbond, 0)
        } else {
            (0, self.stake_manager.unbond - self.stake_manager.bond)
        };

        self.stake_manager.latest_era = new_era;
        self.stake_manager.era_process_data = EraProcessData {
            need_bond,
            need_unbond,
            old_active: self.stake_manager.active,
            new_active: 0,
            pending_stake_accounts: self.stake_manager.stake_accounts.clone(),
        };

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Bond<'info> {
    #[account(mut)]
    pub stake_manager: Account<'info, StakeManager>,

    /// CHECK: validator account
    #[account(mut)]
    pub validator: UncheckedAccount<'info>,

    #[account(
    mut,
    seeds = [
        &stake_manager.key().to_bytes(),
        StakeManager::POOL_SEED
    ],
    bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(
        init,
        payer = rent_payer,
        space = std::mem::size_of::<StakeState>(),
        owner = stake::program::ID,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        owner = system_program::ID
    )]
    pub rent_payer: Signer<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub epoch_schedule: Sysvar<'info, EpochSchedule>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: stake config account
    #[account(address = stake::config::ID)]
    pub stake_config: UncheckedAccount<'info>,

    /// CHECK: stake history account
    #[account(address = stake_history::ID)]
    pub stake_history: UncheckedAccount<'info>,

    pub stake_program: Program<'info, Stake>,
    pub system_program: Program<'info, System>,
}

impl<'info> Bond<'info> {
    pub fn process(&mut self) -> Result<()> {
        require_gt!(
            self.stake_manager.era_process_data.need_bond,
            0,
            Errors::EraDoesNotNeedBond
        );

        require!(
            self.stake_manager.validators.contains(self.validator.key),
            Errors::ValidatorNotExist
        );

        require!(
            !self
                .stake_manager
                .stake_accounts
                .contains(&self.stake_account.key()),
            Errors::StakeAccountAlreadyExist
        );

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.stake_pool.to_account_info(),
                    to: self.stake_account.to_account_info(),
                },
                &[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManager::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]],
            ),
            self.stake_manager.era_process_data.need_bond,
        )?;

        invoke(
            &stake::instruction::initialize(
                &self.stake_account.key(),
                &Authorized {
                    staker: self.stake_pool.key(),
                    withdrawer: self.stake_pool.key(),
                },
                &Lockup::default(),
            ),
            &[
                self.stake_program.to_account_info(),
                self.stake_account.to_account_info(),
                self.rent.to_account_info(),
            ],
        )?;

        invoke_signed(
            &stake::instruction::delegate_stake(
                &self.stake_account.key(),
                &self.stake_pool.key(),
                self.validator.key,
            ),
            &[
                self.stake_program.to_account_info(),
                self.stake_account.to_account_info(),
                self.stake_pool.to_account_info(),
                self.validator.to_account_info(),
                self.clock.to_account_info(),
                self.stake_history.to_account_info(),
                self.stake_config.to_account_info(),
            ],
            &[&[
                &self.stake_manager.key().to_bytes(),
                StakeManager::POOL_SEED,
                &[self.stake_manager.pool_seed_bump],
            ]],
        )?;

        self.stake_manager.era_process_data.need_bond = 0;
        self.stake_manager
            .stake_accounts
            .push(self.stake_account.key());

        Ok(())
    }
}
