use anchor_lang::prelude::*;
use crate::state::*;
use crate::events::*;

#[derive(Accounts)]
pub struct PlatformFee <'info> { 
    #[account(
        mut,
        has_one = admin
    )]
    state: Account<'info, State>,

    #[account(mut)]
    admin: Signer<'info>,
}


#[derive(Accounts)]
pub struct PlatformProposalBFS <'info> { 
    #[account(
        mut,
        has_one = admin
    )]
    state: Account<'info, State>,

    #[account(mut)]
    admin: Signer<'info>,
}
pub fn change_platform_fee_helper(ctx:Context<PlatformFee>, platform_fee: u64) -> Result<()> {
    let state = &mut ctx.accounts.state;
    state.platform_fee = platform_fee;

    emit!(PlatformFeeChanged {
        admin: ctx.accounts.admin.key(),
        platform_fee: state.platform_fee 
    });

    Ok(())
}

pub fn change_platform_proposal_bfs_helper(ctx: Context<PlatformProposalBFS>, platform_proposal_bfs: u16) -> Result<()> {
    let state = &mut ctx.accounts.state;
    state.platform_proposal_bps = platform_proposal_bfs;

    emit!(PlatformProposalBFSChanged {
        admin: ctx.accounts.admin.key(),
        platform_proposal_bfs: state.platform_proposal_bps 
    });

    Ok(())
}