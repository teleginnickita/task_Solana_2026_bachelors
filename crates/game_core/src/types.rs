use anchor_lang::prelude::*;

/// Enumerates all supported base resources.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceKind {
    Wood = 0,
    Iron = 1,
    Gold = 2,
    Leather = 3,
    Stone = 4,
    Diamond = 5,
}

/// Enumerates all craftable item types.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ItemType {
    KozakSabre = 0,
    ElderStaff = 1,
    KharakternykArmor = 2,
    BattleBracelet = 3,
}

impl ItemType {
    /// Returns `true` if the raw discriminant maps to a supported item type.
    pub fn is_supported(value: u8) -> bool {
        value <= Self::BattleBracelet as u8
    }
}

