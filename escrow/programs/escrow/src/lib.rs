use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;

pub use instructions::*;
pub use error::EscrowError;

declare_id!("EWAghmkH9oRqUDUWSQv6Fob8LHyvb3YbHhrnm5R8YtJd");

#[program]
pub mod escrow {
    use super::*;

    pub fn do_make(ctx: Context<Make>, seed: u64, deposit: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)?;
        Ok(())
    }
    pub fn do_refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
    pub fn do_take_deposit(ctx: Context<TakeDeposit>) -> Result<()> {
        ctx.accounts.take_deposit()
    }
    pub fn do_take_withdraw(ctx: Context<TakeWithdraw>) -> Result<()> {
        ctx.accounts.take_withdraw()
    }
    pub fn do_take_close(ctx: Context<TakeClose>) -> Result<()> {
        ctx.accounts.take_close()
    }
}
