use anchor_lang::prelude::*;

declare_id!("ResMng1111111111111111111111111111111111");

#[program]
pub mod resource_manager {
    use super::*;

    /// Initializes the resource manager global configuration.
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

