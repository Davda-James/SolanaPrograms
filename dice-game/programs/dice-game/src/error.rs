use anchor_lang::prelude::*;

#[error_code]
pub enum DiceGameError {
    #[msg("Amount must be greater than zero.")]
    InvalidBetAmount,
    #[msg("Player of bet did not match")]
    InvalidPlayer,
    #[msg("Time not reached yet")]
    TimeoutNotReached,
    #[msg("Invalid Ed25519 instruction")]
    InvalidEd25519Instruction,
    #[msg("Invalid Ed25519 public key")]
    Ed25519Program,
    #[msg("Invalid Ed25519 signature")]
    InvalidEd25519Signature,
    #[msg("Signer mismatch")]
    SignerMismatch,
    #[msg("Signature mismatch")]
    SignatureMismatch,
    #[msg("Message Mismatch")]
    MessageMismatch,
    #[msg("Overflow")]
    Overflow,
    #[msg("ED25519 program error")]
    ED25519ProgramError,
    #[msg("ED25519 accounts length error")]
    ED25519AccountsError,
    #[msg("Signature must be one")]
    ED25519SignatureMustBeOne,
    #[msg("Invalid Ed25519 public key")]
    InvalidEd25519PublicKey,
}
