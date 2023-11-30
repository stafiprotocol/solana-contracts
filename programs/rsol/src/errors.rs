use anchor_lang::prelude::*;

#[error_code]
pub enum Errors {
    #[msg("Program id not match")]
    ProgramIdNotMatch,

    #[msg("Remaining accounts not match")]
    RemainingAccountsNotMatch,

    #[msg("Admin not match")]
    AdminNotMatch,

    #[msg("Delegation empty")]
    DelegationEmpty,

    #[msg("Stake amount too low")]
    StakeAmountTooLow,

    #[msg("Stake account not active")]
    StakeAccountNotActive,

    #[msg("Stake account with lockup")]
    StakeAccountWithLockup,

    #[msg("Pool not exist")]
    PoolNotExist,

    #[msg("Validator not exist")]
    ValidatorNotExist,

    #[msg("Stake account already exist")]
    StakeAccountAlreadyExist,
}
