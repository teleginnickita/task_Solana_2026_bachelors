use anchor_lang::prelude::*;

declare_id!("Market11111111111111111111111111111111111");

#[program]
pub mod marketplace {
    use super::*;

    /// Initializes the marketplace program state.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

