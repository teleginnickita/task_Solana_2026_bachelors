use anchor_lang::prelude::*;

declare_id!("GqTny3DGUaCXESufnpUQXG1p8QFodc1aCrYG1qvPkqXd");

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
