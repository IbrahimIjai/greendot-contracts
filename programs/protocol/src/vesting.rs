use crate::constants::*;
use crate::state::*;
use crate::errors::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};


pub fn calculate_claimable_amount(
    user_info: &Account<UserPresaleInfo>,
    presale: &Account<Presale>,
) -> Result<(u64, u64, u64)> {
    // let current_time = Clock::get()?.unix_timestamp;
    let total_purchased = user_info.purchased;
    
    // If vesting is not enabled, all tokens are claimable immediately
    if !presale.vesting_enabled {
        let first_release = total_purchased;
        return Ok((first_release, 0, 0));
    }
    
    // Calculate vesting amounts
    let first_release = total_purchased
        .checked_mul(VESTING_FIRST_RELEASE_PERCENTAGE as u64)
        .unwrap()
        .checked_div(100)
        .unwrap();
    
    let second_release = total_purchased
        .checked_mul(VESTING_SECOND_RELEASE_PERCENTAGE as u64)
        .unwrap()
        .checked_div(100)
        .unwrap();
    
    let third_release = total_purchased
        .checked_mul(VESTING_THIRD_RELEASE_PERCENTAGE as u64)
        .unwrap()
        .checked_div(100)
        .unwrap();
    
    // Return claimable amounts based on time
    Ok((first_release, second_release, third_release))
}

pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
    let presale = &ctx.accounts.presale;
    let user_info = &mut ctx.accounts.user_info;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Ensure presale is completed
    require!(
        presale.status == STATUS_COMPLETED,
        IdoError::PresaleNotCompleted
    );
    
    // Calculate claimable amounts
    let (first_release, second_release, third_release) = 
        calculate_claimable_amount(user_info, presale)?;
    
    let mut amount_to_claim:u64 = 0;
    
    // Check if first release is available
    if current_time >= presale.first_release_time && !user_info.first_claim_processed {
        amount_to_claim = amount_to_claim.checked_add(first_release).unwrap();
        user_info.first_claim_processed = true;
    }
    
    // Check if second release is available
    if current_time >= presale.second_release_time && !user_info.second_claim_processed {
        amount_to_claim = amount_to_claim.checked_add(second_release).unwrap();
        user_info.second_claim_processed = true;
    }
    
    // Check if third release is available
    if current_time >= presale.third_release_time && !user_info.third_claim_processed {
        amount_to_claim = amount_to_claim.checked_add(third_release).unwrap();
        user_info.third_claim_processed = true;
    }
    
    // Ensure there's something to claim
    require!(
        amount_to_claim > 0,
        IdoError::NothingToClaim
    );
    
    // Transfer tokens from presale token account to user
    let seeds = &[
        SEED_PREFIX_PRESALE,
        presale.token_mint.as_ref(),
        presale.creator.as_ref(),
        &[presale.bump],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = Transfer {
        from: ctx.accounts.presale_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.presale.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::transfer(cpi_ctx, amount_to_claim)?;
    
    // Update user info
    user_info.claimed = user_info.claimed.checked_add(amount_to_claim).unwrap();
    
    msg!("User claimed {} tokens successfully", amount_to_claim);
    
    Ok(())
}

pub fn get_upcoming_claims(
    user_info: &Account<UserPresaleInfo>,
    presale: &Account<Presale>,
) -> Result<Vec<(i64, u64)>> {
    let current_time = Clock::get()?.unix_timestamp;
    let mut upcoming_claims = Vec::new();
    
    // Calculate vesting amounts
    let (first_release, second_release, third_release) = 
        calculate_claimable_amount(user_info, presale)?;
    
    // Add upcoming claims to the vector
    if !user_info.first_claim_processed && presale.first_release_time > current_time {
        upcoming_claims.push((presale.first_release_time, first_release));
    }
    
    if !user_info.second_claim_processed && presale.second_release_time > current_time {
        upcoming_claims.push((presale.second_release_time, second_release));
    }
    
    if !user_info.third_claim_processed && presale.third_release_time > current_time {
        upcoming_claims.push((presale.third_release_time, third_release));
    }
    
    Ok(upcoming_claims)
}

#[derive(Accounts)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [
            SEED_PREFIX_PRESALE,
            presale.token_mint.as_ref(),
            presale.creator.as_ref(),
        ],
        bump = presale.bump,
        constraint = presale.status == STATUS_COMPLETED @ IdoError::PresaleNotCompleted
    )]
    pub presale: Account<'info, Presale>,
    
    #[account(
        mut,
        constraint = presale_token_account.mint == presale.token_mint,
        constraint = presale_token_account.owner == presale.key()
    )]
    pub presale_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key(),
        constraint = user_token_account.mint == presale.token_mint
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [
            SEED_PREFIX_USER_INFO,
            user.key().as_ref(),
            presale.key().as_ref(),
        ],
        bump = user_info.bump,
        constraint = user_info.user == user.key(),
        constraint = user_info.presale == presale.key()
    )]
    pub user_info: Account<'info, UserPresaleInfo>,
    
    pub token_program: Program<'info, Token>,
}