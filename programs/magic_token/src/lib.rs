use anchor_lang::prelude::*;

declare_id!("2ug8zVrkg3zkpCqEuR4AR2KBEkhSLr49pbcScTLrDKTL");

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
