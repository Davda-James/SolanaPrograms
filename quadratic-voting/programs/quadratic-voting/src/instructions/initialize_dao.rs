use crate::state::Dao;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct InitializeDao<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Dao::DISCRIMINATOR.len() + Dao::INIT_SPACE,
        seeds = [b"dao".as_ref(), name.as_bytes() ,admin.key().as_ref()],
        bump
    )]
    pub dao_account: Account<'info, Dao>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeDao<'info> {
    pub fn init_dao(&mut self, name: String, bumps: &InitializeDaoBumps) -> Result<()> {
        self.dao_account.set_inner(Dao {
            name,
            authority: self.admin.key(),
            proposal_count: 0,
            bump: bumps.dao_account,
        });
        Ok(())
    }
}
