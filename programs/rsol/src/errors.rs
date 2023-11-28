use anchor_lang::prelude::*;

#[error_code]
pub enum Error {
    #[msg("Program id not match")]
    ProgramIdNotMatch,

    #[msg("Remaining accounts not match")]
    RemainingAccountsNotMatch,
}
