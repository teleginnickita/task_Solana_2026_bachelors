use anchor_lang::prelude::*;

declare_id!("xCeFkpeadNjjYyL3BStmgDceC9ZjeRW3DtErAJ1nQ2y");

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
