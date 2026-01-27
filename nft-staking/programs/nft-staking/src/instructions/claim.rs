use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token::{mint_to, Token, Mint, MintTo, TokenAccount}};
    
use crate::error::StakeError;
use crate::state::{StakeConfig, UserAccount};

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub rewards_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = claimer,
        associated_token::mint = rewards_mint,
        associated_token::authority = claimer,
    )] 
    pub associated_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, StakeConfig>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

impl<'info> Claim<'info> {
    pub fn claim_reward(&mut self) -> Result<()> {
        require!(self.user_account.points > 0, StakeError::NoPointsToClaim);
        let signer_seeds:&[&[&[u8]]] = &[&[
            b"config",
            &[self.config.bump]
        ]];
        
        let cpi_accounts = MintTo {
            mint: self.rewards_mint.to_account_info(),
            to: self.associated_token_account.to_account_info(),
            authority: self.config.to_account_info()
        };
        let cpi_ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer_seeds);
        mint_to(cpi_ctx, self.user_account.points as u64)?;

        self.user_account.points = 0; 

        Ok(())
    }
}