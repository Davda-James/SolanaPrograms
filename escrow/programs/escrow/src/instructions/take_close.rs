use anchor_spl::token::{self, CloseAccount, Token, TokenAccount, Mint};
use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct TakeClose<'info> {
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        close = maker
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::authority = escrow,
        associated_token::mint = mint_a
    )]
    pub vault: Account<'info, TokenAccount>,

    pub mint_a: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}
impl<'info> TakeClose<'info> {
    pub fn take_close(&mut self) -> Result<()> {
    
        let seed = self.escrow.seed.to_le_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            self.escrow.maker.as_ref(),
            seed.as_ref(),
            &[self.escrow.bump],
        ]];
    
        let cpi_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
    
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
    
        token::close_account(cpi_ctx)
    }
}
