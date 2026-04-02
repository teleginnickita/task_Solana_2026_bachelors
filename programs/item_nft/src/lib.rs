use anchor_lang::prelude::*;
use game_core::{
    constants::ITEM_METADATA_SEED,
    types::ItemType,
};

declare_id!("6wUan26ACFhc3DhFPWh3K3QGxhHMAubKAyUtqRfJz9ej");

#[program]
pub mod item_nft {
    use super::*;

    /// Registers protocol metadata for a newly created NFT mint.
    pub fn register_item_metadata(
        ctx: Context<RegisterItemMetadata>,
        item_type: u8,
    ) -> Result<()> {
        require!(ItemType::is_supported(item_type), ItemNftError::InvalidItemType);

        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.item_type = item_type;
        item_metadata.owner = ctx.accounts.owner.key();
        item_metadata.mint = ctx.accounts.mint.key();
        item_metadata.bump = ctx.bumps.item_metadata;

        Ok(())
    }

    /// Updates protocol metadata ownership after an item transfer.
    pub fn transfer_item_metadata(ctx: Context<TransferItemMetadata>) -> Result<()> {
        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.owner = ctx.accounts.new_owner.key();

        Ok(())
    }

    /// Marks item metadata as burned after a successful marketplace sale.
    pub fn burn_item_metadata(ctx: Context<BurnItemMetadata>) -> Result<()> {
        let item_metadata = &mut ctx.accounts.item_metadata;
        item_metadata.owner = Pubkey::default();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct RegisterItemMetadata<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: The mint account will later be constrained by the NFT minting flow.
    pub mint: UncheckedAccount<'info>,
    #[account(
        init,
        payer = owner,
        space = 8 + ItemMetadata::INIT_SPACE,
        seeds = [ITEM_METADATA_SEED, mint.key().as_ref()],
        bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferItemMetadata<'info> {
    pub owner: Signer<'info>,
    /// CHECK: The new owner can be any system account or wallet.
    pub new_owner: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [ITEM_METADATA_SEED, mint.key().as_ref()],
        bump = item_metadata.bump,
        has_one = owner,
        has_one = mint
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    /// CHECK: The mint is tied to the PDA derivation above.
    pub mint: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct BurnItemMetadata<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [ITEM_METADATA_SEED, mint.key().as_ref()],
        bump = item_metadata.bump,
        has_one = owner,
        has_one = mint
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    /// CHECK: The mint is tied to the PDA derivation above.
    pub mint: UncheckedAccount<'info>,
}

#[error_code]
pub enum ItemNftError {
    #[msg("Unsupported item type.")]
    InvalidItemType,
}

/// Stores protocol-owned metadata for a crafted NFT item.
#[account]
#[derive(InitSpace)]
pub struct ItemMetadata {
    /// Encoded [`ItemType`] discriminant.
    pub item_type: u8,
    /// Current wallet owner of the NFT according to the protocol state.
    pub owner: Pubkey,
    /// Mint address of the NFT.
    pub mint: Pubkey,
    /// PDA bump for this account.
    pub bump: u8,
}
