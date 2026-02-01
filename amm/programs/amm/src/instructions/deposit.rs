use crate::error::AMMError;
use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        has_one = mint_x,
        has_one = mint_y,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = depositer,
    )]
    pub depositer_ata_x: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = depositer,
    )]
    pub depositer_ata_y: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositer,
        associated_token::mint = mint_lp,
        associated_token::authority = depositer,
    )]
    pub depositer_ata_lp: Account<'info, TokenAccount>,

    #[account(mut)]
    pub depositer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Deposit<'info> {
    pub fn deposit_to_pool(&mut self, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        require!(self.config.locked == false, AMMError::ConfigLocked);
        require!(amount > 0, AMMError::InvalidAmount);

        let (x, y) = match self.mint_lp.supply == 0
            && self.vault_x.amount == 0
            && self.vault_y.amount == 0
        {
            true => (max_x, max_y),
            false => {
                let amounts = match ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    6,
                ) {
                    Ok(a) => a,
                    Err(_) => return err!(AMMError::InsufficientLiquidity),
                };
                (amounts.x, amounts.y)
            }
        };

        require!(x <= max_x && y <= max_y, AMMError::ExceededMaxSlippage);
        self.deposit_tokens(true, x)?;
        self.deposit_tokens(false, y)?;
        self.mint_lp_tokens(amount)?;

        Ok(())
    }
    pub fn deposit_tokens(&mut self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to) = match is_x {
            true => (
                self.depositer_ata_x.to_account_info(),
                self.vault_x.to_account_info(),
            ),
            false => (
                self.depositer_ata_y.to_account_info(),
                self.vault_y.to_account_info(),
            ),
        };

        let cpi_accounts = Transfer {
            from,
            to,
            authority: self.depositer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
    pub fn mint_lp_tokens(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.depositer_ata_lp.to_account_info(),
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
        mint_to(cpi_ctx, amount)?;
        Ok(())
    }
}
