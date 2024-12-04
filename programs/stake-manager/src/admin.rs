use crate::{Errors, StakeManager};
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
pub struct TransferFeeRecipient<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub stake_manager: Box<Account<'info, StakeManager>>,

    pub admin: Signer<'info>,
}

impl<'info> TransferFeeRecipient<'info> {
    pub fn process(&mut self, new_fee_recipient: Pubkey) -> Result<()> {
        self.stake_manager.fee_recipient = new_fee_recipient;

        msg!("TransferFeeRecipient: new fee_recipient: {}", new_fee_recipient);
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
        Ok(())
    }
}
