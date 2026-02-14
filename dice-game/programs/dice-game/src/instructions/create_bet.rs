use crate::error::DiceGameError;
use crate::state::Bet;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(seed: u128)]
pub struct CreateBet<'info> {
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault".as_ref(), house.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        init,
        payer = player,
        seeds = [b"bet".as_ref(), vault.key().as_ref() , seed.to_le_bytes().as_ref()],
        space = Bet::DISCRIMINATOR.len() + Bet::INIT_SPACE,
        bump,
    )]
    pub bet: Account<'info, Bet>,
    /// CHECK: This is good will handle it.
    pub house: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateBet<'info> {
    pub fn place_bet(&mut self, amount: u64, roll: u8, seed: u128) -> Result<()> {
        require!(amount > 0, DiceGameError::InvalidBetAmount);
        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: self.player.to_account_info(),
            to: self.vault.to_account_info(),
        };
        self.bet.set_inner(Bet {
            player: self.player.key(),
            seed,
            slot: Clock::get()?.slot,
            roll: roll,
            amount,
        });

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
        anchor_lang::system_program::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
