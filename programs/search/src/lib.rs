use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hashv;
use game_core::constants::{PLAYER_SEED, RESOURCE_COUNT};

declare_id!("3kx233sHmZfTrMHJ66sqBip2nAqntfL6y6V219BfmBdN");

const SEARCH_COOLDOWN_SECONDS: i64 = 60;
const SEARCH_REWARD_SLOTS: usize = 3;

#[program]
pub mod search {
    use super::*;

    /// Initializes the player cooldown PDA for search actions.
    pub fn initialize_player(ctx: Context<InitializePlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.last_found_resources = [0; SEARCH_REWARD_SLOTS];
        player.bump = ctx.bumps.player;

        Ok(())
    }

    /// Performs a resource search if the player's cooldown has elapsed.
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let player = &mut ctx.accounts.player;

        require!(
            cooldown_elapsed(player.last_search_timestamp, now),
            SearchError::SearchCooldownActive
        );

        player.last_found_resources = roll_resources(player.owner, now);
        player.last_search_timestamp = now;

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

#[derive(Accounts)]
pub struct SearchResources<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump = player.bump,
        has_one = owner
    )]
    pub player: Account<'info, Player>,
}

/// Stores player-specific cooldown state for the search mechanic.
#[account]
#[derive(InitSpace)]
pub struct Player {
    /// Wallet that owns this player state.
    pub owner: Pubkey,
    /// Unix timestamp of the last successful search action.
    pub last_search_timestamp: i64,
    /// Last three resources found during the latest successful search.
    pub last_found_resources: [u8; SEARCH_REWARD_SLOTS],
    /// PDA bump for this account.
    pub bump: u8,
}

#[error_code]
pub enum SearchError {
    #[msg("The player must wait 60 seconds before searching again.")]
    SearchCooldownActive,
}

fn cooldown_elapsed(last_search_timestamp: i64, now: i64) -> bool {
    last_search_timestamp == 0 || now.saturating_sub(last_search_timestamp) >= SEARCH_COOLDOWN_SECONDS
}

fn roll_resources(owner: Pubkey, timestamp: i64) -> [u8; SEARCH_REWARD_SLOTS] {
    let mut found = [0u8; SEARCH_REWARD_SLOTS];

    for (index, slot) in found.iter_mut().enumerate() {
        let digest = hashv(&[
            b"search-resource",
            owner.as_ref(),
            &timestamp.to_le_bytes(),
            &[index as u8],
        ]);

        *slot = digest.0[0] % RESOURCE_COUNT as u8;
    }

    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooldown_accepts_first_search() {
        assert!(cooldown_elapsed(0, 100));
    }

    #[test]
    fn cooldown_rejects_early_retry() {
        assert!(!cooldown_elapsed(100, 159));
    }

    #[test]
    fn cooldown_accepts_after_full_delay() {
        assert!(cooldown_elapsed(100, 160));
    }

    #[test]
    fn rolls_always_fit_resource_range() {
        let owner = Pubkey::new_unique();
        let resources = roll_resources(owner, 1_717_171_717);

        assert_eq!(resources.len(), SEARCH_REWARD_SLOTS);
        assert!(resources.iter().all(|value| *value < RESOURCE_COUNT as u8));
    }
}
