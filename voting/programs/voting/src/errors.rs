use anchor_lang::error_code;

#[error_code]
pub enum ErrorType {
    #[msg("Fee Vault accumulated amount overflowed")]
    FeeVaultOverflowed,

    #[msg("Proposal count overflowed")]
    ProposalCountOverflowed,

    #[msg("BPS cannot exceed 10000")]
    BFSOverflowed,

    #[msg("Value Overflowed")]
    ValueOverflowed,

    #[msg("Voter Count Overflowed")]
    VoteCountOverflowed,

    #[msg("Only voter who created account can vote")]
    VoterUnauthorized,

    #[msg("Already Voted")]
    AlreadyVoted
}
