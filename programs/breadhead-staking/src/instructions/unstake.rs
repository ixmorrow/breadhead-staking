use {
    crate::{errors::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_lang::AccountsClose,
    anchor_spl::token::{Mint, Token, TokenAccount, Revoke, revoke},
    mpl_token_metadata::{
        instruction::thaw_delegated_account,
        ID as metadata_program_id,
        utils::is_master_edition
    },
    solana_program::program::invoke_signed
};

#[derive(Accounts)]
pub struct UnstakeCtx<'info> {
    #[account(mut)]
    stake_pool: Box<Account<'info, StakePool>>,
    #[account(mut, constraint = stake_entry.pool == stake_pool.key() @ ErrorCode::InvalidStakePool)]
    stake_entry: Box<Account<'info, StakeEntry>>,

    /// CHECK: Safe this is used a program signer
    #[account(
        mut,
        seeds = [PROGRAM_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    program_authority: AccountInfo<'info>,
    original_mint: Box<Account<'info, Mint>>,

    /// CHECK: constraint verifies this is a master edition
    #[account(constraint = 
        is_master_edition(
            &master_edition, original_mint.decimals, original_mint.supply) == true
            @ ErrorCode::InvalidMasterEdition
        )]
    master_edition: AccountInfo<'info>,

    // user
    #[account(mut, constraint = user.key() == stake_entry.last_staker @ ErrorCode::InvalidUnstakeUser)]
    user: Signer<'info>,
    #[account(mut, constraint =
        user_original_mint_token_account.mint == stake_entry.original_mint
        && user_original_mint_token_account.owner == user.key()
        @ ErrorCode::InvalidUserOriginalMintTokenAccount)]
    user_original_mint_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [user.key().as_ref(), original_mint.key().as_ref(), STAKE_STATE_SEED.as_bytes()],
        bump = stake_state.bump
    )]
    stake_state: Account<'info, StakeState>,

    // programs
    token_program: Program<'info, Token>,
    /// CHECK: constraint verifies this is the metadata program
    #[account(constraint =
        metadata_program.key() == metadata_program_id
        @ ErrorCode::InvalidMetadataProgram
    )]
    metadata_program: AccountInfo<'info>
}

pub fn handler(ctx: Context<UnstakeCtx>) -> Result<()> {

    if ctx.accounts.stake_pool.min_stake_seconds.is_some()
        && ctx.accounts.stake_pool.min_stake_seconds.unwrap() > 0
        && ((Clock::get().unwrap().unix_timestamp - ctx.accounts.stake_entry.last_staked_at) as u32) < ctx.accounts.stake_pool.min_stake_seconds.unwrap()
    {
        return Err(error!(ErrorCode::MinStakeSecondsNotSatisfied));
    }

    if ctx.accounts.stake_pool.cooldown_seconds.is_some() && ctx.accounts.stake_pool.cooldown_seconds.unwrap() > 0 {
        if ctx.accounts.stake_entry.cooldown_start_seconds.is_none() {
            ctx.accounts.stake_entry.cooldown_start_seconds = Some(Clock::get().unwrap().unix_timestamp);
            return Ok(());
        } else if ctx.accounts.stake_entry.cooldown_start_seconds.is_some() && ((Clock::get().unwrap().unix_timestamp - ctx.accounts.stake_entry.cooldown_start_seconds.unwrap()) as u32) < ctx.accounts.stake_pool.cooldown_seconds.unwrap() {
            return Err(error!(ErrorCode::CooldownSecondRemaining));
        }
    }

    // thaw token account
    let thaw_ix = thaw_delegated_account(
        ctx.accounts.metadata_program.key(),
        ctx.accounts.program_authority.key(),
        ctx.accounts.user_original_mint_token_account.key(),
        ctx.accounts.master_edition.key(),
        ctx.accounts.original_mint.key()
    );

    let auth_bump = *ctx.bumps.get("program_authority").unwrap();
    let auth_seeds = &[PROGRAM_AUTHORITY_SEED.as_bytes(), &[auth_bump]];
    let signer = &[&auth_seeds[..]];
    invoke_signed(
        &thaw_ix,
        &[
            ctx.accounts.metadata_program.to_account_info(),
            ctx.accounts.program_authority.to_account_info(),
            ctx.accounts.user_original_mint_token_account.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.original_mint.to_account_info()
        ],
        signer
    )?;

    // revoke program authority as delegate
    revoke(ctx.accounts.revoke_ctx())?;

    let stake_pool = &mut ctx.accounts.stake_pool;
    let stake_entry = &mut ctx.accounts.stake_entry;

    stake_entry.total_stake_seconds = stake_entry.total_stake_seconds.saturating_add(
        (u128::try_from(stake_entry.cooldown_start_seconds.unwrap_or(Clock::get().unwrap().unix_timestamp))
            .unwrap()
            .saturating_sub(u128::try_from(stake_entry.last_staked_at).unwrap()))
        .checked_mul(u128::try_from(stake_entry.amount).unwrap())
        .unwrap(),
    );
    stake_entry.last_staker = Pubkey::default();
    stake_entry.original_mint_claimed = false;
    stake_entry.stake_mint_claimed = false;
    stake_entry.amount = 0;
    stake_entry.cooldown_start_seconds = None;
    stake_pool.total_staked = stake_pool.total_staked.checked_sub(1).expect("Sub error");
    stake_entry.kind = StakeEntryKind::Permissionless as u8;

    // close user stake state account
    ctx.accounts.stake_state.close(ctx.accounts.user.to_account_info())?;

    Ok(())
}

impl<'info> UnstakeCtx <'info> {
    pub fn revoke_ctx(&self) -> CpiContext<'_,'_,'_, 'info, Revoke<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Revoke {
            source: self.user_original_mint_token_account.to_account_info(),
            authority: self.user.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}