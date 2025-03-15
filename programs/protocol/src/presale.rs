use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::*;
use crate::state::*;
use crate::tier::*;

pub fn create_presale(
    ctx: Context<CreatePresale>,
    tokens_for_sale: u64,
    token_price: u64,
    start_time: i64,
    end_time: i64,
    registration_start_time: i64,
    registration_end_time: i64,
    listing_price: u64,
    vesting_enabled: bool,
) -> Result<()> {
    let presale = &mut ctx.accounts.presale;
    let current_time = Clock::get()?.unix_timestamp;

    // Validate time setup
    require!(
        registration_start_time < registration_end_time,
        IdoError::InvalidTimeSetup
    );

    require!(
        registration_end_time < start_time,
        IdoError::InvalidTimeSetup
    );

    require!(start_time < end_time, IdoError::InvalidTimeSetup);

    require!(
        registration_start_time > current_time,
        IdoError::InvalidTimeSetup
    );

    // Calculate tier allocations
    let (tier1_allocation, tier2_allocation, tier3_allocation) =
        calculate_presale_tier_allocations(tokens_for_sale);

    // Initialize presale

    presale.admin = ctx.accounts.global_state.admin;
    presale.creator = ctx.accounts.creator.key();
    presale.mint_of_token_being_sold = ctx.accounts.mint_of_token_being_sold.key();
    presale.status = STATUS_PENDING; // Starts as pending until admin approves
    presale.token_price = token_price;
    presale.tokens_for_sale = tokens_for_sale;
    presale.tokens_sold = 0;
    presale.start_time = start_time;
    presale.end_time = end_time;
    presale.registration_start_time = registration_start_time;
    presale.registration_end_time = registration_end_time;
    presale.tier1_allocation = tier1_allocation;
    presale.tier2_allocation = tier2_allocation;
    presale.tier3_allocation = tier3_allocation;
    presale.tier1_sold = 0;
    presale.tier2_sold = 0;
    presale.tier3_sold = 0;
    presale.sol_raised = 0;
    presale.listing_price = listing_price;
    presale.is_listed = false;
    presale.presale_token_account = ctx.accounts.presale_token_account.key();
    presale.presale_sol_account = ctx.accounts.global_state.treasury_wallet;
    presale.vesting_enabled = vesting_enabled;

    // Set vesting times if enabled
    if vesting_enabled {
        presale.first_release_time = end_time;
        presale.second_release_time = end_time + VESTING_FIRST_RELEASE_DURATION;
        presale.third_release_time = end_time + VESTING_SECOND_RELEASE_DURATION;
    } else {
        presale.first_release_time = end_time;
        presale.second_release_time = end_time;
        presale.third_release_time = end_time;
    }

    presale.bump = ctx.bumps.presale;

    // Transfer tokens from creator to presale token account
    let cpi_accounts = Transfer {
        from: ctx.accounts.creator_token_account.to_account_info(),
        to: ctx.accounts.presale_token_account.to_account_info(),
        authority: ctx.accounts.creator.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, tokens_for_sale)?;

    // Update global state
    let global_state = &mut ctx.accounts.global_state;
    global_state.total_presales += 1;

    msg!("Presale created successfully");

    Ok(())
}

pub fn approve_presale(ctx: Context<ApprovePresale>) -> Result<()> {
    let presale = &mut ctx.accounts.presale;

    // Ensure only admin can approve
    require!(
        ctx.accounts.admin.key() == presale.admin,
        IdoError::Unauthorized
    );

    // Ensure presale is in pending status
    require!(
        presale.status == STATUS_PENDING,
        IdoError::InvalidPresaleStatus
    );

    // Update status to approved
    presale.status = STATUS_APPROVED;

    // Update global state
    let global_state = &mut ctx.accounts.global_state;
    global_state.active_presales += 1;

    msg!("Presale approved successfully");

    Ok(())
}

pub fn register_for_presale(ctx: Context<RegisterForPresale>) -> Result<()> {
    let presale = &ctx.accounts.presale;
    let user_stake = &ctx.accounts.user_stake;
    let user_info = &mut ctx.accounts.user_info;

    // Check if user is eligible for any tier
    check_tier_eligibility(user_stake, presale)?;

    // Initialize user presale info
    user_info.user = ctx.accounts.user.key();
    user_info.presale = presale.key();
    user_info.allocation = 0; // Will be calculated during purchase
    user_info.purchased = 0;
    user_info.claimed = 0;
    user_info.first_claim_processed = false;
    user_info.second_claim_processed = false;
    user_info.third_claim_processed = false;
    user_info.bump = ctx.bumps.user_info;

    msg!("User registered for presale successfully");

    Ok(())
}

