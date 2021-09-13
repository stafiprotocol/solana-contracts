//! An example of a bridge to execute arbitrary Solana proposals.
//!
//! This program can be used to allow a bridge to govern anything a regular
//! Pubkey can govern. One can use the bridge as a BPF program upgrade
//! authority, a mint authority, etc.
//!
//! To use, one must first create a `Bridge` account, specifying two important
//! parameters:
//!
//! 1. Owners - the set of addresses that sign proposals for the bridge.
//! 2. Threshold - the number of signers required to execute a proposal.
//!
//! Once the `Bridge` account is created, one can create a `Proposal`
//! account, specifying the parameters for a normal solana proposal.
//!
//! To sign, owners should invoke the `approve` instruction.

use anchor_lang::prelude::*;
use std::convert::Into;
use std::collections::BTreeMap;
use anchor_spl::token::{self, Burn, MintTo};

declare_id!("Cqpdbx8h2uVj4s3aKHCnhTdbgxx9eRSFg3UwPE7te9M6");

#[program]
pub mod bridge {
    use super::*;

    // Initializes a new bridge account with a set of owners and a threshold.
    pub fn create_bridge(
        ctx: Context<CreateBridge>,
        owners: Vec<Pubkey>,
        threshold: u64,
        nonce: u8,
        resource_id_to_mint: BTreeMap<[u8;32], Pubkey>,
        admin: Pubkey,
    ) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.owners = owners;
        bridge.threshold = threshold;
        bridge.nonce = nonce;
        bridge.owner_set_seqno = 0;
        bridge.deposit_counts = BTreeMap::new();
        bridge.resource_id_to_mint = resource_id_to_mint;
        bridge.admin = admin;
        Ok(())
    }

    pub fn set_resource_id(
        ctx: Context<AdminAuth>,
        resource_id: [u8;32],
        mint: Pubkey,
    ) -> Result<()> {
        let bridge = &mut ctx.accounts.bridge;
        bridge.resource_id_to_mint.insert(resource_id, mint);
        Ok(())
    }

    // Initiates a transfer by creating a deposit account
    pub fn burn_and_create_deposit(
        ctx: Context<CreateDeposit>,
        amount: u64,
        receiver: Vec<u8>,
        dest_chain_id: u8,
        resource_id: [u8;32],
    ) -> Result<()> {
        //burn
        token::burn(ctx.accounts.into(), amount)?;

        let bridge_account = &mut ctx.accounts.bridge;
        let deposit_account = &mut ctx.accounts.deposit;
        let deposit_count =  bridge_account.deposit_counts.entry(dest_chain_id).or_insert(0);

        // update bridge deposit counts
        *deposit_count += 1;
        // update deposit
        deposit_account.amount = amount;
        deposit_account.receiver = receiver;
        deposit_account.dest_chain_id = dest_chain_id;
        deposit_account.resource_id = resource_id;
        deposit_account.deposit_nonce = *deposit_count;
        Ok(())
    }

    // Creates a new proposal account
    // which must be one of the owners of the bridge.
    pub fn create_mint_proposal(
        ctx: Context<CreateMintProposal>,
        resource_id: [u8;32],
        to: Pubkey,
        amount: u64,
        token_program: Pubkey,
    ) -> Result<()> {
        let _ = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.proposer.key)
            .ok_or(ErrorCode::InvalidOwner)?;

        let mut signers = Vec::new();
        signers.resize(ctx.accounts.bridge.owners.len(), false);

        let p = &mut ctx.accounts.proposal;
        let mint_op = ctx.accounts.bridge.resource_id_to_mint.get(&resource_id);
        let mint = if let Some(m) = mint_op{
            m
        }else{
            return Err(ErrorCode::InvalidResourceId.into());
        };

        p.mint = *mint;
        p.to = to;
        p.amount = amount;
        p.token_program = token_program;
        p.signers = signers;
        p.bridge = *ctx.accounts.bridge.to_account_info().key;
        p.did_execute = false;
        p.owner_set_seqno = ctx.accounts.bridge.owner_set_seqno;

        Ok(())
    }

    // Approve and Executes the given proposal if threshold owners have signed it.
    pub fn approve_mint_proposal(ctx: Context<Approve>) -> Result<()> {
        msg!("000");
        let owner_index = ctx
            .accounts
            .bridge
            .owners
            .iter()
            .position(|a| a == ctx.accounts.approver.key)
            .ok_or(ErrorCode::InvalidOwner)?;

        ctx.accounts.proposal.signers[owner_index] = true;
        msg!("111");
        // Do we have enough signers.
        let sig_count = ctx
            .accounts
            .proposal
            .signers
            .iter()
            .filter(|&did_sign| *did_sign)
            .count() as u64;
        
            msg!("222");
        if sig_count < ctx.accounts.bridge.threshold {
            return Ok(());
        }

        // Has this been executed already?
        if ctx.accounts.proposal.did_execute {
            return Err(ErrorCode::AlreadyExecuted.into());
        }

        if ctx.accounts.proposal.mint != *ctx.accounts.mint.key {
            return Err(ErrorCode::InvalidMintAccount.into());
        }
        if ctx.accounts.proposal.to != *ctx.accounts.to.key {
            return Err(ErrorCode::InvalidToAccount.into());
        }

        // Execute the mint proposal signed by the bridge.
        let amount = ctx.accounts.proposal.amount;
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.clone(),
            to: ctx.accounts.to.clone(),
            authority: ctx.accounts.bridge_signer.clone(),
        };
        msg!("222");
        let cpi_program = ctx.accounts.token_program.clone();
        let seeds = &[
            ctx.accounts.bridge.to_account_info().key.as_ref(),
            &[ctx.accounts.bridge.nonce],
        ];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        msg!("333");
        token::mint_to(cpi_ctx, amount)?;
        msg!("444");
        // Burn the mint proposal to ensure one time use.
        ctx.accounts.proposal.did_execute = true;

        Ok(())
    }

    // Sets the owners field on the bridge. The only way this can be invoked
    // is via a recursive call from execute_proposal -> set_owners.
    pub fn set_owners(ctx: Context<AdminAuth>, owners: Vec<Pubkey>) -> Result<()> {
        let owners_len = owners.len() as u64;
        if owners_len == 0 {
            return Err(ErrorCode::InvalidOwnerLength.into());
        }

        let bridge = &mut ctx.accounts.bridge;
        if owners_len < bridge.threshold {
            bridge.threshold = owners_len;
        }

        bridge.owners = owners;
        bridge.owner_set_seqno += 1;
        Ok(())
    }

    // Changes the execution threshold of the bridge. The only way this can be
    // invoked is via a recursive call from execute_proposal ->
    // change_threshold.
    pub fn change_threshold(ctx: Context<AdminAuth>, threshold: u64) -> Result<()> {
        if threshold == 0 {
            return Err(ErrorCode::InvalidThreshold.into());
        }
        if threshold > ctx.accounts.bridge.owners.len() as u64 {
            return Err(ErrorCode::InvalidThreshold.into());
        }
        let bridge = &mut ctx.accounts.bridge;
        bridge.threshold = threshold;
        Ok(())
    }
}


