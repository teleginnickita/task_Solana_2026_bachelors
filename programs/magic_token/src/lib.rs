use anchor_lang::prelude::*;

declare_id!("Magic111111111111111111111111111111111111");

#[program]
pub mod magic_token {
    use super::*;

    /// Initializes the magic token program state.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

