use crate::constants::*;
use crate::state::*;
use crate::errors::*;
use anchor_lang::prelude::*;


pub fn get_tier_for_amount(amount: u64) -> u8 {
    if amount >= TIER_3_REQUIREMENT {
        3
    } else if amount >= TIER_2_REQUIREMENT {
        2
    } else if amount >= TIER_1_REQUIREMENT {
        1
    } else {
        0
    }
}

pub fn calculate_presale_tier_allocations(total_tokens_for_sale: u64) -> (u64, u64, u64) {
    let tier1_allocation = total_tokens_for_sale * TIER_1_ALLOCATION_PERCENTAGE as u64 / 100;
    let tier2_allocation = total_tokens_for_sale * TIER_2_ALLOCATION_PERCENTAGE as u64 / 100;
    let tier3_allocation = total_tokens_for_sale * TIER_3_ALLOCATION_PERCENTAGE as u64 / 100;
    
    (tier1_allocation, tier2_allocation, tier3_allocation)
}

pub fn check_tier_eligibility(
    user_stake: &Account<UserStake>,
    presale: &Account<Presale>,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    
    // Ensure registration is open
    require!(
        current_time >= presale.registration_start_time,
        IdoError::RegistrationNotStarted
    );
    
    require!(
        current_time <= presale.registration_end_time,
        IdoError::RegistrationEnded
    );
    
    // Check if user has staked enough for at least tier 1
    require!(
        user_stake.tier > 0,
        IdoError::InsufficientTierQualification
    );
    
    Ok(())
}

pub fn can_purchase_from_tier(
    tier: u8,
    user_tier: u8,
    presale: &Account<Presale>,
) -> bool {
    // Users can purchase from their tier or lower tiers
    if user_tier < tier {
        return false;
    }
    
    match tier {
        1 => presale.tier1_sold < presale.tier1_allocation,
        2 => presale.tier2_sold < presale.tier2_allocation,
        3 => presale.tier3_sold < presale.tier3_allocation,
        _ => false,
    }
}

pub fn get_available_allocation_for_tier(
    tier: u8,
    presale: &Account<Presale>,
) -> Result<u64> {
    match tier {
        1 => Ok(presale.tier1_allocation.saturating_sub(presale.tier1_sold)),
        2 => Ok(presale.tier2_allocation.saturating_sub(presale.tier2_sold)),
        3 => Ok(presale.tier3_allocation.saturating_sub(presale.tier3_sold)),
        _ => err!(IdoError::InsufficientTierQualification),
    }
}