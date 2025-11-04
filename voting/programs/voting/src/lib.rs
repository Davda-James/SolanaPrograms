#![allow(deprecated)]
use anchor_lang::prelude::*;
mod instructions;
mod state;
mod events;
mod errors;

use instructions::*;

declare_id!("2p9jD9dnZ2M6pya8euvB9RTb1qX4G1eQQ1YaeEfSNbV9");

#[program]
pub mod voting {

    use super::*;

    pub fn initialize(ctx: Context<InitializeState>, platform_fee: u64, platform_proposal_bps: u16) -> Result<()> {
        init_global_state(ctx, platform_fee,platform_proposal_bps)
    }
    pub fn initialize_election(ctx: Context<InitializeElection>, name: String, proposal_fee: u64) -> Result<()> {
        init_election(ctx, name, proposal_fee)
    }

    pub fn create_proposal(ctx: Context<InitializeProposal>, name: String) -> Result<()> {
        create_new_proposal(ctx, name)
    }

    pub fn initialize_voter(ctx: Context<InitializeVoter>) -> Result<()> {
        init_voter(ctx)
    }

    pub fn vote_on_proposal(ctx: Context<VoteProposal>) -> Result<()> {
        vote_for_proposal(ctx)
    }
    pub fn change_platform_fee(ctx: Context<PlatformFee>, platform_fee: u64) -> Result<()> {
        change_platform_fee_helper(ctx ,platform_fee)
    }

    pub fn change_platform_proposal_bfs(ctx: Context<PlatformProposalBFS>, platform_proposal_bps: u16) -> Result<()> {
        change_platform_proposal_bfs_helper(ctx, platform_proposal_bps)
    }
}
