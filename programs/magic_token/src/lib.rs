use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface};
use game_core::constants::{
    MAGIC_TOKEN_AUTHORITY_SEED, MAGIC_TOKEN_CONFIG_SEED, MAGIC_TOKEN_MINT_SEED,
    MARKETPLACE_AUTHORITY_SEED,
};

declare_id!("2ug8zVrkg3zkpCqEuR4AR2KBEkhSLr49pbcScTLrDKTL");

#[program]
pub mod magic_token {
    use super::*;

    /// Initializes the global magic token config.
    pub fn initialize_magic_token_config(
        ctx: Context<InitializeMagicTokenConfig>,
        marketplace_program: Pubkey,
        mint: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.magic_token_config;
        config.admin = ctx.accounts.admin.key();
        config.marketplace_program = marketplace_program;
        config.mint = mint;
        config.bump = ctx.bumps.magic_token_config;

        Ok(())
    }

    /// Initializes the Token-2022 mint used for marketplace rewards.
    pub fn initialize_magic_token_mint(ctx: Context<InitializeMagicTokenMint>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.magic_token_config.mint,
            ctx.accounts.magic_token_mint.key(),
            MagicTokenError::UnexpectedMint
        );

        Ok(())
    }

    /// Mints reward tokens when called by the authorized marketplace program.
    pub fn mint_marketplace_reward(
        ctx: Context<MintMarketplaceReward>,
        amount: u64,
    ) -> Result<()> {
        require!(amount > 0, MagicTokenError::InvalidMintAmount);
        require_keys_eq!(
            ctx.accounts.magic_token_config.mint,
            ctx.accounts.magic_token_mint.key(),
            MagicTokenError::UnexpectedMint
        );
        require_keys_eq!(
            ctx.accounts.recipient_token_account.mint,
            ctx.accounts.magic_token_mint.key(),
            MagicTokenError::RecipientMintMismatch
        );

        let signer_seeds: &[&[u8]] = &[MAGIC_TOKEN_AUTHORITY_SEED, &[ctx.bumps.mint_authority]];
        let signer = [signer_seeds];
        let cpi_accounts = MintTo {
            mint: ctx.accounts.magic_token_mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            &signer,
        );

        token_interface::mint_to(cpi_context, amount)
    }
}

#[derive(Accounts)]
pub struct InitializeMagicTokenConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + MagicTokenConfig::INIT_SPACE,
        seeds = [MAGIC_TOKEN_CONFIG_SEED],
        bump
    )]
    pub magic_token_config: Account<'info, MagicTokenConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeMagicTokenMint<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [MAGIC_TOKEN_CONFIG_SEED],
        bump = magic_token_config.bump,
        has_one = admin
    )]
    pub magic_token_config: Account<'info, MagicTokenConfig>,
    /// CHECK: PDA signer used as the mint authority for the reward token.
    #[account(seeds = [MAGIC_TOKEN_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [MAGIC_TOKEN_MINT_SEED],
        bump,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
        mint::token_program = token_program
    )]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintMarketplaceReward<'info> {
    #[account(
        seeds = [MARKETPLACE_AUTHORITY_SEED],
        bump,
        seeds::program = magic_token_config.marketplace_program
    )]
    pub marketplace_authority: Signer<'info>,
    #[account(seeds = [MAGIC_TOKEN_CONFIG_SEED], bump = magic_token_config.bump)]
    pub magic_token_config: Account<'info, MagicTokenConfig>,
    /// CHECK: PDA signer used as the mint authority for the reward token.
    #[account(seeds = [MAGIC_TOKEN_AUTHORITY_SEED], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub magic_token_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}

/// Stores configuration for the reward token program.
#[account]
#[derive(InitSpace)]
pub struct MagicTokenConfig {
    /// Admin that initializes and manages the config.
    pub admin: Pubkey,
    /// Marketplace program allowed to mint rewards via CPI.
    pub marketplace_program: Pubkey,
    /// Mint address of the reward token.
    pub mint: Pubkey,
    /// PDA bump.
    pub bump: u8,
}

#[error_code]
pub enum MagicTokenError {
    #[msg("Magic token mint does not match the configured address.")]
    UnexpectedMint,
    #[msg("Mint amount must be greater than zero.")]
    InvalidMintAmount,
    #[msg("Recipient token account is not associated with the configured mint.")]
    RecipientMintMismatch,
}
