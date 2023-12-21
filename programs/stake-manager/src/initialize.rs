use anchor_lang::prelude::*;
use anchor_lang::solana_program::stake::instruction::LockupArgs;
use anchor_lang::solana_program::{program::invoke, stake, stake::state::StakeAuthorize};
use anchor_spl::stake::{Stake, StakeAccount};
use anchor_spl::token::{Mint, TokenAccount};

pub use crate::errors::Errors;
use crate::EraProcessData;
pub use crate::StakeManager;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED,
        ],
        bump,
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(token::mint = rsol_mint)]
    pub fee_recipient: Box<Account<'info, TokenAccount>>,

    pub rsol_mint: Box<Account<'info, Mint>>,

    pub admin: Signer<'info>,

    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeData {
    pub rsol_mint: Pubkey,
    pub validator: Pubkey,
    pub bond: u64,
    pub unbond: u64,
    pub active: u64,
    pub latest_era: u64,
    pub rate: u64,
    pub total_rsol_supply: u64,
    pub total_protocol_fee: u64,
}

impl<'info> Initialize<'info> {
    pub fn process(&mut self, initialize_data: InitializeData, pool_seed_bump: u8) -> Result<()> {
        require_keys_neq!(self.stake_manager.key(), self.stake_pool.key());

        let rent_exempt_for_pool_acc = self.rent.minimum_balance(0);
        require_eq!(
            self.stake_pool.lamports(),
            rent_exempt_for_pool_acc,
            Errors::RentNotEnough
        );

        require_eq!(
            initialize_data.rate,
            self.stake_manager
                .calc_rate(initialize_data.active, initialize_data.total_rsol_supply)?,
            Errors::InitializeDataMatch
        );

        self.stake_manager.set_inner(StakeManager {
            admin: self.admin.key(),
            balancer: self.admin.key(),
            rsol_mint: initialize_data.rsol_mint,
            rent_exempt_for_pool_acc,
            pool_seed_bump,
            fee_recipient: self.fee_recipient.key(),
            min_stake_amount: StakeManager::DEFAULT_MIN_STAKE_AMOUNT,
            unstake_fee_commission: StakeManager::DEFAULT_UNSTAKE_FEE_COMMISSION,
            protocol_fee_commission: StakeManager::DEFAULT_PROTOCOL_FEE_COMMISSION,
            rate_change_limit: StakeManager::DEFAULT_RATE_CHANGE_LIMIT,
            stake_accounts_len_limit: StakeManager::DEFAULT_STAKE_ACCOUNT_LEN_LIMIT,
            split_accounts_len_limit: StakeManager::DEFAULT_SPLIT_ACCOUNT_LEN_LIMIT,
            unbonding_duration: StakeManager::DEFAULT_UNBONDING_DURATION,
            latest_era: initialize_data.latest_era,
            rate: initialize_data.rate,
            total_rsol_supply: initialize_data.total_rsol_supply,
            total_protocol_fee: initialize_data.total_protocol_fee,
            era_bond: initialize_data.bond,
            era_unbond: initialize_data.unbond,
            active: initialize_data.active,
            validators: vec![initialize_data.validator],
            stake_accounts: vec![],
            split_accounts: vec![],
            era_process_data: EraProcessData {
                need_bond: 0,
                need_unbond: 0,
                old_active: 0,
                new_active: 0,
                pending_stake_accounts: vec![],
            },
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MigrateStakeAccount<'info> {
    #[account(mut)]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManager::POOL_SEED,
        ],
        bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    #[account(mut)]
    pub stake_account: Box<Account<'info, StakeAccount>>,

    pub stake_authority: Signer<'info>,

    pub stake_program: Program<'info, Stake>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> MigrateStakeAccount<'info> {
    pub fn process(&mut self) -> Result<()> {
        let delegation = self
            .stake_account
            .delegation()
            .ok_or_else(|| error!(Errors::DelegationEmpty))?;

        require_gte!(
            delegation.stake,
            self.stake_manager.min_stake_amount,
            Errors::StakeAmountTooLow
        );

        require_eq!(
            delegation.deactivation_epoch,
            std::u64::MAX,
            Errors::StakeAccountNotActive
        );

        require_gt!(
            self.stake_manager.stake_accounts_len_limit,
            self.stake_manager.stake_accounts.len() as u64,
            Errors::StakeAccountsLenOverLimit
        );

        if !self
            .stake_manager
            .validators
            .contains(&delegation.voter_pubkey)
        {
            return err!(Errors::ValidatorNotExist);
        }
        if self
            .stake_manager
            .stake_accounts
            .contains(&self.stake_account.key())
        {
            return err!(Errors::StakeAccountAlreadyExist);
        }

        let lockup = self.stake_account.lockup().unwrap();
        if lockup.is_in_force(&self.clock, None) {
            return err!(Errors::StakeAccountWithLockup);
        }

        // clean old lockup
        if lockup.custodian != Pubkey::default() {
            invoke(
                &stake::instruction::set_lockup(
                    &self.stake_account.key(),
                    &LockupArgs {
                        unix_timestamp: Some(0),
                        epoch: Some(0),
                        custodian: Some(Pubkey::default()),
                    },
                    self.stake_authority.key,
                ),
                &[
                    self.stake_program.to_account_info(),
                    self.stake_account.to_account_info(),
                    self.stake_authority.to_account_info(),
                ],
            )?;
        }

        // change new staker to stake pool
        invoke(
            &stake::instruction::authorize(
                self.stake_account.to_account_info().key,
                self.stake_authority.key,
                &self.stake_pool.key(),
                StakeAuthorize::Staker,
                None,
            ),
            &[
                self.stake_program.to_account_info(),
                self.stake_account.to_account_info(),
                self.clock.to_account_info(),
                self.stake_authority.to_account_info(),
            ],
        )?;

        // change new withdrawer to stake pool
        invoke(
            &stake::instruction::authorize(
                self.stake_account.to_account_info().key,
                self.stake_authority.key,
                &self.stake_pool.key(),
                StakeAuthorize::Withdrawer,
                None,
            ),
            &[
                self.stake_program.to_account_info(),
                self.stake_account.to_account_info(),
                self.clock.to_account_info(),
                self.stake_authority.to_account_info(),
            ],
        )?;

        // collect stake account
        self.stake_manager
            .stake_accounts
            .push(self.stake_account.key());

        Ok(())
    }
}
