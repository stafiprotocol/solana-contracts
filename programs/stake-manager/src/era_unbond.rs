use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::stake_history;
use anchor_lang::{
    solana_program::{
        program::invoke_signed,
        stake::{self, state::StakeStateV2},
    },
    system_program,
};
use anchor_spl::stake::{
    deactivate_stake as solana_deactivate_stake, withdraw,
    DeactivateStake as SolanaDeactivateStake, Stake, StakeAccount, Withdraw,
};

#[derive(Accounts)]
pub struct EraUnbond<'info> {
    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED
        ],
        bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(mut)]
    pub from_stake_account: Account<'info, StakeAccount>,

    #[account(
        init,
        payer = rent_payer,
        space = std::mem::size_of::<StakeStateV2>(),
        owner = stake::program::ID,
    )]
    pub split_stake_account: Account<'info, StakeAccount>,

    /// CHECK: validator account
    #[account(mut)]
    pub validator: UncheckedAccount<'info>,

    #[account(
        mut,
        owner = system_program::ID
    )]
    pub rent_payer: Signer<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: stake history account
    #[account(address = stake_history::ID)]
    pub stake_history: UncheckedAccount<'info>,
    pub stake_program: Program<'info, Stake>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventEraUnbond {
    pub era: u64,
    pub from_stake_account: Pubkey,
    pub split_account: Pubkey,
    pub unbond_amount: u64,
}

impl<'info> EraUnbond<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_process_data.need_unbond(),
            Errors::EraNoNeedUnBond
        );

        require!(
            self.stake_manager
                .stake_accounts
                .contains(&self.from_stake_account.key()),
            Errors::StakeAccountNotExist
        );

        require!(
            !self
                .stake_manager
                .split_accounts
                .contains(&self.split_stake_account.key()),
            Errors::SplitStakeAccountAlreadyExist
        );

        let delegation = self
            .from_stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        require_keys_eq!(
            delegation.voter_pubkey,
            self.validator.key(),
            Errors::ValidatorNotMatch
        );

        let total_need_unbond = self.stake_manager.era_process_data.need_unbond;

        let (will_deactive_account, will_deactive_amount) = if delegation.stake <= total_need_unbond
        {
            // withdraw rent reserve back to payer
            withdraw(
                CpiContext::new(
                    self.stake_program.to_account_info(),
                    Withdraw {
                        stake: self.split_stake_account.to_account_info(),
                        withdrawer: self.split_stake_account.to_account_info(),
                        to: self.rent_payer.to_account_info(),
                        clock: self.clock.to_account_info(),
                        stake_history: self.stake_history.to_account_info(),
                    },
                ),
                self.split_stake_account.get_lamports(),
                None,
            )?;

            self.stake_manager
                .stake_accounts
                .retain(|&e| e != self.from_stake_account.key());

            self.stake_manager
                .era_process_data
                .pending_stake_accounts
                .retain(|&e| e != self.from_stake_account.key());

            (self.from_stake_account.to_account_info(), delegation.stake)
        } else {
            // split
            let split_instruction = stake::instruction::split(
                self.from_stake_account.to_account_info().key,
                self.stake_pool.key,
                total_need_unbond,
                &self.split_stake_account.key(),
            )
            .last()
            .unwrap()
            .clone();

            invoke_signed(
                &split_instruction,
                &[
                    self.stake_program.to_account_info(),
                    self.from_stake_account.to_account_info(),
                    self.split_stake_account.to_account_info(),
                    self.stake_pool.to_account_info(),
                ],
                &[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManager::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]],
            )?;

            (
                self.split_stake_account.to_account_info(),
                total_need_unbond,
            )
        };

        // deactive
        solana_deactivate_stake(CpiContext::new_with_signer(
            self.stake_program.to_account_info(),
            SolanaDeactivateStake {
                stake: will_deactive_account.clone(),
                staker: self.stake_pool.to_account_info(),
                clock: self.clock.to_account_info(),
            },
            &[&[
                &self.stake_manager.key().to_bytes(),
                StakeManager::POOL_SEED,
                &[self.stake_manager.pool_seed_bump],
            ]],
        ))?;

        self.stake_manager
            .split_accounts
            .push(will_deactive_account.key());

        self.stake_manager.era_process_data.need_unbond -= will_deactive_amount;

        emit!(EventEraUnbond {
            era: self.stake_manager.latest_era,
            from_stake_account: self.from_stake_account.key(),
            split_account: will_deactive_account.key(),
            unbond_amount: will_deactive_amount
        });
        Ok(())
    }
}