pub fn buy_tokens(ctx: Context<BuyTokens>, mut amount: u64) -> Result<()> {
    // Create a copy of the key before mutable borrow

    let user_key = ctx.accounts.user.key();

    let presale = &mut ctx.accounts.presale;
    let user_stake = &ctx.accounts.user_stake;
    let user_info = &mut ctx.accounts.user_info;
    let current_time = Clock::get()?.unix_timestamp;

    // Ensure presale is live
    require!(
        presale.status == STATUS_LIVE,
        IdoError::InvalidPresaleStatus
    );

    // Ensure presale is within time bounds
    require!(
        current_time >= presale.start_time,
        IdoError::PresaleNotStarted
    );

    require!(current_time <= presale.end_time, IdoError::PresaleEnded);

    // Calculate SOL amount needed
    let sol_amount = amount.checked_mul(presale.token_price).unwrap();

    // Check if user can purchase based on tier
    let user_tier = user_stake.tier;
    let mut available_allocation = 0;

    // Try to buy from tier 1 first if user is tier 1 or higher
    if can_purchase_from_tier(1, user_tier, presale) {
        available_allocation += get_available_allocation_for_tier(1, presale)?;
    }

    // Then try tier 2 if user is tier 2 or higher
    if can_purchase_from_tier(2, user_tier, presale) {
        available_allocation += get_available_allocation_for_tier(2, presale)?;
    }

    // Then try tier 3 if user is tier 3
    if can_purchase_from_tier(3, user_tier, presale) {
        available_allocation += get_available_allocation_for_tier(3, presale)?;
    }

    // Ensure user has enough allocation
    require!(
        available_allocation >= amount,
        IdoError::InsufficientAllocation
    );

    // Store the account info before using it
    let presale_key = presale.key();
    let presale_info = presale.to_account_info();

    // Transfer SOL from user to presale account
    invoke(
        &system_instruction::transfer(&user_key, &presale_key, sol_amount),
        &[
            ctx.accounts.user.to_account_info(),
            presale_info,
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Update presale info
    presale.tokens_sold = presale.tokens_sold.checked_add(amount).unwrap();
    presale.sol_raised = presale.sol_raised.checked_add(sol_amount).unwrap();

    // Update tier allocations
    if can_purchase_from_tier(1, user_tier, presale) {
        let tier1_purchase = std::cmp::min(
            amount,
            presale.tier1_allocation.saturating_sub(presale.tier1_sold),
        );

        if tier1_purchase > 0 {
            presale.tier1_sold = presale.tier1_sold.checked_add(tier1_purchase).unwrap();
            amount = amount.saturating_sub(tier1_purchase);
        }
    }

    if amount > 0 && can_purchase_from_tier(2, user_tier, presale) {
        let tier2_purchase = std::cmp::min(
            amount,
            presale.tier2_allocation.saturating_sub(presale.tier2_sold),
        );

        if tier2_purchase > 0 {
            presale.tier2_sold = presale.tier2_sold.checked_add(tier2_purchase).unwrap();
            amount = amount.saturating_sub(tier2_purchase);
        }
    }

    if amount > 0 && can_purchase_from_tier(3, user_tier, presale) {
        let tier3_purchase = std::cmp::min(
            amount,
            presale.tier3_allocation.saturating_sub(presale.tier3_sold),
        );

        if tier3_purchase > 0 {
            presale.tier3_sold = presale.tier3_sold.checked_add(tier3_purchase).unwrap();
        }
    }

    // Update user info
    user_info.allocation = user_info.allocation.checked_add(amount).unwrap();
    user_info.purchased = user_info.purchased.checked_add(amount).unwrap();

    msg!("User purchased {} tokens successfully", amount);

    Ok(())
}

pub fn list_token(ctx: Context<ListToken>) -> Result<()> {
    let presale = &mut ctx.accounts.presale;

    // Store account info before mutably borrowing presale
    let presale_info = presale.to_account_info();

    // Ensure presale is not already listed
    require!(!presale.is_listed, IdoError::TokenAlreadyListed);

    // Calculate the amount of SOL to add to liquidity (80% of SOL raised)
    let sol_to_liquidity = presale
        .sol_raised
        .checked_mul(80)
        .unwrap()
        .checked_div(100)
        .unwrap();

    // Calculate the amount of tokens to add to liquidity (20% of tokens sold)
    let tokens_to_liquidity = presale
        .tokens_sold
        .checked_mul(20)
        .unwrap()
        .checked_div(100)
        .unwrap();

    // Transfer SOL to the liquidity pool
    let sol_transfer_ix = system_instruction::transfer(
        &presale.key(),
        &ctx.accounts.liquidity_pool.key(),
        sol_to_liquidity,
    );

    invoke(
        &sol_transfer_ix,
        &[
            presale_info.clone(),
            ctx.accounts.liquidity_pool.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Transfer tokens to the liquidity pool

    let seeds = &[
        SEED_PREFIX_PRESALE,
        presale.mint_of_token_being_sold.as_ref(),
        presale.creator.as_ref(),
        &[presale.bump],
    ];
    let signer = &[&seeds[..]];

    // Transfer tokens using CPI
    let cpi_accounts = Transfer {
        from: ctx.accounts.presale_token_account.to_account_info(),
        to: ctx.accounts.liquidity_token_account.to_account_info(),
        authority: presale_info.clone(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    token::transfer(
        CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
        tokens_to_liquidity,
    )?;

    // Update presale status
    presale.is_listed = true;

    msg!("Token listed successfully");

    Ok(())
}

#[derive(Accounts)]
pub struct CreatePresale<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = 8 + std::mem::size_of::<Presale>(),
        seeds = [
            SEED_PREFIX_PRESALE,
            mint_of_token_being_sold.key().as_ref(),
            creator.key().as_ref(),
        ],
        bump
    )]
    pub presale: Account<'info, Presale>,

    pub creator_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = creator,
        token::mint = mint_of_token_being_sold,
        token::authority = presale,
        seeds = [
            SEED_PREFIX_PRESALE,
            mint_of_token_being_sold.key().as_ref(),
            creator.key().as_ref(),
            b"token_account"
        ],
        bump
    )]
    pub presale_token_account: Account<'info, TokenAccount>,

    pub mint_of_token_being_sold: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ApprovePresale<'info> {
    #[account(
        constraint = admin.key() == global_state.admin @ IdoError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX_PRESALE,
            presale.mint_of_token_being_sold.as_ref(),
            presale.creator.as_ref(),
        ],
        bump = presale.bump
    )]
    pub presale: Account<'info, Presale>,

    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
}

