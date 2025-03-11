use anchor_lang::prelude::*;

#[error_code]
pub enum IdoError {
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid presale status")]
    InvalidPresaleStatus,
    
    #[msg("Presale not started yet")]
    PresaleNotStarted,
    
    #[msg("Presale already ended")]
    PresaleEnded,
    
    #[msg("Insufficient token amount to qualify for tier")]
    InsufficientTierQualification,
    
    #[msg("Tier allocation limit reached")]
    TierAllocationLimitReached,
    
    #[msg("Invalid vesting schedule")]
    InvalidVestingSchedule,
    
    #[msg("Nothing to claim at the moment")]
    NothingToClaim,
    
    #[msg("Presale time setup is invalid")]
    InvalidTimeSetup,
    
    #[msg("Token decimal mismatch")]
    TokenDecimalMismatch,
    
    #[msg("Insufficient allocation")]
    InsufficientAllocation,
    
    #[msg("Presale is not completed yet")]
    PresaleNotCompleted,
    
    #[msg("Vesting not started yet")]
    VestingNotStarted,
    
    #[msg("Claim already processed")]
    ClaimAlreadyProcessed,
    
    #[msg("Presale registration period ended")]
    RegistrationEnded,
    
    #[msg("Presale registration period not started")]
    RegistrationNotStarted,

    #[msg("Token has been listed to the market already")]
    TokenAlreadyListed,
}