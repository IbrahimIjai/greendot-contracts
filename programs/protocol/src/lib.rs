use anchor_lang::prelude::*;

declare_id!("DfFSFuoKbV9PKmqd1PDh6cLQuciqrVcChWjeZvrZMZ1y");

pub mod state;
pub mod utils;
pub mod errors;
pub mod presale;
pub mod tier;
pub mod constants;
pub mod staking;
pub mod vesting;

// use state::*;
use utils::*;
// use errors::*;
use presale::*;
// use tier::*;
// use constants::*;
use staking::*;
use vesting::*;
#[program]

pub mod protocol {
    use super::*;

  // Global state management
    pub fn initialize(
        ctx: Context<InitializeGlobalState>,
        staking_token_mint: Pubkey,
        treasury_wallet: Pubkey,
    ) -> Result<()> {
        initialize_global_state(ctx, staking_token_mint, treasury_wallet)
    }

    pub fn update_admin(
        ctx: Context<UpdateAdmin>,
        new_admin: Pubkey,
    ) -> Result<()> {
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
    
    pub fn start_presale(ctx: Context<ManagePresale>) -> Result<()> {
        presale::start_presale(ctx)
    }
    
    pub fn end_presale(ctx: Context<ManagePresale>) -> Result<()> {
        presale::end_presale(ctx)
    }
    
    pub fn register_for_presale(ctx: Context<RegisterForPresale>) -> Result<()> {
        presale::register_for_presale(ctx)
    }
    
    pub fn buy_tokens(
        ctx: Context<BuyTokens>,
        amount: u64,
    ) -> Result<()> {
        presale::buy_tokens(ctx, amount)
    }
    
    pub fn withdraw_funds(ctx: Context<WithdrawFunds>) -> Result<()> {
        presale::withdraw_funds(ctx)
    }
    
    pub fn list_token(
        ctx: Context<ListToken>,
        // pool_seed: u64,
    ) -> Result<()> {
        presale::list_token(ctx, )
    }


    // Staking functions

    pub fn stake(
        ctx: Context<StakeTokens>,
        amount: u64,
    ) -> Result<()> {
        staking::stake_tokens(ctx, amount)
    }
    
    pub fn unstake(
        ctx: Context<UnstakeTokens>,
        amount: u64,
    ) -> Result<()> {
        staking::unstake_tokens(ctx, amount)
    }
    

     // Vesting/claiming functions
    pub fn claim_tokens(ctx: Context<ClaimTokens>) -> Result<()> {
        vesting::claim_tokens(ctx)
    }

}

#[derive(Accounts)]
pub struct Initialize {}
