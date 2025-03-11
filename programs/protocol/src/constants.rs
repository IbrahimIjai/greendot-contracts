pub const TIER_1_REQUIREMENT: u64 = 1_000;
pub const TIER_2_REQUIREMENT: u64 = 10_000;
pub const TIER_3_REQUIREMENT: u64 = 100_000;

pub const TIER_1_ALLOCATION_PERCENTAGE: u8 = 34;
pub const TIER_2_ALLOCATION_PERCENTAGE: u8 = 33;
pub const TIER_3_ALLOCATION_PERCENTAGE: u8 = 33;

pub const PRESALE_CREATOR_WITHDRAWAL_PERCENTAGE: u8 = 30;
pub const LIQUIDITY_PERCENTAGE: u8 = 60;
pub const PROTOCOL_FEE_PERCENTAGE: u8 = 10;

pub const VESTING_FIRST_RELEASE_PERCENTAGE: u8 = 40;
pub const VESTING_SECOND_RELEASE_PERCENTAGE: u8 = 30;
pub const VESTING_THIRD_RELEASE_PERCENTAGE: u8 = 30;

pub const VESTING_FIRST_RELEASE_DURATION: i64 = 300; // 5 minutes in seconds
pub const VESTING_SECOND_RELEASE_DURATION: i64 = 600; // 10 minutes in seconds
pub const VESTING_THIRD_RELEASE_DURATION: i64 = 900; // 15 minutes in seconds

pub const SEED_PREFIX_PRESALE: &[u8] = b"presale";
pub const SEED_PREFIX_USER_STAKE: &[u8] = b"user_stake";
pub const SEED_PREFIX_USER_INFO: &[u8] = b"user_info";
pub const SEED_PREFIX_VESTING: &[u8] = b"vesting";

pub const STATUS_PENDING: u8 = 0;
pub const STATUS_APPROVED: u8 = 1;
pub const STATUS_LIVE: u8 = 2;
pub const STATUS_COMPLETED: u8 = 3;
pub const STATUS_CANCELLED: u8 = 4;