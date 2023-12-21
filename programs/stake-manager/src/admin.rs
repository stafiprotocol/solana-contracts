use std::io::Write;

use crate::{Errors, StakeManager, StakeManagerOld};
use anchor_lang::{prelude::*, system_program};
#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> TransferAdmin<'info> {
    pub fn process(&mut self, new_admin: Pubkey) -> Result<()> {
        self.stake_manager.admin = new_admin;

        msg!("TransferAdmin: new admin: {}", new_admin);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TransferBalancer<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> TransferBalancer<'info> {
    pub fn process(&mut self, new_balancer: Pubkey) -> Result<()> {
        self.stake_manager.balancer = new_balancer;

        msg!("TransferBalancer: new balancer: {}", new_balancer);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetMinStakeAmount<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> SetMinStakeAmount<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        self.stake_manager.min_stake_amount = amount;

        msg!("SetMinStakeAmount: amount: {}", amount);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetUnbondingDuration<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> SetUnbondingDuration<'info> {
    pub fn process(&mut self, duration: u64) -> Result<()> {
        self.stake_manager.unbonding_duration = duration;

        msg!("SetUnbondingDuration: duration: {}", duration);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetUnstakeFeeCommission<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> SetUnstakeFeeCommission<'info> {
    pub fn process(&mut self, unstake_fee_commission: u64) -> Result<()> {
        self.stake_manager.unstake_fee_commission = unstake_fee_commission;

        msg!("SetUnstakeFeeCommission: duration: {}", unstake_fee_commission);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct SetRateChangeLimit<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> SetRateChangeLimit<'info> {
    pub fn process(&mut self, rate_chagne_limit: u64) -> Result<()> {
        self.stake_manager.rate_change_limit = rate_chagne_limit;

        msg!("SetRateChangeLimit: rate change limit: {}", rate_chagne_limit);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct AddValidator<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> AddValidator<'info> {
    pub fn process(&mut self, new_validator: Pubkey) -> Result<()> {
        require!(!self.stake_manager.validators.contains(&new_validator), Errors::ValidatorAlreadyExist);

        self.stake_manager.validators.push(new_validator);

        msg!("AddValidator: new validator: {}", new_validator.key().to_string());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RemoveValidator<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> RemoveValidator<'info> {
    pub fn process(&mut self, remove_validator: Pubkey) -> Result<()> {
        require!(self.stake_manager.validators.contains(&remove_validator), Errors::ValidatorNotExist);

        self.stake_manager.validators.retain(|&e| e != remove_validator);

        msg!("RemoveValidator: remove validator: {}", remove_validator.key().to_string());
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(new_size: u32)]
pub struct ReallocStakeManager<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch,
        realloc = new_size as usize,
        realloc::payer = rent_payer,
        realloc::zero = false,
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
    
    #[account(
        mut,
        owner = system_program::ID,
    )]
    pub rent_payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> ReallocStakeManager<'info> {
    pub fn process(&mut self, new_size: u32) -> Result<()> {
        msg!("new_size {}", new_size);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpgradeStakeManager<'info> {
    /// CHECK: on process func 
    pub stake_manager: AccountInfo<'info>,
    pub admin: Signer<'info>,   
}

impl<'info> UpgradeStakeManager<'info> {
    pub fn process(&mut self) -> Result<()> {
        let stake_manager_old = StakeManagerOld::try_deserialize_unchecked(
            &mut self.stake_manager.try_borrow_data()?.as_ref())?;
        
        require_keys_eq!(stake_manager_old.admin, self.admin.key(), Errors::AdminNotMatch);

        let stake_manager  =  StakeManager{
            admin: stake_manager_old.admin, 
            balancer: stake_manager_old.admin, 
            rsol_mint: stake_manager_old.rsol_mint, 
            fee_recipient: stake_manager_old.fee_recipient, 
            pool_seed_bump: stake_manager_old.pool_seed_bump, 
            rent_exempt_for_pool_acc: stake_manager_old.rent_exempt_for_pool_acc, 
            min_stake_amount: stake_manager_old.min_stake_amount, 
            unstake_fee_commission: stake_manager_old.unstake_fee_commission, 
            protocol_fee_commission: stake_manager_old.protocol_fee_commission, 
            rate_change_limit: stake_manager_old.rate_change_limit, 
            stake_accounts_len_limit: stake_manager_old.stake_accounts_len_limit, 
            split_accounts_len_limit: stake_manager_old.split_accounts_len_limit, 
            unbonding_duration: stake_manager_old.unbonding_duration, 
            latest_era: stake_manager_old.latest_era, 
            rate: stake_manager_old.rate, 
            era_bond: stake_manager_old.era_bond, 
            era_unbond: stake_manager_old.era_unbond, 
            active: stake_manager_old.active, 
            total_rsol_supply: stake_manager_old.total_rsol_supply, 
            total_protocol_fee: stake_manager_old.total_protocol_fee, 
            validators: stake_manager_old.validators.clone(), 
            stake_accounts: stake_manager_old.stake_accounts.clone(), 
            split_accounts: stake_manager_old.split_accounts.clone(), 
            era_process_data: stake_manager_old.era_process_data.clone(),
        };

        let mut buffer: Vec<u8> = Vec::new();
        stake_manager.try_serialize(&mut buffer)?;
        self.stake_manager.try_borrow_mut_data()?.write_all(&buffer)?;

        Ok(())
    }
}
