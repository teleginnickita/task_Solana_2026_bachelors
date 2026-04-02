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

    /// Converts a raw discriminant into an [`ItemType`].
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::KozakSabre),
            1 => Some(Self::ElderStaff),
            2 => Some(Self::KharakternykArmor),
            3 => Some(Self::BattleBracelet),
            _ => None,
        }
    }

    /// Returns the recipe amounts for all six base resources.
    pub fn recipe(self) -> [u64; 6] {
        match self {
            Self::KozakSabre => [1, 3, 0, 1, 0, 0],
            Self::ElderStaff => [2, 0, 1, 0, 0, 1],
            Self::KharakternykArmor => [0, 2, 1, 4, 0, 0],
            Self::BattleBracelet => [0, 4, 2, 0, 0, 2],
        }
    }
}
