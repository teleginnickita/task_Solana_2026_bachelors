use anchor_lang::prelude::*;

declare_id!("Search111111111111111111111111111111111111");

#[program]
pub mod search {
    use super::*;

    /// Initializes the search program state.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

