use anchor_lang::prelude::*;

pub const STAKE_ENTRY_PREFIX: &str = "stake-entry";
pub const STAKE_ENTRY_SIZE: usize = 8 + std::mem::size_of::<StakeEntry>() + 8;

pub const STAKE_POOL_PREFIX: &str = "stake-pool";

pub const IDENTIFIER_PREFIX: &str = "identifier";
pub const IDENTIFIER_SIZE: usize = 8 + std::mem::size_of::<Identifier>() + 8;

pub const STAKE_AUTHORIZATION_PREFIX: &str = "stake-authorization";
pub const STAKE_AUTHORIZATION_SIZE: usize = 8 + std::mem::size_of::<StakeAuthorizationRecord>() + 8;

pub const PROGRAM_AUTHORITY_SEED: &str = "authority";

pub const STAKE_STATE_SEED: &str = "state";

// pub const REST_LEVELS: [i64; 5] = [1728000, 3456000, 5184000, 8640000, 12960000];

// test
pub const REST_LEVELS: [i64; 5] = [1, 2, 3, 4, 5];


#[derive(Clone, Debug, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum StakeEntryKind {
    Permissionless = 0, // original
    Permissioned = 1,   // someone else called update_total_stake_seconds indicating claim_reward must check signer so this is a permissioned claim_rewards
}

#[account]
pub struct StakeEntry {
    pub bump: u8,
    pub pool: Pubkey,
    pub amount: u64,
    pub original_mint: Pubkey,
    pub original_mint_claimed: bool,
    pub last_staker: Pubkey,
    pub last_staked_at: i64,
    pub total_stake_seconds: u128,
    pub stake_mint_claimed: bool,
    pub kind: u8,
    pub stake_mint: Option<Pubkey>,
    pub cooldown_start_seconds: Option<i64>,
}

pub const STAKE_POOL_SIZE: usize = 8 + std::mem::size_of::<StakePool>() + 8;
#[account]
pub struct StakePool {
    pub bump: u8,
    pub authority: Pubkey,
    // pub requires_collections: Vec<Pubkey>,
    pub requires_authorization: bool,
    pub reset_on_stake: bool,
    pub total_staked: u64,
    pub cooldown_seconds: Option<u32>,
    pub min_stake_seconds: Option<u32>,
    pub end_date: Option<i64>,
}

pub const STAKE_STATE_SIZE: usize = 8 + std::mem::size_of::<StakeState>() + 8;
#[account]
pub struct StakeState {
    pub bump: u8,
    pub stake_start: i64,
    pub resting_level: u8,
    pub token_account: Pubkey,
    pub original_mint: Pubkey,
    pub pool: Pubkey,
    pub achievment_level: Achievement,
}

#[account]
pub struct StakeAuthorizationRecord {
    pub bump: u8,
    pub pool: Pubkey,
    pub mint: Pubkey,
}

#[account]
pub struct Identifier {
    pub bump: u8,
    pub count: u64,
}

pub fn get_stake_seed(supply: u64, user: Pubkey) -> Pubkey {
    if supply > 1 {
        user
    } else {
        Pubkey::default()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum Achievement {
    DoughBoy,
    SixtyNineBadge,
    BagHolder,
    MoonShot,
    Loaf,
    BreadGetter
}

pub fn derive_resting_level(stake_start_time: i64) -> u8 {
    // subtract start time from current time
    let stake_duration = Clock::get().unwrap().unix_timestamp - stake_start_time;
    let mut resting_level: u8 = 0;

    msg!("Stake start: {}", stake_start_time);
    msg!("Current: {}", Clock::get().unwrap().unix_timestamp);
    msg!("Stake duration: {}", stake_duration);

    // determine where time length falls in the RESTING_LEVELS array
    for i in 0..REST_LEVELS.len() {
        if stake_duration < REST_LEVELS[i] {
            resting_level = i as u8;
            break;
        }
        else if stake_duration > REST_LEVELS[i] && i == 4 {
            resting_level = i as u8 + 1;
        }
    }
    msg!("Rest level: {}", resting_level);
    return resting_level
}