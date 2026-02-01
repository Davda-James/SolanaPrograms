pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("4f4DhXC59uftcSi5wZzVQVBEDnU6AV6qb8NQmgbuwSDZ");

#[program]
pub mod amm {
    use super::*;
    pub fn initialize_config(ctx: Context<InitializeConfig>, seed: u64, fee: u16) -> Result<()> {
        ctx.accounts.init_config(seed, fee, &ctx.bumps)
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        ctx.accounts.deposit_to_pool(amount, max_x, max_y)
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        ctx.accounts.withdraw(amount, min_x, min_y)
    }
    pub fn swap(
        ctx: Context<Swap>,
        is_x_to_y: bool,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        ctx.accounts
            .swap_tokens(is_x_to_y, amount_in, min_amount_out)
    }
}
