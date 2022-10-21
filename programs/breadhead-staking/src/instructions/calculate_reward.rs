use {
    crate::{errors::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, TokenAccount},
};

pub fn handler(ctx: Context<RewardCtx>) -> Result<()> {
    let user_state = &mut ctx.accounts.stake_state;

    let user_resting_level = derive_resting_level(user_state.stake_start);
    user_state.resting_level = user_resting_level;

    msg!("user state rest level: {}", user_state.resting_level);

    match user_state.resting_level {
        0 => user_state.achievment_level = Achievement::DoughBoy,
        1 => user_state.achievment_level = Achievement::SixtyNineBadge,
        2 => user_state.achievment_level = Achievement::BagHolder,
        3 => user_state.achievment_level = Achievement::MoonShot,
        4 => user_state.achievment_level = Achievement::Loaf,
        5 => user_state.achievment_level = Achievement::BreadGetter,
        _ => return Err(error!(ErrorCode::InvalidRestingLevel))
    }

    msg!("user achievement level: {:?}", user_state.achievment_level);

    Ok(())
}

#[derive(Accounts)]
pub struct RewardCtx<'info> {
    #[account(mut, seeds = [STAKE_ENTRY_PREFIX.as_bytes(), stake_entry.pool.as_ref(), stake_entry.original_mint.as_ref(), get_stake_seed(original_mint.supply, user.key()).as_ref()], bump=stake_entry.bump)]
    pub stake_entry: Box<Account<'info, StakeEntry>>,

    // user
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        constraint = user_original_mint_token_account.amount > 0
        && user_original_mint_token_account.mint == stake_entry.original_mint
        && user_original_mint_token_account.owner == user.key()
        @ ErrorCode::InvalidUserOriginalMintTokenAccount
    )]
    pub user_original_mint_token_account: Box<Account<'info, TokenAccount>>,
    pub original_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), original_mint.key().as_ref(), STAKE_STATE_SEED.as_bytes()],
        bump = stake_state.bump,
        constraint = stake_state.token_account == user_original_mint_token_account.key()
        @ ErrorCode::InvalidStakeEntryOriginalMintTokenAccount,
    )]
    pub stake_state: Account<'info, StakeState>,
}