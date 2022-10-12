pub mod errors;
pub mod instructions;
pub mod state;

use {anchor_lang::prelude::*, instructions::*};

declare_id!("FpEMdUwx8GAE4kc5BXgP5pKwAv7FstakVU6JLRnT5kmH");

#[program]
pub mod breadhead_staking {
    use super::*;

    pub fn init_identifier(ctx: Context<InitIdentifierCtx>) -> Result<()> {
        init_identifier::handler(ctx)
    }

    pub fn init_pool(ctx: Context<InitPoolCtx>, ix: InitPoolIx) -> Result<()> {
        init_pool::handler(ctx, ix)
    }

    pub fn init_entry(ctx: Context<InitEntryCtx>, user: Pubkey) -> Result<()> {
        init_entry::handler(ctx, user)
    }

    pub fn stake(ctx: Context<StakeCtx>, amount: u64) -> Result<()> {
        stake::handler(ctx, amount)
    }

    // pub fn unstake(ctx: Context<UnstakeCtx>) -> Result<()> {
    //     unstake::handler(ctx)
    // }

    // pub fn update_pool(ctx: Context<UpdatePoolCtx>, ix: UpdatePoolIx) -> Result<()> {
    //     update_pool::handler(ctx, ix)
    // }

    // pub fn close_stake_pool(ctx: Context<CloseStakePoolCtx>) -> Result<()> {
    //     close_stake_pool::handler(ctx)
    // }

    // pub fn close_stake_entry(ctx: Context<CloseStakeEntryCtx>) -> Result<()> {
    //     close_stake_entry::handler(ctx)
    // }

    // pub fn reasssign_stake_entry(ctx: Context<ReassignStakeEntryCtx>, ix: ReassignStakeEntryIx) -> Result<()> {
    //     reassign_stake_entry::handler(ctx, ix)
    // }

}