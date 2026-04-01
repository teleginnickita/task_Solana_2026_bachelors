use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use game_core::constants::{PLAYER_SEED, RESOURCE_COUNT, RESOURCE_AUTHORITY_SEED, SEARCH_AUTHORITY_SEED};
use resource_manager::{self, cpi::accounts::MintSearchReward, program::ResourceManager};

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

    /// Performs a search and mints the three discovered resources to the owner.
    pub fn search_and_mint_resources(ctx: Context<SearchAndMintResources>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let player = &mut ctx.accounts.player;

        require!(
            cooldown_elapsed(player.last_search_timestamp, now),
            SearchError::SearchCooldownActive
        );

        let found_resources = roll_resources(player.owner, player.last_search_timestamp);
        let mint_accounts = [
            &ctx.accounts.resource_mint_0,
            &ctx.accounts.resource_mint_1,
            &ctx.accounts.resource_mint_2,
        ];
        let token_accounts = [
            &ctx.accounts.recipient_token_account_0,
            &ctx.accounts.recipient_token_account_1,
            &ctx.accounts.recipient_token_account_2,
        ];

        for index in 0..SEARCH_REWARD_SLOTS {
            require_keys_eq!(
                token_accounts[index].owner,
                ctx.accounts.owner.key(),
                SearchError::InvalidRecipientOwner
            );
            require_keys_eq!(
                token_accounts[index].mint,
                mint_accounts[index].key(),
                SearchError::RecipientMintMismatch
            );

            let cpi_accounts = MintSearchReward {
                search_authority: ctx.accounts.search_authority.to_account_info(),
                game_config: ctx.accounts.game_config.to_account_info(),
                mint_authority: ctx.accounts.resource_manager_authority.to_account_info(),
                resource_mint: mint_accounts[index].to_account_info(),
                recipient_token_account: token_accounts[index].to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let signer_seeds: &[&[u8]] = &[SEARCH_AUTHORITY_SEED, &[ctx.bumps.search_authority]];
            let signer = [signer_seeds];
            let cpi_context = CpiContext::new_with_signer(
                ctx.accounts.resource_manager_program.to_account_info(),
                cpi_accounts,
                &signer,
            );

            resource_manager::cpi::mint_search_reward(
                cpi_context,
                found_resources[index],
            )?;
        }

        player.last_found_resources = found_resources;
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

#[derive(Accounts)]
pub struct SearchAndMintResources<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [PLAYER_SEED, owner.key().as_ref()],
        bump = player.bump,
        has_one = owner
    )]
    pub player: Account<'info, Player>,
    #[account(seeds = [SEARCH_AUTHORITY_SEED], bump)]
    /// CHECK: PDA signer used only for CPI calls into the resource manager.
    pub search_authority: UncheckedAccount<'info>,
    /// CHECK: Owned by the resource manager program and forwarded to CPI.
    pub game_config: UncheckedAccount<'info>,
    #[account(seeds = [RESOURCE_AUTHORITY_SEED], bump, seeds::program = resource_manager_program.key())]
    /// CHECK: Resource-manager PDA signer, validated by derivation.
    pub resource_manager_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub resource_mint_0: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub resource_mint_1: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub resource_mint_2: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub recipient_token_account_0: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub recipient_token_account_1: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub recipient_token_account_2: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub resource_manager_program: Program<'info, ResourceManager>,
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
    #[msg("Recipient token account must belong to the searching player.")]
    InvalidRecipientOwner,
    #[msg("Recipient token account mint does not match the provided resource mint.")]
    RecipientMintMismatch,
}

fn cooldown_elapsed(last_search_timestamp: i64, now: i64) -> bool {
    last_search_timestamp == 0 || now.saturating_sub(last_search_timestamp) >= SEARCH_COOLDOWN_SECONDS
}

fn roll_resources(owner: Pubkey, timestamp: i64) -> [u8; SEARCH_REWARD_SLOTS] {
    let mut found = [0u8; SEARCH_REWARD_SLOTS];
    let owner_bytes = owner.to_bytes();
    let timestamp_bias = timestamp.to_le_bytes()[0];
    let base = owner_bytes[0].wrapping_add(timestamp_bias);

    for (index, slot) in found.iter_mut().enumerate() {
        *slot = base.wrapping_add((index as u8).wrapping_mul(2)) % RESOURCE_COUNT as u8;
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
