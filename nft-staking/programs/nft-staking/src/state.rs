use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct CollectionInfo {
    pub collection: Pubkey,
    pub authority: Pubkey,
    #[max_len(32)]
    pub name: String,
    #[max_len(200)]
    pub uri: String,
    #[max_len(32)]
    pub nft_name: String,
    #[max_len(200)]
    pub nft_uri: String,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub staked_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct StakeConfig {
    pub admin: Pubkey,
    pub points_per_stake: u8,
    pub max_stake: u8,
    pub freeze_period: u32,
    pub rewards_bump: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub points: u32,
    pub amount_staked: u8,
    pub bump: u8,
}
