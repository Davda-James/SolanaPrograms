use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::state::Escrow;

#[derive(Accounts)]
pub struct TakeDeposit<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    pub mint_b: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = taker
    )]
    pub taker_ata_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = maker
    )]
    pub maker_ata_b: Account<'info, TokenAccount>,

    #[account(
        has_one = mint_b,
        has_one = maker,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    pub token_program: Program<'info, Token>,
}
impl <'info> TakeDeposit<'info> {
    pub fn take_deposit(&mut self) -> Result<()> {
    
        let cpi_accounts = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };
    
        let cpi_ctx =
            CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
    
        token::transfer_checked(cpi_ctx, self.escrow.receive,self.mint_b.decimals)?;
        Ok(())
    }
}
