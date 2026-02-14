pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("4v6HnRxKdJwGJjra65yX8tHZjLnfhARYFC1DMokFL5Yd");

#[program]
pub mod dice_game {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        ctx.accounts.init(amount)
    }
    pub fn place_bet(ctx: Context<CreateBet>, seed: u128, amount: u64, choice: u8) -> Result<()> {
        ctx.accounts.place_bet(amount, choice, seed)
    }
    pub fn refund_bet(ctx: Context<RefundBet>) -> Result<()> {
        ctx.accounts.refund(&ctx.bumps)
    }
    pub fn resolve_bet(ctx: Context<ResolveBet>, sig: Vec<u8>) -> Result<()> {
        ctx.accounts.resolve(&sig, &ctx.bumps)
    }
}