#[derive(Accounts)]
pub struct MultiSigAuth<'info> {
    #[account(mut)]
    bridge: ProgramAccount<'info, Bridge>,
    #[account(
        signer, 
        seeds = [bridge.to_account_info().key.as_ref()],
        bump = bridge.nonce,
    )]
    bridge_signer: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AdminAuth<'info> {
    #[account(mut)]
    bridge: ProgramAccount<'info, Bridge>,
    #[account(signer, constraint = &bridge.admin == admin.key)]
    admin: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateBridge<'info> {
    #[account(zero)]
    bridge: ProgramAccount<'info, Bridge>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateDeposit<'info> {
    #[account(mut)]
    bridge: ProgramAccount<'info, Bridge>,
    #[account(zero)]
    deposit: ProgramAccount<'info, Deposit>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,//token account owner
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub from: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateMintProposal<'info> {
    bridge: ProgramAccount<'info, Bridge>,
    #[account(zero)]
    proposal: ProgramAccount<'info, MintProposal>,
    // One of the owners. Checked in the handler.
    #[account(signer)]
    proposer: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(constraint = bridge.owner_set_seqno == proposal.owner_set_seqno)]
    bridge: ProgramAccount<'info, Bridge>,
    #[account(
        seeds = [bridge.to_account_info().key.as_ref()],
        bump = bridge.nonce,
    )]
    bridge_signer: AccountInfo<'info>,
    #[account(mut, has_one = bridge)]
    proposal: ProgramAccount<'info, MintProposal>,
    // One of the bridge owners. Checked in the handler.
    #[account(signer)]
    approver: AccountInfo<'info>,

    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[account]
