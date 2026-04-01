use anchor_lang::prelude::*;

declare_id!("ItemNFT111111111111111111111111111111111111");

#[program]
pub mod item_nft {
    use super::*;

    /// Initializes the item NFT program state.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

