use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct State {
    pub admin: Pubkey,
    pub platform_fee: u64,
    pub election_count: u64,
    pub platform_proposal_bps: u16
}   

#[account]
#[derive(InitSpace)]
pub struct FeeVault {
    pub bump: u8,
    pub total_fees: u64
}

#[account]
#[derive(InitSpace)]
pub struct Election {
    pub owner: Pubkey,
    #[max_len(100)]
    pub name: String,
    pub proposal_count: u64,
    pub proposal_fee: u64,       
}

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub election: Pubkey,
    pub creator: Pubkey,
    #[max_len(100)]
    pub name: String,
    pub vote_count: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Voter {
    pub election: Pubkey,
    pub voter: Pubkey,
    pub has_voted: bool,
    pub proposal_voted_for: Option<Pubkey>
}
