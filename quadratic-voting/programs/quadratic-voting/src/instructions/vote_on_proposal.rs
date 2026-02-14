use crate::{
    error::QuadraticVotingError,
    state::{Dao, Proposal, Vote},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        constraint = proposal.dao == dao.key() @ QuadraticVotingError::InvalidDao
    )]
    pub dao: Account<'info, Dao>,
    #[account(
        mut,
        has_one = dao
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        init,
        payer = voter,
        space = Vote::DISCRIMINATOR.len() + Vote::INIT_SPACE,
        seeds = [b"vote", voter.key().as_ref(), proposal.key().as_ref()],
        bump,
    )]
    pub vote_account: Account<'info, Vote>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = voter
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> VoteOnProposal<'info> {
    pub fn cast_vote(&mut self, vote: bool, votes: u64, bumps: &VoteOnProposalBumps) -> Result<()> {
        let cost = votes
            .checked_mul(votes)
            .ok_or(QuadraticVotingError::Overflow)?;
        require!(
            self.creator_token_account.amount >= cost,
            QuadraticVotingError::InsufficientTokens
        );

        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            Burn {
                mint: self.mint.to_account_info(),
                from: self.creator_token_account.to_account_info(),
                authority: self.voter.to_account_info(),
            },
        );
        burn(cpi_ctx, cost)?;

        self.vote_account.set_inner(Vote {
            authority: self.voter.key(),
            vote_type: vote,
            vote_credits: votes,
            bump: bumps.vote_account,
        });
        if vote {
            self.proposal.yes_votes = self
                .proposal
                .yes_votes
                .checked_add(votes)
                .ok_or(QuadraticVotingError::Overflow)?;
        } else {
            self.proposal.no_votes = self
                .proposal
                .no_votes
                .checked_add(votes)
                .ok_or(QuadraticVotingError::Overflow)?;
        }
        Ok(())
    }
}