#[derive(Accounts)]
pub struct ManagePresale<'info> {
    #[account(
        constraint = admin.key() == global_state.admin @ IdoError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX_PRESALE,
            presale.mint_of_token_being_sold.as_ref(),
            presale.creator.as_ref(),
        ],
        bump = presale.bump
    )]
    pub presale: Account<'info, Presale>,

    #[account(
        mut,
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,
}

#[derive(Accounts)]
pub struct RegisterForPresale<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX_PRESALE,
            presale.mint_of_token_being_sold.as_ref(),
            presale.creator.as_ref(),
        ],
        bump = presale.bump,
        constraint = presale.status == STATUS_APPROVED || presale.status == STATUS_LIVE @ IdoError::InvalidPresaleStatus
    )]
    pub presale: Account<'info, Presale>,

    #[account(
        seeds = [
            SEED_PREFIX_USER_STAKE,
            user.key().as_ref(),
            global_state.staking_token_mint.as_ref()
        ],
        bump = user_stake.bump,
        constraint = user_stake.user == user.key()
    )]
    pub user_stake: Account<'info, UserStake>,

    #[account(
        init,
        payer = user,
        space = 8 + std::mem::size_of::<UserPresaleInfo>(),
        seeds = [
            SEED_PREFIX_USER_INFO,
            user.key().as_ref(),
            presale.key().as_ref(),
        ],
        bump
    )]
    pub user_info: Account<'info, UserPresaleInfo>,

    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct BuyTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX_PRESALE,
            presale.mint_of_token_being_sold.as_ref(),
            presale.creator.as_ref(),
        ],
        bump = presale.bump,
        constraint = presale.status == STATUS_LIVE @ IdoError::InvalidPresaleStatus
    )]
    pub presale: Account<'info, Presale>,

    #[account(
        seeds = [
            SEED_PREFIX_USER_STAKE,
            user.key().as_ref(),
            global_state.staking_token_mint.as_ref()
        ],
        bump = user_stake.bump,
        constraint = user_stake.user == user.key()
    )]
    pub user_stake: Account<'info, UserStake>,

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

    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListToken<'info> {
    #[account(
        constraint = admin.key() == global_state.admin @ IdoError::Unauthorized
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SEED_PREFIX_PRESALE,
            presale.mint_of_token_being_sold.as_ref(),
            presale.creator.as_ref(),
        ],
        bump = presale.bump,
        constraint = presale.status == STATUS_COMPLETED @ IdoError::PresaleNotCompleted
    )]
    pub presale: Account<'info, Presale>,

    #[account(
        mut,
        constraint = presale_token_account.mint == presale.mint_of_token_being_sold,
        constraint = presale_token_account.owner == presale.key()
    )]
    pub presale_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    /// CHECK: This is the liquidity pool account
    pub liquidity_pool: AccountInfo<'info>,

    #[account(mut)]
    pub liquidity_token_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"global_state"],
        bump = global_state.bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
