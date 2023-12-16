use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_spl::token::Burn;
use anchor_spl::token::{Mint, Token, TokenAccount};
use minter::program::Minter;
use minter::{self, MintManager};
use std::collections::BTreeMap;
use std::convert::Into;
#[derive(Accounts)]
pub struct AdminAuth<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub bridge: Box<Account<'info, Bridge>>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetMintAuthority<'info> {
    #[account(
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub bridge: Box<Account<'info, Bridge>>,

    pub admin: Signer<'info>,

    /// CHECK: pda
    #[account(
        seeds = [
            &bridge.key().to_bytes()
        ],
        bump = bridge.nonce,
    )]
    pub bridge_signer: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateBridge<'info> {
    #[account(zero)]
    pub bridge: Box<Account<'info, Bridge>>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TransferOut<'info> {
    #[account(mut)]
    pub bridge: Box<Account<'info, Bridge>>,

    /// CHECK: token account's owner
    #[account(
        signer, 
        mut
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint,
    )]
    pub from: Box<Account<'info, TokenAccount>>,

    /// CHECK: fee receiver
    #[account(mut)]
    pub fee_receiver: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMintProposal<'info> {
    pub bridge: Box<Account<'info, Bridge>>,
    #[account(zero)]
    pub proposal: Box<Account<'info, MintProposal>>,

    // token account which has been initiated
    pub to: Box<Account<'info, TokenAccount>>,

    // One of the owners. Checked in the handler.
    #[account(
        owner = system_program::ID
    )]
    pub proposer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Approve<'info> {
    #[account(
        constraint = bridge.owner_set_seqno == proposal.owner_set_seqno
    )]
    pub bridge: Box<Account<'info, Bridge>>,

    /// CHECK: pda
    #[account(
        seeds = [
            &bridge.key().to_bytes()
        ],
        bump = bridge.nonce,
    )]
    pub bridge_signer: UncheckedAccount<'info>,

    #[account(
        mut, 
        has_one = bridge
    )]
    pub proposal: Box<Account<'info, MintProposal>>,

    // One of the bridge owners. Checked in the handler.
    #[account(
        owner = system_program::ID
    )]
    pub approver: Signer<'info>,

    #[account(
        mut,
        address = mint_manager.rsol_mint
    )]
    pub mint: Box<Account<'info, Mint>>,

    // token account which has been initiated
    #[account(
        mut,
        token::mint = mint_manager.rsol_mint
    )]
    pub to: Box<Account<'info, TokenAccount>>,

    pub mint_manager: Box<Account<'info, MintManager>>,

    /// CHECK:  check on minter program
    pub mint_authority: UncheckedAccount<'info>,

    pub minter_program: Program<'info, Minter>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Bridge {
    pub admin: Pubkey,
    pub owners: Vec<Pubkey>,
    pub threshold: u64,
    pub nonce: u8,
    pub owner_set_seqno: u32,
    pub support_chain_ids: Vec<u8>,
    // destinationChainID => number of deposits
    pub deposit_counts: BTreeMap<u8, u64>,
    // resource id => token mint address
    pub resource_id_to_mint: BTreeMap<[u8; 32], Pubkey>,
    pub fee_receiver: Pubkey,
    // destinationChainID => fee amount of sol
    pub fee_amounts: BTreeMap<u8, u64>,
}

#[event]
pub struct EventTransferOut {
    pub transfer: Pubkey,
    pub receiver: Vec<u8>,
    pub amount: u64,
    pub dest_chain_id: u8,
    pub resource_id: [u8; 32],
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

impl<'a, 'b, 'c, 'info> From<&mut TransferOut<'info>>
    for CpiContext<'a, 'b, 'c, 'info, Burn<'info>>
{
    fn from(accounts: &mut TransferOut<'info>) -> CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: accounts.mint.to_account_info(),
            from: accounts.from.to_account_info(),
            authority: accounts.authority.to_account_info(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[error_code]
pub enum Errors {
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
    #[msg("Program id account data must have same length")]
    ParamLength,
    #[msg("Chain id not support")]
    NotSupportChainId,
    #[msg("Chain id exist")]
    ChainIdExist,
    #[msg("Chain id not exist")]
    ChainIdNotExist,
    #[msg("Resource id not found")]
    InvalidResourceId,
    #[msg("Mint account not match proposal's mint")]
    InvalidMintAccount,
    #[msg("To account not match proposal's to")]
    InvalidToAccount,
    #[msg("From account invalid")]
    InvalidFromAccount,
    #[msg("Invalid fee receiver")]
    InvalidFeeReceiver,
    #[msg("Not support mint type")]
    NotSupportMintType,
    #[msg("Admin not match")]
    AdminNotMatch,
}
