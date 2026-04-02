use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use game_core::constants::MARKETPLACE_AUTHORITY_SEED;
use item_nft::{self, cpi::accounts::BurnItemMetadata, program::ItemNft, ItemMetadata};
use magic_token::{self, cpi::accounts::MintMarketplaceReward, program::MagicToken};
use resource_manager::GameConfig;

declare_id!("xCeFkpeadNjjYyL3BStmgDceC9ZjeRW3DtErAJ1nQ2y");

#[program]
pub mod marketplace {
    use super::*;

    /// Mints a reward payout to a seller through the magic token program.
    pub fn mint_seller_reward(ctx: Context<MintSellerReward>, amount: u64) -> Result<()> {
        let signer_seeds: &[&[u8]] = &[MARKETPLACE_AUTHORITY_SEED, &[ctx.bumps.marketplace_authority]];
        let signer = [signer_seeds];
        let cpi_accounts = MintMarketplaceReward {
            marketplace_authority: ctx.accounts.marketplace_authority.to_account_info(),
            magic_token_config: ctx.accounts.magic_token_config.to_account_info(),
            mint_authority: ctx.accounts.magic_token_authority.to_account_info(),
            magic_token_mint: ctx.accounts.magic_token_mint.to_account_info(),
            recipient_token_account: ctx.accounts.recipient_token_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.magic_token_program.to_account_info(),
            cpi_accounts,
            &signer,
        );

        magic_token::cpi::mint_marketplace_reward(cpi_context, amount)
    }

    /// Sells an item into the protocol marketplace, burns its metadata ownership, and pays MagicToken.
    pub fn sell_item(ctx: Context<SellItem>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.item_metadata.owner,
            ctx.accounts.seller.key(),
            MarketplaceError::InvalidItemOwner
        );

        let item_type = ctx.accounts.item_metadata.item_type as usize;
        require!(item_type < ctx.accounts.game_config.item_prices.len(), MarketplaceError::InvalidItemType);
        let payout = ctx.accounts.game_config.item_prices[item_type];
        require!(payout > 0, MarketplaceError::InvalidPayout);

        let signer_seeds: &[&[u8]] = &[MARKETPLACE_AUTHORITY_SEED, &[ctx.bumps.marketplace_authority]];
        let signer = [signer_seeds];

        let mint_reward_accounts = MintMarketplaceReward {
            marketplace_authority: ctx.accounts.marketplace_authority.to_account_info(),
            magic_token_config: ctx.accounts.magic_token_config.to_account_info(),
            mint_authority: ctx.accounts.magic_token_authority.to_account_info(),
            magic_token_mint: ctx.accounts.magic_token_mint.to_account_info(),
            recipient_token_account: ctx.accounts.recipient_token_account.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let mint_reward_context = CpiContext::new_with_signer(
            ctx.accounts.magic_token_program.to_account_info(),
            mint_reward_accounts,
            &signer,
        );
        magic_token::cpi::mint_marketplace_reward(mint_reward_context, payout)?;

        let burn_item_accounts = BurnItemMetadata {
            owner: ctx.accounts.seller.to_account_info(),
            item_metadata: ctx.accounts.item_metadata.to_account_info(),
            mint: ctx.accounts.item_mint.to_account_info(),
        };
        let burn_item_context = CpiContext::new(
            ctx.accounts.item_nft_program.to_account_info(),
            burn_item_accounts,
        );
        item_nft::cpi::burn_item_metadata(burn_item_context)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintSellerReward<'info> {
    #[account(seeds = [MARKETPLACE_AUTHORITY_SEED], bump)]
    /// CHECK: PDA signer used only for CPI calls into the magic token program.
    pub marketplace_authority: UncheckedAccount<'info>,
    /// CHECK: Owned by the magic token program and validated there.
    pub magic_token_config: UncheckedAccount<'info>,
    /// CHECK: Magic token program PDA, validated in the callee.
    pub magic_token_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub magic_token_program: Program<'info, MagicToken>,
}

#[derive(Accounts)]
pub struct SellItem<'info> {
    pub seller: Signer<'info>,
    #[account(seeds = [MARKETPLACE_AUTHORITY_SEED], bump)]
    /// CHECK: PDA signer used only for CPI calls into the magic token program.
    pub marketplace_authority: UncheckedAccount<'info>,
    pub game_config: Account<'info, GameConfig>,
    /// CHECK: Owned by the magic token program and validated there.
    pub magic_token_config: UncheckedAccount<'info>,
    /// CHECK: Magic token program PDA, validated in the callee.
    pub magic_token_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub item_metadata: Account<'info, ItemMetadata>,
    /// CHECK: Tied to the item metadata PDA and validated in the item_nft program.
    pub item_mint: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub magic_token_program: Program<'info, MagicToken>,
    pub item_nft_program: Program<'info, ItemNft>,
}

#[error_code]
pub enum MarketplaceError {
    #[msg("The seller does not own the provided item.")]
    InvalidItemOwner,
    #[msg("The item type stored in metadata is not supported.")]
    InvalidItemType,
    #[msg("The configured payout must be greater than zero.")]
    InvalidPayout,
}
