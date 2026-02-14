use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Dao {
    #[max_len(256)]
    pub name: String,
    pub authority: Pubkey,
    pub proposal_count: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub authority: Pubkey,
    pub dao: Pubkey,
    #[max_len(256)]
    pub description: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Vote {
    pub authority: Pubkey,
    pub vote_type: bool,
    pub vote_credits: u64,
    pub bump: u8,
}
