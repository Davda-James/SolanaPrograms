use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Invalid Amount")]
    InvalidAmount,
}
