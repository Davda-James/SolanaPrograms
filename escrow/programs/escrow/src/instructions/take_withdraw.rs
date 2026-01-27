use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};
use crate::state::*; 

#[derive(Accounts)]
pub struct TakeWithdraw<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    pub mint_a: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = taker
    )]
    pub taker_ata_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        has_one = mint_a,
        seeds = [b"escrow", escrow.maker.as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

impl<'info> TakeWithdraw<'info> {
    pub fn take_withdraw(&mut self) -> Result<()> {
    
        let seed = self.escrow.seed.to_le_bytes();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            self.escrow.maker.as_ref(),
            seed.as_ref(),
            &[self.escrow.bump],
        ]];
    
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };
    
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
    
        token::transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;
        Ok(())
    }
}
