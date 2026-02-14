use crate::state::{Dao, Proposal};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeProposal<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(mut)]
    pub dao_account: Account<'info, Dao>,
    #[account(
        init,
        payer = creator,
        space = Proposal::DISCRIMINATOR.len() + Proposal::INIT_SPACE,
        seeds = [b"proposal".as_ref(), dao_account.key().as_ref(), dao_account.proposal_count.to_le_bytes().as_ref() ,creator.key().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeProposal<'info> {
    pub fn create_proposal(
        &mut self,
        description: String,
        bumps: &InitializeProposalBumps,
    ) -> Result<()> {
        self.proposal.set_inner(Proposal {
            description,
            dao: self.dao_account.key(),
            yes_votes: 0,
            no_votes: 0,
            authority: self.creator.key(),
            bump: bumps.proposal,
        });
        self.dao_account.proposal_count += 1;
        Ok(())
    }
}
