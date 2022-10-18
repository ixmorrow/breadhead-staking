use {crate::state::*, anchor_lang::prelude::*, anchor_spl::token::Mint};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitPoolIx {
    requires_collections: Vec<Pubkey>,
    requires_authorization: bool,
    authority: Pubkey,
    reset_on_stake: bool,
    cooldown_seconds: Option<u32>,
    min_stake_seconds: Option<u32>,
    end_date: Option<i64>,
}

#[derive(Accounts)]
#[instruction(ix: InitPoolIx)]
pub struct InitPoolCtx<'info> {
    #[account(
        init,
        payer = payer,
        space = STAKE_POOL_SIZE,
        seeds = [STAKE_POOL_PREFIX.as_bytes(), original_mint.key().as_ref()],
        bump
    )]
    stake_pool: Account<'info, StakePool>,
    pub original_mint: Account<'info, Mint>,
    // #[account(mut)]
    // identifier: Account<'info, Identifier>,

    #[account(mut)]
    payer: Signer<'info>,
    system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitPoolCtx>, ix: InitPoolIx) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    // let identifier = &mut ctx.accounts.identifier;
    stake_pool.bump = *ctx.bumps.get("stake_pool").unwrap();
    // stake_pool.identifier = identifier.count;
    // stake_pool.requires_collections = ix.requires_collections;
    stake_pool.requires_authorization = ix.requires_authorization;
    stake_pool.authority = ix.authority;
    stake_pool.reset_on_stake = ix.reset_on_stake;
    stake_pool.cooldown_seconds = ix.cooldown_seconds;
    stake_pool.min_stake_seconds = ix.min_stake_seconds;
    stake_pool.end_date = ix.end_date;
    stake_pool.total_staked = 0;

    // let identifier = &mut ctx.accounts.identifier;
    // identifier.count += 1;
    Ok(())
}
