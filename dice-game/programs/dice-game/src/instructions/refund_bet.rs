use anchor_lang::prelude::*;

use crate::error::DiceGameError;
use crate::state::Bet;

#[derive(Accounts)]
pub struct RefundBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        mut,
        has_one = player @ DiceGameError::InvalidPlayer,
        close = player,
        seeds = [b"bet".as_ref(), vault.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub bet: Account<'info, Bet>,
    #[account(
        mut,
        seeds = [b"vault".as_ref(), house.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    /// CHECK: This is good will handle it.
    pub house: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> RefundBet<'info> {
    pub fn refund(&mut self, bumps: &RefundBetBumps) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        require!(current_slot > self.bet.slot && current_slot - self.bet.slot > 1000, DiceGameError::TimeoutNotReached);

        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: self.vault.to_account_info(),
            to: self.player.to_account_info(),
        };
        let house_key = self.house.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault".as_ref(), house_key.as_ref(), &[bumps.vault]]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        anchor_lang::system_program::transfer(cpi_ctx, self.bet.amount)?;
        Ok(())
    }
}
