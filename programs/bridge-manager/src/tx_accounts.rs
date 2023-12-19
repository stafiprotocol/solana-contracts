use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_spl::token::{Mint, Token, TokenAccount};
use mint_manager::program::MintManagerProgram;
use mint_manager::{self, MintManagerAccount};
use std::convert::Into;
use crate::states::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct AdminAuth<'info> {
    #[account(
        mut, 
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub bridge: Box<Account<'info, BridgeManagerAccount>>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetMintAuthority<'info> {
    #[account(
        has_one = admin @ Errors::AdminNotMatch
    )]
    pub bridge: Box<Account<'info, BridgeManagerAccount>>,

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
    pub bridge: Box<Account<'info, BridgeManagerAccount>>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct TransferOut<'info> {
    #[account(mut)]
    pub bridge: Box<Account<'info, BridgeManagerAccount>>,

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
    pub bridge: Box<Account<'info, BridgeManagerAccount>>,
    #[account(zero)]
    pub proposal: Box<Account<'info, MintProposalAccount>>,

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
    pub bridge: Box<Account<'info, BridgeManagerAccount>>,

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
    pub proposal: Box<Account<'info, MintProposalAccount>>,

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

    pub mint_manager: Box<Account<'info, MintManagerAccount>>,

    /// CHECK:  check on mint manager program
    pub mint_authority: UncheckedAccount<'info>,

    pub mint_manager_program: Program<'info, MintManagerProgram>,
    pub token_program: Program<'info, Token>,
}


