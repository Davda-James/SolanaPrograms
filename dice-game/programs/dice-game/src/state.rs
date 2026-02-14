use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Bet {
    pub player: Pubkey,
    pub seed: u128,
    pub slot: u64,
    pub roll: u8,
    pub amount: u64,
}

impl Bet {
    pub fn to_slice(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(self.player.as_ref());
        data.extend_from_slice(&self.seed.to_le_bytes());
        data.extend_from_slice(&self.slot.to_le_bytes());
        data.push(self.roll);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data
    }
}
