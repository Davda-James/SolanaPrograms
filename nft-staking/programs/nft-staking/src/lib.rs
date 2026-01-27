pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;
pub use error::*;

declare_id!("DuQMkqKZRqh7UmA2UV9JgsZLPFYqUuTvcJjiwPUKzAeo");

#[program]
pub mod nft_staking {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        points_per_stake: u8,
        max_stake: u8,
        freeze_period: u32,
    ) -> Result<()> {
        ctx.accounts
            .initialize_config(points_per_stake, max_stake, freeze_period, &ctx.bumps)
    }

    pub fn initialize_user(ctx: Context<InitializeUser>) -> Result<()> {
        ctx.accounts.initialize_user_account(&ctx.bumps)
    }

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        args: CreateCollectionArgs,
    ) -> Result<()> {
        ctx.accounts.create_collection(args, &ctx.bumps)
    }

    pub fn mint_nft(ctx: Context<MintNFT>) -> Result<()> {
        ctx.accounts.mint_nft()
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake_nft(&ctx.bumps)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake_staked_nft()
    }
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        ctx.accounts.claim_reward()
    }
}
