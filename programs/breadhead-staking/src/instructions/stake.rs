use {
    crate::{errors::ErrorCode, state::*},
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount, Approve, approve},
    mpl_token_metadata::{
        instruction::freeze_delegated_account,
        ID as metadata_program_id,
        utils::is_master_edition
    },
    solana_program::program::invoke_signed
};

#[derive(Accounts)]
pub struct StakeCtx<'info> {
    #[account(mut, seeds = [STAKE_ENTRY_PREFIX.as_bytes(), stake_entry.pool.as_ref(), stake_entry.original_mint.as_ref(), get_stake_seed(original_mint.supply, user.key()).as_ref()], bump=stake_entry.bump)]
    stake_entry: Box<Account<'info, StakeEntry>>,

    #[account(mut, constraint = stake_entry.pool == stake_pool.key() @ ErrorCode::InvalidStakePool)]
    stake_pool: Box<Account<'info, StakePool>>,

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
    #[account(mut)]
    user: Signer<'info>,
    #[account(mut, constraint =
        user_original_mint_token_account.amount > 0
        && user_original_mint_token_account.mint == stake_entry.original_mint
        && user_original_mint_token_account.owner == user.key()
        @ ErrorCode::InvalidUserOriginalMintTokenAccount
    )]
    user_original_mint_token_account: Box<Account<'info, TokenAccount>>,

    // programs
    token_program: Program<'info, Token>,
    /// CHECK: constraint verifies this is the metadata program
    #[account(constraint =
        metadata_program.key() == metadata_program_id
        @ ErrorCode::InvalidMetadataProgram
    )]
    metadata_program: AccountInfo<'info>
}

pub fn handler(ctx: Context<StakeCtx>, amount: u64) -> Result<()> {

    if ctx.accounts.stake_pool.end_date.is_some() && Clock::get().unwrap().unix_timestamp > ctx.accounts.stake_pool.end_date.unwrap() {
        return Err(error!(ErrorCode::StakePoolHasEnded));
    }

    if ctx.accounts.stake_entry.amount != 0 {
        ctx.accounts.stake_entry.total_stake_seconds = ctx.accounts.stake_entry.total_stake_seconds.saturating_add(
            (u128::try_from(ctx.accounts.stake_entry.cooldown_start_seconds.unwrap_or(Clock::get().unwrap().unix_timestamp))
                .unwrap()
                .saturating_sub(u128::try_from(ctx.accounts.stake_entry.last_staked_at).unwrap()))
            .checked_mul(u128::try_from(ctx.accounts.stake_entry.amount).unwrap())
            .unwrap(),
        );
        ctx.accounts.stake_entry.cooldown_start_seconds = None;
    }

    // approve program authority over token account
    approve(ctx.accounts.approve_ctx(), 1)?;

    let stake_pool = &mut ctx.accounts.stake_pool;
    let stake_entry = &mut ctx.accounts.stake_entry;

    // freeze token account
    let freeze_ix = freeze_delegated_account(
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
        &freeze_ix,
        &[
            ctx.accounts.metadata_program.to_account_info(),
            ctx.accounts.program_authority.to_account_info(),
            ctx.accounts.user_original_mint_token_account.to_account_info(),
            ctx.accounts.master_edition.to_account_info(),
            ctx.accounts.original_mint.to_account_info()
        ],
        signer
    )?;


    if stake_pool.reset_on_stake && stake_entry.amount == 0 {
        stake_entry.total_stake_seconds = 0;
    }

    // update stake entry
    stake_entry.last_staked_at = Clock::get().unwrap().unix_timestamp;
    stake_entry.last_staker = ctx.accounts.user.key();
    stake_entry.amount = stake_entry.amount.checked_add(amount).unwrap();
    stake_pool.total_staked = stake_pool.total_staked.checked_add(1).expect("Add error");

    Ok(())
}


impl<'info> StakeCtx <'info> {
    pub fn approve_ctx(&self) -> CpiContext<'_,'_,'_, 'info, Approve<'info>> {
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Approve {
            to: self.user_original_mint_token_account.to_account_info(),
            delegate: self.program_authority.to_account_info(),
            authority: self.user.to_account_info()
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}