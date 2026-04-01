use anchor_lang::prelude::*;

declare_id!("Craft1111111111111111111111111111111111111");

#[program]
pub mod crafting {
    use super::*;

    /// Initializes the crafting program state.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

