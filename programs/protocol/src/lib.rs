use anchor_lang::prelude::*;

// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("4SF1zHVgcXfhbp1KCxuRPNQzurfML6XvQuVERnEnAWU8");

pub mod constants;
pub mod errors;
pub mod presale;
pub mod staking;
pub mod state;
pub mod tier;
pub mod utils;
pub mod vesting;

use presale::*;
use staking::*;
use utils::*;
use vesting::*;

#[program]
mod sfund_sonic {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializeGlobalState>,
        staking_token_mint: Pubkey,
        treasury_wallet: Pubkey,
    ) -> Result<()> {
        initialize_global_state(ctx, staking_token_mint, treasury_wallet)
    }

    pub fn update_admin(ctx: Context<UpdateAdmin>, new_admin: Pubkey) -> Result<()> {
        utils::update_admin(ctx, new_admin)
    }

    // Presale functions
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
        presale::create_presale(
            ctx,
            tokens_for_sale,
            token_price,
            start_time,
            end_time,
            registration_start_time,
            registration_end_time,
            listing_price,
            vesting_enabled,
        )
    }

    pub fn approve_presale(ctx: Context<ApprovePresale>) -> Result<()> {
        presale::approve_presale(ctx)
    }

    pub fn register_for_presale(ctx: Context<RegisterForPresale>) -> Result<()> {
        presale::register_for_presale(ctx)
    }

    pub fn buy_tokens(ctx: Context<BuyTokens>, amount: u64) -> Result<()> {
        presale::buy_tokens(ctx, amount)
    }

    pub fn list_token(ctx: Context<ListToken>) -> Result<()> {
        presale::list_token(ctx)
    }

    // Staking functions

    pub fn stake(ctx: Context<StakeTokens>, amount: u64) -> Result<()> {
        staking::stake_tokens(ctx, amount)
    }

    pub fn unstake(ctx: Context<UnstakeTokens>, amount: u64) -> Result<()> {
        staking::unstake_tokens(ctx, amount)
    }

    // Vesting/claiming functions

    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        vesting::claim_tokens(ctx)
    }

}
