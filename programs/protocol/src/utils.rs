use crate::state::*;
use crate::errors::*;
use anchor_lang::prelude::*;


pub fn initialize_global_state(
    ctx: Context<InitializeGlobalState>,
    staking_token_mint: Pubkey,
    treasury_wallet: Pubkey,
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    global_state.admin = ctx.accounts.admin.key();
    global_state.staking_token_mint = staking_token_mint;
    global_state.treasury_wallet = treasury_wallet;
    global_state.total_presales = 0;
    global_state.active_presales = 0;
    global_state.total_stakers = 0;
    global_state.bump = ctx.bumps.global_state;
    msg!("Global state initialized successfully");
    
    Ok(())
}


pub fn update_admin(
    ctx: Context<UpdateAdmin>,
    new_admin: Pubkey,
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;
    
    // Ensure only current admin can update
    require!(
        ctx.accounts.admin.key() == global_state.admin,
        IdoError::Unauthorized
    );
    
    // Update admin
    global_state.admin = new_admin;
    
    msg!("Admin updated successfully");
    
    Ok(())
}


#[derive(Accounts)]
pub struct InitializeGlobalState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<GlobalState>(),
        seeds = [b"global_state"],
        bump
    )]

    pub global_state: Account<'info, GlobalState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAdmin<'info> {
    #[account(
        constraint = admin.key() == global_state.admin @ IdoError::Unauthorized
    )]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
}