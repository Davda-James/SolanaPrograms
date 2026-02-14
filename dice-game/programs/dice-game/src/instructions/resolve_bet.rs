use anchor_instruction_sysvar::ed25519::Ed25519InstructionSignatures;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use solana_program::hash::hash;
use std::str::FromStr;

use crate::{error::DiceGameError, state::Bet};

pub const HOUSE_EDGE: u128 = 150;

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub house: Signer<'info>,
    #[account(mut)]
    /// CHECK: This is good will handle it.
    pub player: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"vault".as_ref(), house.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        mut,
        has_one = player,
        close = player,
        seeds = [b"bet".as_ref(), vault.key().as_ref(), bet.seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub bet: Account<'info, Bet>,
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: This is good will handle by address
    pub instruction_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> ResolveBet<'info> {
    pub fn verify_ed25519_signature(&self, sig: &[u8]) -> Result<()> {
        let ix = load_instruction_at_checked(0, &self.instruction_sysvar.to_account_info())
            .map_err(|_| DiceGameError::InvalidEd25519Signature)?;

        let ed25519_program_pubkey =
            Pubkey::from_str("Ed25519SigVerify111111111111111111111111111")
                .map_err(|_| DiceGameError::ED25519ProgramError)?;
        require!(
            ix.program_id == ed25519_program_pubkey,
            DiceGameError::ED25519ProgramError
        );
        require!(ix.accounts.len() == 0, DiceGameError::ED25519AccountsError);
        let sigs = Ed25519InstructionSignatures::unpack(&ix.data)
            .map_err(|_| DiceGameError::ED25519SignatureMustBeOne)?;

        let sig_vec = &sigs.0;

        require!(sig_vec.len() == 1, DiceGameError::InvalidEd25519Signature);
        let _sig = &sig_vec[0];
        require!(_sig.is_verifiable, DiceGameError::InvalidEd25519Signature);
        require_keys_eq!(
            _sig.public_key
                .ok_or(DiceGameError::InvalidEd25519PublicKey)?,
            self.house.key(),
            DiceGameError::InvalidEd25519PublicKey
        );
        require!(
            _sig.signature
                .ok_or(DiceGameError::InvalidEd25519Signature)?
                .eq(sig),
            DiceGameError::InvalidEd25519Signature,
        );
        require!(
            _sig.message
                .as_ref()
                .ok_or(DiceGameError::InvalidEd25519Signature)?
                .eq(&self.bet.to_slice()),
            DiceGameError::MessageMismatch
        );
        Ok(())
    }

    pub fn resolve(&mut self, sig: &[u8], bumps: &ResolveBetBumps) -> Result<()> {
        self.verify_ed25519_signature(sig)?;

        let digest = hash(sig).to_bytes();
        let mut half = [0u8; 16];
        half.copy_from_slice(&digest[0..16]);
        let upper = u128::from_le_bytes(half);
        half.copy_from_slice(&digest[16..32]);
        let lower = u128::from_le_bytes(half);
        let roll = ((upper.wrapping_add(lower)) % 100) as u8 + 1;

        if roll <= self.bet.roll {
            let payout_u128 = u128::from(self.bet.amount)
                .checked_mul(10000u128 - HOUSE_EDGE)
                .ok_or(DiceGameError::Overflow)?
                .checked_div(self.bet.roll as u128)
                .ok_or(DiceGameError::Overflow)?
                .checked_div(100u128)
                .ok_or(DiceGameError::Overflow)?;

            if payout_u128 > u128::from(u64::MAX) {
                return Err(DiceGameError::Overflow.into());
            }
            let payout = payout_u128 as u64;

            let house_key = self.house.key();
            let signer_seeds: &[&[&[u8]]] =
                &[&[b"vault".as_ref(), house_key.as_ref(), &[bumps.vault]]];

            let cpi_accounts = anchor_lang::system_program::Transfer {
                from: self.vault.to_account_info(),
                to: self.player.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            anchor_lang::system_program::transfer(cpi_ctx, payout)?;
        }

        Ok(())
    }
}
