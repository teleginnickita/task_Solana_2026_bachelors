use anchor_lang::prelude::*;
use game_core::{
    constants::PLAYER_SEED,
};

declare_id!("3kx233sHmZfTrMHJ66sqBip2nAqntfL6y6V219BfmBdN");

#[program]
pub mod search {
    use super::*;

    /// Initializes the player cooldown PDA for search actions.
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + Player::INIT_SPACE,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,
    pub system_program: Program<'info, System>,
}

/// Stores player-specific cooldown state for the search mechanic.
#[account]
#[derive(InitSpace)]
pub struct Player {
    /// Wallet that owns this player state.
    pub owner: Pubkey,
    /// Unix timestamp of the last successful search action.
    pub last_search_timestamp: i64,
    /// PDA bump for this account.
    pub bump: u8,
}
