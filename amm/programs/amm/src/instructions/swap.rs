use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = user
    )]
    pub user_ata_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = user
    )]
    pub user_ata_y: Account<'info, TokenAccount>,

    #[account(
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap_tokens(
        &mut self,
        is_x_to_y: bool,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        let amount_out = match is_x_to_y {
            true => ConstantProduct::delta_y_from_x_swap_amount(
                self.vault_x.amount,
                self.vault_y.amount,
                amount_in,
            )
            .unwrap(),
            false => ConstantProduct::delta_x_from_y_swap_amount(
                self.vault_x.amount,
                self.vault_y.amount,
                amount_in,
            )
            .unwrap(),
        };
        require!(
            amount_out >= min_amount_out,
            crate::error::AMMError::ExceededMaxSlippage
        );

        self.deposit_tokens(is_x_to_y, amount_in)?;
        self.withdraw_tokens(!is_x_to_y, amount_out)?;

        Ok(())
    }
    pub fn deposit_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.user_ata_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.user_ata_y.to_account_info(),
                self.vault_y.to_account_info(),
            ),
        };

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
    pub fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.user_ata_x.to_account_info(),
            ),
            false => (
                self.vault_y.to_account_info(),
                self.user_ata_y.to_account_info(),
            ),
        };
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };
        let config_seed = self.config.seed.to_le_bytes();
        let seeds: &[&[&[u8]]] = &[&[b"config", config_seed.as_ref(), &[self.config.config_bump]]];
        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, seeds);
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
