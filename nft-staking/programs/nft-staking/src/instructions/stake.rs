use anchor_lang::prelude::*;
use mpl_core::{
    ID as CORE_PROGRAM_ID, instructions::{AddPluginV1CpiBuilder}, types::{FreezeDelegate, Plugin, PluginAuthority}
};
use crate::{state::{UserAccount, StakeAccount, StakeConfig}};
use crate::StakeError;

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,

    #[account(
        init,
        payer = staker,
        seeds= [b"stake", config.key().as_ref(), asset.key().as_ref()],
        bump,
        space = StakeAccount::DISCRIMINATOR.len() + StakeAccount::INIT_SPACE,
    )]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(
        mut,
        seeds = [b"user", staker.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        mut,
        constraint = asset.owner == &CORE_PROGRAM_ID,
        constraint = !asset.data_is_empty() @ StakeError::AssetNotInitialized
    )]
    /// CHECK: constraints check
    pub asset: UncheckedAccount<'info>,
    
    #[account(
        mut,
        constraint = collection.owner == &CORE_PROGRAM_ID,
        constraint = !collection.data_is_empty() @ StakeError::CollectionNotInitialized
    )]
    /// CHECK: constraints check
    pub collection: UncheckedAccount<'info>,
    
    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: mpl-core handles it
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> Stake<'info> {
    pub fn stake_nft(&mut self, bumps: &StakeBumps) -> Result<()> {
        require!(self.config.max_stake > self.user_account.amount_staked, StakeError::MaxStakeReached);

        AddPluginV1CpiBuilder::new(&self.core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .payer(&self.staker.to_account_info())
        .system_program(&self.system_program)
        .authority(None)
        .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
        .init_authority(PluginAuthority::Address { address: self.stake_account.key() })
        .invoke()?;
        
        self.stake_account.set_inner(StakeAccount { 
            owner: self.staker.key(), 
            mint: self.asset.key(),
            staked_at: Clock::get()?.unix_timestamp, 
            bump: bumps.stake_account 
        });

        self.user_account.amount_staked += 1;
        
        Ok(())          
    }
}

