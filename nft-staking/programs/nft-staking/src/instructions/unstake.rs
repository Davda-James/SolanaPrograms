use anchor_lang::prelude::*;
use mpl_core::instructions::{RemovePluginV1CpiBuilder, UpdatePluginV1CpiBuilder};
use mpl_core::types::{FreezeDelegate, Plugin, PluginType};
use mpl_core::{ID as CORE_PROGRAM_ID};

use crate::state::{StakeConfig, StakeAccount, UserAccount};
use crate::StakeError;

#[derive(Accounts)]
pub struct Unstake<'info> {
     #[account(mut)]
    pub staker: Signer<'info>,
    #[account(
        mut,
        close = staker,
        seeds= [b"stake", config.key().as_ref(), asset.key().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == staker.key() @ StakeError::NotOwner
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

impl<'info> Unstake<'info> {
    pub fn unstake_staked_nft(&mut self) -> Result<()> {
        let time_elapsed = ((Clock::get()?.unix_timestamp-self.stake_account.staked_at)/86400) as u32;
        require!(time_elapsed >= self.config.freeze_period, StakeError::FreezePeriodNotPassed);
        
        let points = (self.config.points_per_stake as u32) * time_elapsed;

        self.user_account.points += points;
        
        let config_key = self.config.key();
        let asset_key = self.asset.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"stake",
            config_key.as_ref(),
            asset_key.as_ref(),
            &[self.stake_account.bump]
        ]];

        UpdatePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.staker.to_account_info())
            .authority(Some(&self.stake_account.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen : false }))
            .invoke_signed(signer_seeds)?;

        RemovePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.staker.to_account_info())
            .authority(None)
            .system_program(&self.system_program.to_account_info())
            .plugin_type(PluginType::FreezeDelegate)
            .invoke()?;
        
        self.user_account.amount_staked -= 1;
        
        Ok(())
    }
}