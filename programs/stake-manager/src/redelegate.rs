use crate::{Errors, StakeManager};
use anchor_lang::prelude::*;
use anchor_lang::{
    solana_program::{
        program::invoke_signed,
        stake::{self, state::StakeStateV2},
        sysvar::stake_history,
    },
    system_program,
};
use anchor_spl::stake::{withdraw, Stake, StakeAccount, Withdraw};


#[derive(Accounts)]
pub struct Redelegate<'info> {
    #[account(
        mut, 
        has_one = balancer @ Errors::BalancerNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub balancer: Signer<'info>,

    /// CHECK: validator account
    #[account(mut)]
    pub to_validator: UncheckedAccount<'info>,

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

    #[account(
        init,
        payer = rent_payer,
        space = std::mem::size_of::<StakeStateV2>(),
        owner = stake::program::ID,
    )]
    pub to_stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        owner = system_program::ID
    )]
    pub rent_payer: Signer<'info>,

    
    pub clock: Sysvar<'info, Clock>,
    /// CHECK: stake config account
    #[account(address = stake::config::ID)]
    pub stake_config: UncheckedAccount<'info>,
    /// CHECK: stake history
    #[account(address = stake_history::ID)]
    pub stake_history: UncheckedAccount<'info>,
    pub stake_program: Program<'info, Stake>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct EventRedelegate {
    pub from_stake_account: Pubkey,
    pub to_stake_account: Pubkey,
    pub redelegate_amount: u64,
}


impl<'info> Redelegate<'info> {
    pub fn process(&mut self, redelegate_amount: u64) -> Result<()> {
        require_gt!(redelegate_amount, 0, Errors::AmountUnmatch);

        require!(
            self.stake_manager.era_process_data.is_empty(), 
            Errors::EraIsProcessing)
        ;

        require!(
            self.stake_manager
                .stake_accounts
                .contains(&self.from_stake_account.key()),
            Errors::StakeAccountNotExist
        );

        require!(
            !self
                .stake_manager
                .stake_accounts
                .contains(&self.to_stake_account.key()),
            Errors::StakeAccountAlreadyExist
        );

        require!(
            !self
                .stake_manager
                .split_accounts
                .contains(&self.split_stake_account.key()),
            Errors::SplitStakeAccountAlreadyExist
        );

        require!(
            self.stake_manager
                .validators
                .contains(self.to_validator.key),
            Errors::ValidatorNotExist
        );

        let delegation = self
            .from_stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        // require stake is active (deactivation_epoch == u64::MAX)
        require_eq!(
            delegation.deactivation_epoch,
            std::u64::MAX,
            Errors::StakeAccountNotActive
        );

        require_keys_neq!(self.to_validator.key(), delegation.voter_pubkey, Errors::ValidatorNotMatch);

        require_gte!(delegation.stake, redelegate_amount, Errors::AmountUnmatch);

        let will_redelegate_from_stake_account =  if redelegate_amount < delegation.stake {
            // split
            let split_instruction = stake::instruction::split(
                self.from_stake_account.to_account_info().key,
                self.stake_pool.key,
                redelegate_amount,
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

            self.split_stake_account.to_account_info()
        } else {
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

            self.from_stake_account.to_account_info()
        };

        // redelegate
        let redelegate_instruction = &stake::instruction::redelegate(
            &will_redelegate_from_stake_account.key(),
            &self.stake_pool.key(),
            &self.to_validator.key(),
            &self.to_stake_account.key(),
        )
        .last()
        .unwrap()
        .clone();

        invoke_signed(
            redelegate_instruction,
                &[
                self.stake_program.to_account_info(),
                will_redelegate_from_stake_account.clone(),
                self.to_stake_account.to_account_info(),
                self.to_validator.to_account_info(),
                self.stake_config.to_account_info(),
                self.stake_pool.to_account_info(),
            ],
            &[&[
                &self.stake_manager.key().to_bytes(),
                StakeManager::POOL_SEED,
                &[self.stake_manager.pool_seed_bump],
            ]],
        )?;


        self.stake_manager
            .split_accounts
            .push(will_redelegate_from_stake_account.key());

        self.stake_manager
            .stake_accounts
            .push(self.to_stake_account.key());

        emit!(EventRedelegate{ 
            from_stake_account: self.from_stake_account.key(), 
            to_stake_account: self.to_stake_account.key(),
            redelegate_amount 
        });
        
        Ok(())
    }
}
