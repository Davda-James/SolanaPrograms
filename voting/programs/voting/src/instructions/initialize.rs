use anchor_lang::prelude::*;
use crate::{errors::ErrorType::*, state::*, events::*};
use anchor_lang::solana_program::{system_instruction, program};


#[derive(Accounts)]
pub struct InitializeState<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + State::INIT_SPACE
    )]
    pub state: Account<'info, State>,

    #[account(
        init,
        payer = admin,
        space = 8 + FeeVault::INIT_SPACE,
        seeds = [b"FeeVault", admin.key().as_ref()],
        bump
    )]
    pub fee_vault: Account<'info,FeeVault>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitializeElection<'info> {
    #[account(
        mut, 
        has_one = admin
    )]
    state: Account<'info, State>,

    #[account(
        init,
        payer = creator,
        space = 8 + Election::INIT_SPACE,
        seeds = [b"Election", creator.key().as_ref(), &state.election_count.to_le_bytes()],
        bump
    )]
    election: Account<'info, Election>,

    #[account(mut)]
    creator: Signer<'info>,

    /// CHECK: Here admin consent is notrequired, its safe to take as unchecked
    admin: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub fee_vault: Account<'info, FeeVault>,
    
    system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct InitializeVoter <'info> {
    #[account(
        init,
        payer = voter,
        space =  8 + Voter::INIT_SPACE,
        seeds = [b"Voter", voter.key().as_ref(), election.key().as_ref()],
        bump
    )]
    voter_account: Account<'info, Voter>,
    
    #[account()]
    election: Account<'info,Election>,

    #[account(mut)]
    voter: Signer<'info>,

    system_program: Program<'info, System>
}

pub fn init_global_state(ctx: Context<InitializeState>, platform_fee: u64, platform_proposal_bps: u16) -> Result<()> {
    require_gt!(10000 , platform_proposal_bps, BFSOverflowed);
    let global_state = &mut ctx.accounts.state;
    let fee_vault = &mut ctx.accounts.fee_vault;
    fee_vault.bump = ctx.bumps.fee_vault;
    global_state.election_count = 0;
    global_state.admin = ctx.accounts.admin.key();
    global_state.platform_fee = platform_fee;
    global_state.platform_proposal_bps = platform_proposal_bps;
    emit!( GlobalStateInitialized {
        admin: ctx.accounts.admin.key(),
        fee_vault: fee_vault.key(),
        platform_fee: global_state.platform_fee,
        platform_proposal_bps: platform_proposal_bps,
        election_count: global_state.election_count
    });
    Ok(())
}

pub fn init_election(ctx: Context<InitializeElection>, name: String, proposal_fee: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let election = &mut ctx.accounts.election;
    let fee_vault = &mut ctx.accounts.fee_vault;
    let creator = &ctx.accounts.creator;
    let ix = system_instruction::transfer(
        &creator.key(),
        &fee_vault.key(),
        state.platform_fee
    );
    let _ = program::invoke(
        &ix,
        &[
            creator.to_account_info(),
            fee_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info()
        ]
    );

    fee_vault.total_fees = fee_vault.total_fees.checked_add(proposal_fee).ok_or(error!(FeeVaultOverflowed))?;
    // set the election owner as creator's public key 
    election.owner = creator.key();
    election.name = name;
    election.proposal_count = 0;
    election.proposal_fee = proposal_fee;

    state.election_count += 1;

    emit!( ElectionInitialized{
        owner: election.owner,
        proposal_count: election.proposal_count,
        proposal_rate: election.proposal_fee,
    });

    Ok(())
}

pub fn init_voter(ctx: Context<InitializeVoter>) -> Result<()>{
    let election = &mut ctx.accounts.election;
    let voter_account = &mut ctx.accounts.voter_account;

    voter_account.election = election.key();
    voter_account.voter = ctx.accounts.voter.key();
    voter_account.proposal_voted_for = None;
    voter_account.has_voted = false;

    emit!(VoterInitialized {
        election: voter_account.election,
        voter: voter_account.voter,
        proposal: voter_account.proposal_voted_for,
        has_voted: voter_account.has_voted
    });

    Ok(())
}

