use crate::constants::*;
use crate::state::*;
use crate::errors::*;
use crate::tier::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};


pub fn stake_tokens(
    ctx: Context<StakeTokens>,
    amount: u64,
) -> Result<()> {
    // let global_state = &ctx.accounts.global_state;
    let user_stake = &mut ctx.accounts.user_stake;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Transfer tokens from user to stake account
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info().clone(),
        to: ctx.accounts.stake_token_account.to_account_info().clone(),
        authority: ctx.accounts.user.to_account_info().clone(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::transfer(cpi_ctx, amount)?;
    
    // Update user stake info
    user_stake.user = ctx.accounts.user.key();
    user_stake.staking_token_mint = ctx.accounts.staking_token_mint.key();
    user_stake.amount = amount;
    user_stake.lock_time = current_time;
    user_stake.tier = get_tier_for_amount(amount);
    user_stake.bump = ctx.bumps.user_stake;
    
    msg!("User has staked {} tokens and qualified for tier {}", amount, user_stake.tier);
    
    Ok(())
}

pub fn unstake_tokens(
    ctx: Context<UnstakeTokens>,
    amount: u64,
) -> Result<()> {
    // Store references to data we need before mutable borrow
    let user_pubkey = ctx.accounts.user_stake.user;
    let token_mint = ctx.accounts.user_stake.staking_token_mint;
    let bump = ctx.accounts.user_stake.bump;
    let staked_amount = ctx.accounts.user_stake.amount;
    
    // Ensure user has enough staked
    require!(
        staked_amount >= amount,
        IdoError::InsufficientAllocation
    );
    
    // Transfer tokens from stake account back to user
    let seeds = &[
        SEED_PREFIX_USER_STAKE,
        user_pubkey.as_ref(),
        token_mint.as_ref(),
        &[bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.stake_token_account.to_account_info().clone(),
        to: ctx.accounts.user_token_account.to_account_info().clone(),
        authority: ctx.accounts.user_stake.to_account_info().clone(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, amount)?;
    
    // Now we can safely get a mutable reference
    let user_stake = &mut ctx.accounts.user_stake;
    
    // Update user stake info
    user_stake.amount = user_stake.amount.saturating_sub(amount);
    user_stake.tier = get_tier_for_amount(user_stake.amount);
    
    msg!("User has unstaked {} tokens and is now in tier {}", amount, user_stake.tier);
    
    Ok(())
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<UserStake>(),
        seeds = [
            SEED_PREFIX_USER_STAKE,
            user.key().as_ref(),
            staking_token_mint.key().as_ref()
        ],
        bump
    )]
    pub user_stake: Account<'info, UserStake>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == staking_token_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = user,
        token::mint = staking_token_mint,
        token::authority = user_stake,
        seeds = [
            SEED_PREFIX_USER_STAKE,
            user.key().as_ref(),
            staking_token_mint.key().as_ref(),
            b"token_account"
        ],
        bump
    )]
    pub stake_token_account: Account<'info, TokenAccount>,
    
    #[account(
        constraint = staking_token_mint.key() == global_state.staking_token_mint
    )]
    pub staking_token_mint: Account<'info, Mint>,
    
    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnstakeTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            SEED_PREFIX_USER_STAKE,
            user.key().as_ref(),
            staking_token_mint.key().as_ref()
        ],
        bump = user_stake.bump,
        constraint = user_stake.user == user.key()
    )]
    pub user_stake: Account<'info, UserStake>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == staking_token_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            SEED_PREFIX_USER_STAKE,
            user.key().as_ref(),
            staking_token_mint.key().as_ref(),
            b"token_account"
        ],
        bump
    )]
    pub stake_token_account: Account<'info, TokenAccount>,
    
    pub staking_token_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
}