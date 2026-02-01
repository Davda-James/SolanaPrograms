use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("Config is locked")]
    ConfigLocked,
    #[msg("Invalid amount provided")]
    InvalidAmount,
    #[msg("Insufficient liquidity provided")]
    InsufficientLiquidity,
    #[msg("Exceeded maximum slippage")]
    ExceededMaxSlippage,
}
