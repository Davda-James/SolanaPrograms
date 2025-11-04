use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ErrorType::*;
use crate::events::*;

#[derive(Accounts)]
pub struct VoteProposal<'info> {
    #[account(mut)]
    voter_account: Account<'info, Voter>,

    #[account()]
    election: Account<'info, Election>,

    #[account(mut)]
    proposal: Account<'info, Proposal>, 

    #[account(mut)]
    voter: Signer<'info>,
}

pub fn vote_for_proposal(ctx: Context<VoteProposal>) -> Result<()> {
    let voter_account: &mut Account<'_, Voter> = &mut ctx.accounts.voter_account;
    let election = &mut ctx.accounts.election;
    let proposal = &mut ctx.accounts.proposal;

    require_keys_eq!(voter_account.voter, ctx.accounts.voter.key(), VoterUnauthorized);
    require_eq!(voter_account.has_voted, false, AlreadyVoted);
    proposal.vote_count = proposal.vote_count.checked_add(1).ok_or(error!(VoteCountOverflowed))?;
    voter_account.has_voted = true;

    emit!(VotedForProposal {
        election: election.key(),
        voter:    voter_account.voter.key(),
        proposal: Some(proposal.key()),
        has_voted: voter_account.has_voted,
    });

    Ok(())
}
