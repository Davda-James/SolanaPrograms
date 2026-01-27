use anchor_lang::prelude::*;
use mpl_core::{ID as CORE_PROGRAM_ID, 
    types::{Attribute, Attributes, Plugin, PluginAuthorityPair},
    instructions::CreateV2CpiBuilder};
use crate::error::StakeError::*;
use crate::state::*;

#[derive(Accounts)]
pub struct MintNFT<'info> {
    #[account(
        mut,
        constraint = asset.data_is_empty() @ AssetAlreadyInitialized 
    )]
    pub asset: Signer<'info>,

    #[account(mut)]
    pub minter: Signer<'info>, 
    
    #[account(
        mut,
        constraint = collection.owner == &CORE_PROGRAM_ID @ InvalidCollection,
        constraint = !collection.data_is_empty() @ CollectionNotInitialized
    )]
    /// CHECK: verified by core
    pub collection: UncheckedAccount<'info>,

    #[account(
        seeds = [b"collection_info", collection.key().as_ref()],
        bump = collection_info.bump
    )]
    pub collection_info: Account<'info, CollectionInfo>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: verified by core
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>
}

impl<'info> MintNFT<'info> {
    pub fn mint_nft(&mut self) -> Result<()> {
        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_info",
            collection_key.as_ref(),
            &[self.collection_info.bump]
        ]];

        CreateV2CpiBuilder::new(&self.core_program.to_account_info())
        .asset(&self.asset.to_account_info())
        .collection(Some(&self.collection.to_account_info()))
        .authority(Some(&self.collection_info.to_account_info()))
        .payer(&self.minter.to_account_info())
        .update_authority(None)
        .system_program(&self.system_program.to_account_info())
        .name(self.collection_info.nft_name.clone())
        .uri(self.collection_info.uri.clone())
        .plugins(vec![PluginAuthorityPair {
            plugin: Plugin::Attributes(Attributes {
                attribute_list: vec![
                    Attribute {
                        key: "Minter".to_string(),
                        value: self.minter.key().to_string()
                    },
                    Attribute {
                        key: "Timestamp".to_string(),
                        value: Clock::get()?.unix_timestamp.to_string()
                    }
                ]
            }),
            authority: None
        }])
        .external_plugin_adapters(vec![])
        .invoke_signed(signer_seeds)?;

        Ok(())
    }
}