pub struct Bridge {
    pub admin: Pubkey,
    pub owners: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u8,
    pub owner_set_seqno: u32,
    // destinationChainID => number of deposits
    pub deposit_counts: BTreeMap<u8, u64>,
    // resource id => token mint address
    pub resource_id_to_mint: BTreeMap<[u8;32], Pubkey>,
}

#[account]
pub struct Deposit {
    pub amount: u64,
    pub depositer: Pubkey,
    pub receiver: Vec<u8>,
    pub dest_chain_id: u8,
    pub resource_id: [u8;32],
    pub deposit_nonce: u64,
}

#[account]
pub struct MintProposal {
    // The bridge account this proposal belongs to.
    pub bridge: Pubkey,
    // signers[index] is true if bridge.owners[index] signed the proposal.
    pub signers: Vec<bool>,
    // Boolean ensuring one time execution.
    pub did_execute: bool,
    // Owner set sequence number.
    pub owner_set_seqno: u32,
    // spl mint account
    pub mint: Pubkey,
    // mint to account
    pub to: Pubkey,
    // mint account
    pub amount: u64,
    //spl token program
    pub token_program: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProposalAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&ProposalAccount> for AccountMeta {
    fn from(account: &ProposalAccount) -> AccountMeta {
        match account.is_writable {
            false => AccountMeta::new_readonly(account.pubkey, account.is_signer),
            true => AccountMeta::new(account.pubkey, account.is_signer),
        }
    }
}

impl From<&AccountMeta> for ProposalAccount {
    fn from(account_meta: &AccountMeta) -> ProposalAccount {
        ProposalAccount {
            pubkey: account_meta.pubkey,
            is_signer: account_meta.is_signer,
            is_writable: account_meta.is_writable,
        }
    }
}


impl<'a, 'b, 'c, 'info> From<&mut CreateDeposit<'info>> for CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
    fn from(accounts: &mut CreateDeposit<'info>) -> CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: accounts.mint.clone(),
            to: accounts.from.clone(),
            authority: accounts.authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}


// impl<'a, 'b, 'c, 'info> From<&mut Approve<'info>>
//     for CpiContext<'a, 'b, 'c, 'info, MintTo<'info>>
// {
//     fn from(accounts: &mut Approve<'info>) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
//         let cpi_accounts = MintTo {
//             mint: accounts.mint.clone(),
//             to: accounts.to.clone(),
//             authority: accounts.bridge_signer.clone(),
//         };
//         let cpi_program = accounts.token_program.clone();
//         let seeds = &[
//             accounts.bridge.to_account_info().key().as_ref(),
//             &[accounts.bridge.nonce],
//         ];
//         let signer = &[&seeds[..]];
//         CpiContext::new_with_signer(cpi_program, cpi_accounts, signer)
//     }
// }


#[error]
pub enum ErrorCode {
    #[msg("The given owner is not part of this bridge.")]
    InvalidOwner,
    #[msg("The given owners is empty.")]
    InvalidOwnerLength,
    #[msg("Not enough owners signed this proposal.")]
    NotEnoughSigners,
    #[msg("Cannot delete a proposal that has been signed by an owner.")]
    ProposalAlreadySigned,
    #[msg("Overflow when adding.")]
    Overflow,
    #[msg("Cannot delete a proposal the owner did not create.")]
    UnableToDelete,
    #[msg("The given proposal has already been executed.")]
    AlreadyExecuted,
    #[msg("Threshold must be less than or equal to the number of owners.")]
    InvalidThreshold,
    #[msg("program id account data must have same length")]
    ParamLength,
    #[msg("chain id not found")]
    InvalidChainId,
    #[msg("resource id not found")]
    InvalidResourceId,
    #[msg("mint account not match proposal's mint")]
    InvalidMintAccount,
    #[msg("to account not match proposal's to")]
    InvalidToAccount,
}
