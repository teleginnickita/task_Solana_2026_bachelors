use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Burn, Mint, MintTo, TokenAccount, TokenInterface};
use game_core::{
    constants::{
        GAME_CONFIG_SEED, ITEM_COUNT, RESOURCE_AUTHORITY_SEED, RESOURCE_COUNT, RESOURCE_MINT_SEED,
        SEARCH_AUTHORITY_SEED,
    },
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
        search_program: Pubkey,
        crafting_program: Pubkey,
    ) -> Result<()> {
        let game_config = &mut ctx.accounts.game_config;
        game_config.admin = ctx.accounts.admin.key();
        game_config.resource_mints = resource_mints;
        game_config.magic_token_mint = magic_token_mint;
        game_config.item_prices = item_prices;
        game_config.search_program = search_program;
        game_config.crafting_program = crafting_program;
        game_config.bump = ctx.bumps.game_config;

        Ok(())
    }

    /// Initializes a Token-2022 mint for one of the six base resources.
    pub fn initialize_resource_mint(
        ctx: Context<InitializeResourceMint>,
        resource_index: u8,
    ) -> Result<()> {
        require!(
            resource_index < RESOURCE_COUNT as u8,
            ResourceManagerError::InvalidResourceIndex
        );

        let expected_mint = ctx.accounts.game_config.resource_mints[resource_index as usize];
        require_keys_eq!(
            expected_mint,
            ctx.accounts.resource_mint.key(),
            ResourceManagerError::UnexpectedResourceMint
        );

        Ok(())
    }

    /// Mints resource tokens into a player's token account under program control.
    pub fn mint_resource(
        ctx: Context<MintResource>,
        resource_index: u8,
        amount: u64,
    ) -> Result<()> {
        require!(
            resource_index < RESOURCE_COUNT as u8,
            ResourceManagerError::InvalidResourceIndex
        );
        require!(amount > 0, ResourceManagerError::InvalidMintAmount);

        let expected_mint = ctx.accounts.game_config.resource_mints[resource_index as usize];
        require_keys_eq!(
            expected_mint,
            ctx.accounts.resource_mint.key(),
            ResourceManagerError::UnexpectedResourceMint
        );
        require_keys_eq!(
            ctx.accounts.recipient_token_account.mint,
            ctx.accounts.resource_mint.key(),
            ResourceManagerError::RecipientMintMismatch
        );

        let signer_seeds: &[&[u8]] = &[
            RESOURCE_AUTHORITY_SEED,
            &[ctx.bumps.mint_authority],
        ];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.resource_mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let signer = [signer_seeds];
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            &signer,
        );

        token_interface::mint_to(cpi_context, amount)
    }

    /// Burns resource tokens from the owner's token account.
    pub fn burn_resource(
        ctx: Context<BurnResource>,
        resource_index: u8,
        amount: u64,
    ) -> Result<()> {
        require!(
            resource_index < RESOURCE_COUNT as u8,
            ResourceManagerError::InvalidResourceIndex
        );
        require!(amount > 0, ResourceManagerError::InvalidBurnAmount);

        let expected_mint = ctx.accounts.game_config.resource_mints[resource_index as usize];
        require_keys_eq!(
            expected_mint,
            ctx.accounts.resource_mint.key(),
            ResourceManagerError::UnexpectedResourceMint
        );
        require_keys_eq!(
            ctx.accounts.owner_token_account.mint,
            ctx.accounts.resource_mint.key(),
            ResourceManagerError::RecipientMintMismatch
        );
        require_keys_eq!(
            ctx.accounts.owner_token_account.owner,
            ctx.accounts.owner.key(),
            ResourceManagerError::TokenAccountOwnerMismatch
        );

        let cpi_accounts = Burn {
            mint: ctx.accounts.resource_mint.to_account_info(),
            from: ctx.accounts.owner_token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );

        token_interface::burn(cpi_context, amount)
    }

    /// Mints one search reward resource when called by the authorized search program.
    pub fn mint_search_reward(
        ctx: Context<MintSearchReward>,
        resource_index: u8,
    ) -> Result<()> {
        require!(
            resource_index < RESOURCE_COUNT as u8,
            ResourceManagerError::InvalidResourceIndex
        );

        let expected_mint = ctx.accounts.game_config.resource_mints[resource_index as usize];
        require_keys_eq!(
            expected_mint,
            ctx.accounts.resource_mint.key(),
            ResourceManagerError::UnexpectedResourceMint
        );
        require_keys_eq!(
            ctx.accounts.recipient_token_account.mint,
            ctx.accounts.resource_mint.key(),
            ResourceManagerError::RecipientMintMismatch
        );

        let signer_seeds: &[&[u8]] = &[RESOURCE_AUTHORITY_SEED, &[ctx.bumps.mint_authority]];
        let signer = [signer_seeds];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.resource_mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            &signer,
        );

        token_interface::mint_to(cpi_context, 1)
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

#[derive(Accounts)]
#[instruction(resource_index: u8)]
pub struct InitializeResourceMint<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        has_one = admin
    )]
    pub game_config: Account<'info, GameConfig>,
    /// CHECK: PDA signer used as the mint authority for all resource mints.
    #[account(seeds = [RESOURCE_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [RESOURCE_MINT_SEED, &[resource_index]],
        bump,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
        mint::token_program = token_program
    )]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintResource<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [GAME_CONFIG_SEED],
        bump = game_config.bump,
        has_one = admin
    )]
    pub game_config: Account<'info, GameConfig>,
    /// CHECK: PDA signer used as the mint authority for all resource mints.
    #[account(seeds = [RESOURCE_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct BurnResource<'info> {
    pub owner: Signer<'info>,
    #[account(seeds = [GAME_CONFIG_SEED], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub owner_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct MintSearchReward<'info> {
    #[account(
        seeds = [SEARCH_AUTHORITY_SEED],
        bump,
        seeds::program = game_config.search_program
    )]
    /// PDA signer belonging to the configured search program.
    pub search_authority: Signer<'info>,
    #[account(seeds = [GAME_CONFIG_SEED], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,
    /// CHECK: PDA signer used as the mint authority for all resource mints.
    #[account(seeds = [RESOURCE_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub resource_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
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
    /// Program id authorized to mint search rewards via CPI.
    pub search_program: Pubkey,
    /// Program id authorized to burn craft inputs via CPI.
    pub crafting_program: Pubkey,
    /// PDA bump for this account.
    pub bump: u8,
}

#[error_code]
pub enum ResourceManagerError {
    #[msg("Resource index is out of range.")]
    InvalidResourceIndex,
    #[msg("Resource mint does not match the game configuration.")]
    UnexpectedResourceMint,
    #[msg("Mint amount must be greater than zero.")]
    InvalidMintAmount,
    #[msg("Burn amount must be greater than zero.")]
    InvalidBurnAmount,
    #[msg("Recipient token account is not associated with the provided mint.")]
    RecipientMintMismatch,
    #[msg("Token account owner does not match the provided owner signer.")]
    TokenAccountOwnerMismatch,
}
