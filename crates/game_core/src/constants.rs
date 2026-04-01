/// PDA seed for the singleton game configuration account.
pub const GAME_CONFIG_SEED: &[u8] = b"game-config";

/// PDA seed for resource mint accounts.
pub const RESOURCE_MINT_SEED: &[u8] = b"resource-mint";

/// PDA seed for the signer authority that controls resource mints.
pub const RESOURCE_AUTHORITY_SEED: &[u8] = b"resource-authority";

/// PDA seed for a player's search state account.
pub const PLAYER_SEED: &[u8] = b"player";

/// PDA seed for per-item metadata accounts.
pub const ITEM_METADATA_SEED: &[u8] = b"item-metadata";

/// Number of base resources in the game.
pub const RESOURCE_COUNT: usize = 6;

/// Number of supported item types in the game config prices array.
pub const ITEM_COUNT: usize = 4;
