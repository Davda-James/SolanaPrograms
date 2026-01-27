use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{AssociatedToken},
    token_interface::{TokenAccount, TokenInterface, CloseAccount, Mint, transfer_checked, TransferChecked, close_account}
};

use crate::state::Escrow;

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), &escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken> 
}

impl<'info> Refund <'info> {
    pub fn refund_and_close_vault(&mut self)-> Result<()> {
        let escrow_seed = self.escrow.seed.to_le_bytes();
        let maker_key = self.maker.key();
        let signer_seed: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(), 
            &escrow_seed.as_ref(), 
            &[self.escrow.bump]
        ]];

        let transfer_accounts = TransferChecked {
            mint: self.mint_a.to_account_info(),
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), transfer_accounts, signer_seed);
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        let close_cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), close_accounts, signer_seed);
    
        close_account(close_cpi_ctx)
    }
}


