use crate::{Errors, StakeManagerAccount};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use mint_manager::cpi::accounts::MintToken;
use mint_manager::program::MintManager;
use mint_manager::{self, MintManagerAccount};
#[derive(Accounts)]
pub struct EraUpdateRate<'info> {
    #[account(
        mut, 
        has_one = fee_recipient @ Errors::FeeRecipientNotMatch
    )]
    pub stake_manager: Account<'info, StakeManagerAccount>,

    #[account(
        seeds = [
            &stake_manager.key().to_bytes(),
            StakeManagerAccount::POOL_SEED,
        ],
        bump = stake_manager.pool_seed_bump
    )]
    pub stake_pool: SystemAccount<'info>,

    pub mint_manager: Box<Account<'info, MintManagerAccount>>,

    #[account(mut)]
    pub rsol_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint_manager.rsol_mint
    )]
    pub fee_recipient: Box<Account<'info, TokenAccount>>,

    /// CHECK:  check in mint-manager program
    pub mint_authority: UncheckedAccount<'info>,

    pub mint_manager_program: Program<'info, MintManager>,
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct EventEraUpdateRate {
    pub era: u64,
    pub rate: u64,
    pub fee: u64,
}

impl<'info> EraUpdateRate<'info> {
    pub fn process(&mut self) -> Result<()> {
        require!(
            self.stake_manager.era_process_data.need_update_rate(),
            Errors::EraNoNeedUpdateRate
        );

        let reward = if self.stake_manager.era_process_data.new_active
            > self.stake_manager.era_process_data.old_active
        {
            self.stake_manager.era_process_data.new_active
                - self.stake_manager.era_process_data.old_active
        } else {
            0
        };

        let protocol_fee = self.stake_manager.calc_protocol_fee(reward)?;
        if protocol_fee > 0 {
            let cpi_program = self.mint_manager_program.to_account_info();
            let cpi_accounts = MintToken {
                mint_manager: self.mint_manager.to_account_info(),
                rsol_mint: self.rsol_mint.to_account_info(),
                mint_to: self.fee_recipient.to_account_info(),
                mint_authority: self.mint_authority.to_account_info(),
                ext_mint_authority: self.stake_pool.to_account_info(),
                token_program: self.token_program.to_account_info(),
            };
            mint_manager::cpi::mint_token(
                CpiContext::new(cpi_program, cpi_accounts).with_signer(&[&[
                    &self.stake_manager.key().to_bytes(),
                    StakeManagerAccount::POOL_SEED,
                    &[self.stake_manager.pool_seed_bump],
                ]]),
                protocol_fee,
            )?;

            self.stake_manager.total_protocol_fee += protocol_fee;
            self.stake_manager.total_rsol_supply += protocol_fee;
        }

        let cal_temp = self.stake_manager.active + self.stake_manager.era_process_data.new_active;
        let new_active = if cal_temp > self.stake_manager.era_process_data.old_active {
            cal_temp - self.stake_manager.era_process_data.old_active
        } else {
            0
        };

        let new_rate = self
            .stake_manager
            .calc_rate(new_active, self.stake_manager.total_rsol_supply)?;
        let rate_change = self
            .stake_manager
            .calc_rate_change(self.stake_manager.rate, new_rate)?;
        require_gte!(
            self.stake_manager.rate_change_limit,
            rate_change,
            Errors::RateChangeOverLimit
        );

        self.stake_manager.era_process_data.old_active = 0;
        self.stake_manager.era_process_data.new_active = 0;
        self.stake_manager.active = new_active;
        self.stake_manager.rate = new_rate;

        emit!(EventEraUpdateRate{ 
            era: self.stake_manager.latest_era, 
            rate: new_rate,
            fee: protocol_fee
        });
        Ok(())
    }
}
