use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Program id not match")]
    ProgramIdNotMatch,

    #[msg("Remaining accounts not match")]
    RemainingAccountsNotMatch,

    #[msg("Admin not match")]
    AdminNotMatch,
    
    #[msg("Invalid token account data")]
    InvalidTokenAccountData,
    
    #[msg("Invalid ext mint authority")]
    InvalidExtMintAuthority,
}
