use crate::error::AMMError;
use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer, Burn, Mint, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,
    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"lp", config.key().as_ref()],
        bump = config.lp_bump
    )]
    pub mint_lp: Box<Account<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = withdrawer
    )]
    pub withdrawer_ata_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = withdrawer
    )]
    pub withdrawer_ata_y: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = withdrawer
    )]
    pub withdrawer_ata_lp: Account<'info, TokenAccount>,
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

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        require!(
            self.mint_lp.supply > 0 && self.mint_lp.supply >= amount,
            AMMError::InsufficientLiquidity
        );
        let xy_amount = match ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            amount,
            6,
        ) {
            Ok(a) => a,
            Err(_) => return err!(AMMError::InsufficientLiquidity),
        };

        require!(
            xy_amount.x >= min_x && xy_amount.y >= min_y,
            AMMError::ExceededMaxSlippage
        );

        self.burn_lp(amount)?;
        self.withdraw_tokens(true, xy_amount.x)?;
        self.withdraw_tokens(false, xy_amount.y)?;
        Ok(())
    }
    pub fn withdraw_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.vault_x.to_account_info(),
                self.withdrawer_ata_x.to_account_info(),
            ),
            false => (
                self.vault_y.to_account_info(),
                self.withdrawer_ata_y.to_account_info(),
            ),
        };
        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.config.to_account_info(),
        };
        let config_seed = self.config.seed.to_le_bytes();
        let signer_seeds: &[&[&[u8]]] =
            &[&[b"config", config_seed.as_ref(), &[self.config.config_bump]]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
    pub fn burn_lp(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.withdrawer_ata_lp.to_account_info(),
            authority: self.withdrawer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        burn(cpi_ctx, amount)?;
        Ok(())
    }
}
