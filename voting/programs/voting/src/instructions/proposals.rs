use anchor_lang::prelude::*;
use crate::{state::*, errors::ErrorType::*, events::*};
use anchor_lang::solana_program::{system_instruction, program};

#[derive(Accounts)]
pub struct InitializeProposal <'info>{
    #[account(
        init,
        payer = creator,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [b"Proposals", election.key().as_ref(), &election.proposal_count.to_le_bytes()],
        bump 
    )]
    proposal: Account<'info, Proposal>,

    #[account()]
    state: Account<'info, State>,

    #[account(mut)]
    fee_vault: Account<'info, FeeVault>,

    #[account(mut, has_one = owner)]
    election: Account<'info, Election>,
    /// CHECK: no requirement of owner here
    #[account(mut)]
    owner: UncheckedAccount<'info>,

    #[account(mut)]
    creator: Signer<'info>,

    system_program: Program<'info, System>
}

pub fn create_new_proposal(ctx: Context<InitializeProposal>, name: String) -> Result<()> {
    let state= &mut ctx.accounts.state;
    let fee_vault = &mut ctx.accounts.fee_vault;
    let proposal = &mut ctx.accounts.proposal;
    let election = &mut ctx.accounts.election;
    let creator = &ctx.accounts.creator;
    let owner = &mut ctx.accounts.owner;

    let platform_proposal_share = election.proposal_fee.checked_mul((state.platform_proposal_bps / 100) as u64).ok_or(error!(ValueOverflowed))?;
    let election_owner_proposal_share = election.proposal_fee - platform_proposal_share;

    // transferring amount to the fee vault (admin) as per proposal share of platform
    let ix_fee_vault = system_instruction::transfer(
        &creator.key(),
        &fee_vault.key(),
        platform_proposal_share
    );
    let _ = program::invoke(
        &ix_fee_vault,
        &[
            creator.to_account_info(),
            fee_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info()            
        ]
    );

    // transfer the proposal fee share from the proposer (creator) to the election owner (reward)
    let ix = system_instruction::transfer(
        &creator.key(),
        &owner.key(),
        election_owner_proposal_share,
    );
    let _ = program::invoke(
        &ix,
        &[
            creator.to_account_info(),
            owner.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    proposal.election = election.key();
    proposal.creator = creator.key();
    proposal.name = name;
    proposal.vote_count = 0;

    election.proposal_count = election
        .proposal_count
        .checked_add(1)
        .ok_or(error!(ProposalCountOverflowed))?;

    emit!(ProposalCreated {
        election: election.key(),
        creator: creator.key(),
        vote_count: proposal.vote_count
    });

    Ok(())
}