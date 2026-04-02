use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use game_core::types::ItemType;
use item_nft::{self, cpi::accounts::RegisterItemMetadata, program::ItemNft};
use resource_manager::{self, cpi::accounts::BurnResource, program::ResourceManager};

declare_id!("GqTny3DGUaCXESufnpUQXG1p8QFodc1aCrYG1qvPkqXd");

#[program]
pub mod crafting {
    use super::*;

    /// Burns the recipe resources and registers the crafted item metadata.
    pub fn craft_item(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
        let item_type = ItemType::from_u8(item_type).ok_or(CraftingError::InvalidItemType)?;
        let recipe = item_type.recipe();

        burn_recipe_component(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.owner.key(),
            ctx.accounts.game_config.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.resource_manager_program.to_account_info(),
            0,
            recipe[0],
            &ctx.accounts.resource_mint_0,
            &ctx.accounts.owner_token_account_0,
        )?;
        burn_recipe_component(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.owner.key(),
            ctx.accounts.game_config.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.resource_manager_program.to_account_info(),
            1,
            recipe[1],
            &ctx.accounts.resource_mint_1,
            &ctx.accounts.owner_token_account_1,
        )?;
        burn_recipe_component(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.owner.key(),
            ctx.accounts.game_config.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.resource_manager_program.to_account_info(),
            2,
            recipe[2],
            &ctx.accounts.resource_mint_2,
            &ctx.accounts.owner_token_account_2,
        )?;
        burn_recipe_component(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.owner.key(),
            ctx.accounts.game_config.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.resource_manager_program.to_account_info(),
            3,
            recipe[3],
            &ctx.accounts.resource_mint_3,
            &ctx.accounts.owner_token_account_3,
        )?;
        burn_recipe_component(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.owner.key(),
            ctx.accounts.game_config.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.resource_manager_program.to_account_info(),
            4,
            recipe[4],
            &ctx.accounts.resource_mint_4,
            &ctx.accounts.owner_token_account_4,
        )?;
        burn_recipe_component(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.owner.key(),
            ctx.accounts.game_config.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.resource_manager_program.to_account_info(),
            5,
            recipe[5],
            &ctx.accounts.resource_mint_5,
            &ctx.accounts.owner_token_account_5,
        )?;

        let item_nft_cpi_accounts = RegisterItemMetadata {
            owner: ctx.accounts.owner.to_account_info(),
            mint: ctx.accounts.item_mint.to_account_info(),
            item_metadata: ctx.accounts.item_metadata.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let item_nft_cpi_context = CpiContext::new(
            ctx.accounts.item_nft_program.to_account_info(),
            item_nft_cpi_accounts,
        );

        item_nft::cpi::register_item_metadata(item_nft_cpi_context, item_type as u8)
    }
}

#[derive(Accounts)]
pub struct CraftItem<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: Owned by the resource manager program and validated there.
    pub game_config: UncheckedAccount<'info>,
    #[account(mut)]
    pub resource_mint_0: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub resource_mint_1: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub resource_mint_2: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub resource_mint_3: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub resource_mint_4: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub resource_mint_5: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub owner_token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub owner_token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub owner_token_account_2: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub owner_token_account_3: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub owner_token_account_4: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub owner_token_account_5: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Placeholder mint pubkey for the crafted item. Real NFT minting comes next.
    pub item_mint: UncheckedAccount<'info>,
    /// CHECK: PDA is derived and initialized by the item_nft CPI call.
    #[account(mut)]
    pub item_metadata: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub resource_manager_program: Program<'info, ResourceManager>,
    pub item_nft_program: Program<'info, ItemNft>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum CraftingError {
    #[msg("Unsupported item type.")]
    InvalidItemType,
    #[msg("Provided token account does not belong to the crafting player.")]
    TokenAccountOwnerMismatch,
    #[msg("Provided token account mint does not match the resource mint.")]
    ResourceMintMismatch,
}

fn burn_recipe_component<'info>(
    owner: AccountInfo<'info>,
    owner_key: Pubkey,
    game_config: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    resource_manager_program: AccountInfo<'info>,
    resource_index: u8,
    amount: u64,
    resource_mint: &InterfaceAccount<'info, Mint>,
    owner_token_account: &InterfaceAccount<'info, TokenAccount>,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }

    require_keys_eq!(
        owner_token_account.owner,
        owner_key,
        CraftingError::TokenAccountOwnerMismatch
    );
    require_keys_eq!(
        owner_token_account.mint,
        resource_mint.key(),
        CraftingError::ResourceMintMismatch
    );

    let cpi_accounts = BurnResource {
        owner,
        game_config,
        resource_mint: resource_mint.to_account_info(),
        owner_token_account: owner_token_account.to_account_info(),
        token_program,
    };
    let cpi_context = CpiContext::new(
        resource_manager_program,
        cpi_accounts,
    );

    resource_manager::cpi::burn_resource(cpi_context, resource_index, amount)
}
