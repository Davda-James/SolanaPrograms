use anchor_lang::prelude::*;
use mpl_core::{ID as CORE_PROGRAM_ID, instructions::CreateCollectionV2CpiBuilder};

use crate::state::CollectionInfo;
use crate::error::StakeError::*;


#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateCollectionArgs {
    pub name: String,
    pub uri: String,
    pub nft_name: String,
    pub nft_uri: String,
}

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint = collection.data_is_empty() @ CollectionAlreadyInitialized
    )]
    pub collection: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = CollectionInfo::DISCRIMINATOR.len() + CollectionInfo::INIT_SPACE,
        seeds = [b"collection_info", collection.key().as_ref()],
        bump
    )]
    pub collection_info: Account<'info, CollectionInfo>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: Verified by addressc
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateCollection<'info> {
    pub fn create_collection(&mut self, args: CreateCollectionArgs, bumps: &CreateCollectionBumps) -> Result<()>{
        self.collection_info.set_inner(CollectionInfo {
            collection: self.collection.key(),
            authority: self.authority.key(),
            name: args.name.clone(),
            uri: args.uri.clone(),
            nft_name: args.nft_name,
            nft_uri: args.nft_uri,
            bump: bumps.collection_info
        });
        let collection_key = self.collection.key();
        let signer_seeds: &[&[&[u8]]] = &[&[b"collection_info", collection_key.as_ref(), &[bumps.collection_info]]];
        CreateCollectionV2CpiBuilder::new(&self.core_program.to_account_info())
        .collection(&self.collection.to_account_info())
        .payer(&self.authority.to_account_info())
        .update_authority(Some(&self.collection_info.to_account_info()))
        .system_program(&self.system_program.to_account_info())
        .name(args.name.clone())
        .uri(args.uri.clone())
        .plugins(vec![])
        .external_plugin_adapters(vec![])
        .invoke_signed(signer_seeds)?;
        
        Ok(())
    }
}
