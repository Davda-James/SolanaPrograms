use anchor_lang::prelude::*;

#[error_code]
pub enum QuadraticVotingError {
    #[msg("Overflow Occured")]
    Overflow,
    #[msg("InsufficientTokens")]
    InsufficientTokens,
    #[msg("Dao for proposal did not match")]
    InvalidDao,
}
