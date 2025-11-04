use anchor_lang::prelude::*;

#[event]
pub struct GlobalStateInitialized {
    pub admin: Pubkey,
    pub fee_vault: Pubkey,
    pub platform_fee: u64,
    pub platform_proposal_bps: u16,
    pub election_count: u64
}

#[event]
pub struct ElectionInitialized {
    pub owner: Pubkey,  
    pub proposal_count: u64,
    pub proposal_rate: u64,
}

#[event]
pub struct ProposalCreated {
    pub election: Pubkey,
    pub creator: Pubkey,
    pub vote_count: u64
}

#[event]
pub struct VoterInitialized {
    pub election: Pubkey,
    pub voter: Pubkey,
    pub proposal : Option<Pubkey>,
    pub has_voted: bool
}

#[event]
pub struct VotedForProposal {
    pub election: Pubkey,
    pub voter: Pubkey,
    pub proposal: Option<Pubkey>,
    pub has_voted: bool
}

#[event]
pub struct PlatformFeeChanged {
    pub admin: Pubkey,
    pub platform_fee: u64
}

#[event]
pub struct PlatformProposalBFSChanged {
    pub admin: Pubkey,
    pub platform_proposal_bfs: u16
}