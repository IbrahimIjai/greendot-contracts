use anchor_lang::prelude::*;

#[account]
pub struct Presale {
    pub admin: Pubkey,                      // Admin who can approve and manage presales
    pub creator: Pubkey,                    // Creator of the presale
    pub mint_of_token_being_sold: Pubkey,                 // Token being sold
    pub status: u8,                         // 0: Pending, 1: Approved, 2: Live, 3: Completed, 4: Cancelled
    pub token_price: u64,                   // Price in lamports per token
    pub tokens_for_sale: u64,               // Total number of tokens for sale
    pub tokens_sold: u64,                   // Number of tokens sold so far
    pub start_time: i64,                    // Start time of the presale (unix timestamp)
    pub end_time: i64,                      // End time of the presale (unix timestamp)
    pub registration_start_time: i64,       // Start time for registration
    pub registration_end_time: i64,         // End time for registration
    pub tier1_allocation: u64,              // Allocation for tier 1 users
    pub tier2_allocation: u64,              // Allocation for tier 2 users
    pub tier3_allocation: u64,              // Allocation for tier 3 users
    pub tier1_sold: u64,                    // Amount sold to tier 1 users
    pub tier2_sold: u64,                    // Amount sold to tier 2 users
    pub tier3_sold: u64,                    // Amount sold to tier 3 users
    pub sol_raised: u64,                    // Total SOL raised
    pub listing_price: u64,                 // Listing price in lamports per token
    pub is_listed: bool,                    // Whether the token has been listed
    pub presale_token_account: Pubkey,      // Token account holding presale tokens
    pub presale_sol_account: Pubkey,        // SOL account receiving payments
    pub vesting_enabled: bool,              // Whether vesting is enabled
    pub first_release_time: i64,            // Time of first release
    pub second_release_time: i64,           // Time of second release
    pub third_release_time: i64,            // Time of third release
    pub bump: u8,                           // PDA bump
}


#[account]
pub struct UserStake {
    pub user: Pubkey,                       // User wallet
    pub staking_token_mint: Pubkey,         // Staking token mint (SFUND or XToken)
    pub amount: u64,                        // Amount staked
    pub lock_time: i64,                     // Time when tokens were locked
    pub tier: u8,                           // User's tier based on locked amount
    pub bump: u8,                           // PDA bump
}

#[account]
pub struct UserPresaleInfo {
    pub user: Pubkey,                       // User wallet
    pub presale: Pubkey,                    // Presale account
    pub allocation: u64,                    // User's total allocation
    pub purchased: u64,                     // Amount purchased
    pub claimed: u64,                       // Amount claimed
    pub first_claim_processed: bool,        // Whether first claim has been processed
    pub second_claim_processed: bool,       // Whether second claim has been processed
    pub third_claim_processed: bool,        // Whether third claim has been processed
    pub bump: u8,                           // PDA bump
}

#[account]
pub struct GlobalState {
    pub admin: Pubkey,                      // Program admin
    pub staking_token_mint: Pubkey,         // SFUND or XToken mint
    pub treasury_wallet: Pubkey,            // Treasury wallet for protocol fees
    pub total_presales: u64,                // Total number of presales created
    pub active_presales: u64,               // Number of active presales
    pub total_stakers: u64,                 // Total number of stakers
    pub bump: u8,                           // PDA bump
}