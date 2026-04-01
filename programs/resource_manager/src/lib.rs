use anchor_lang::prelude::*;
use game_core::{
    constants::{GAME_CONFIG_SEED, ITEM_COUNT, RESOURCE_COUNT},
};

declare_id!("CcvkCG2poiGbhLzkodhefhJbMbBW9AxHdKPR3eHrxfvf");

#[program]
pub mod resource_manager {
    use super::*;

    /// Initializes the singleton game configuration PDA.
    pub fn initialize_game_config(
        ctx: Context<InitializeGameConfig>,
        resource_mints: [Pubkey; RESOURCE_COUNT],
        magic_token_mint: Pubkey,
        item_prices: [u64; ITEM_COUNT],
    ) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = ctx.accounts.admin.key();
        game_config.resource_mints = resource_mints;
        game_config.magic_token_mint = magic_token_mint;
        game_config.item_prices = item_prices;
        game_config.bump = ctx.bumps.game_config;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGameConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + GameConfig::INIT_SPACE,
        seeds = [GAME_CONFIG_SEED],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,
    pub system_program: Program<'info, System>,
}

/// Stores global game configuration and mint addresses.
#[account]
#[derive(InitSpace)]
pub struct GameConfig {
    /// Admin authority allowed to manage global game settings.
    pub admin: Pubkey,
    /// Mint addresses for the six base resources.
    pub resource_mints: [Pubkey; RESOURCE_COUNT],
    /// Mint address for the reward token used by marketplace payouts.
    pub magic_token_mint: Pubkey,
    /// Payout prices for each item type.
    pub item_prices: [u64; ITEM_COUNT],
    /// PDA bump for this account.
    pub bump: u8,
}
