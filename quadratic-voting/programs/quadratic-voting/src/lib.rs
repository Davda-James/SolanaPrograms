pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("7fyX3AdE3tevAjD9tdX1ymKrUgkbBqRGc8PFXJ3noovd");

#[program]
pub mod quadratic_voting {
    use super::*;

    pub fn initialize_dao(ctx: Context<InitializeDao>, name: String) -> Result<()> {
        ctx.accounts.init_dao(name, &ctx.bumps)
    }
    pub fn create_proposal(ctx: Context<InitializeProposal>, description: String) -> Result<()> {
        ctx.accounts.create_proposal(description, &ctx.bumps)
    }
    pub fn vote(ctx: Context<VoteOnProposal>, vote: bool, votes: u64) -> Result<()> {
        ctx.accounts.cast_vote(vote, votes, &ctx.bumps)
    }
}
