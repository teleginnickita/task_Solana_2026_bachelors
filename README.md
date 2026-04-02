# Гра "Козацький бізнес" — Solana / Anchor

Реалізація тестового завдання WhiteBIT для екосистеми Solana.

Проєкт побудований як Anchor workspace з кількома програмами, які взаємодіють через CPI:

1. `resource_manager`
2. `search`
3. `item_nft`
4. `crafting`
5. `magic_token`
6. `marketplace`

## Статус реалізації

У поточній версії вже реалізовано:

1. `search`:
   ончейн `Player` PDA, cooldown 60 секунд, deterministic search, CPI-мінт 3 ресурсів через `resource_manager`.
2. `resource_manager`:
   `GameConfig`, ініціалізація Token-2022 resource mint-ів, контрольований mint, burn і CPI mint для `search`.
3. `item_nft`:
   `ItemMetadata`, реєстрація metadata, transfer metadata ownership, burn metadata.
4. `crafting`:
   рецепти предметів, burn ресурсів через `resource_manager`, створення `ItemMetadata` через `item_nft`.
5. `magic_token`:
   окремий config, Token-2022 mint, контрольований mint тільки через `marketplace`.
6. `marketplace`:
   payout seller-у через `magic_token`, продаж предмета з нарахуванням `MagicToken` за ціною з `GameConfig`.

## Важливе уточнення

У README завдання предмети описані як NFT через Metaplex. У цій реалізації поки що завершений протокольний шар предметів через `ItemMetadata`, але повний Metaplex NFT mint/burn flow ще не доданий.

Тобто зараз:

1. предмет має унікальний `mint` pubkey у metadata;
2. ownership і продаж контролюються ончейн;
3. але повний Metaplex Token Metadata lifecycle ще може бути доданий як наступний етап.

## Program IDs

`Anchor.toml` вже синхронізований з локальними keypair-ами:

| Program | Address |
|---|---|
| `resource_manager` | `CcvkCG2poiGbhLzkodhefhJbMbBW9AxHdKPR3eHrxfvf` |
| `item_nft` | `6wUan26ACFhc3DhFPWh3K3QGxhHMAubKAyUtqRfJz9ej` |
| `crafting` | `GqTny3DGUaCXESufnpUQXG1p8QFodc1aCrYG1qvPkqXd` |
| `search` | `3kx233sHmZfTrMHJ66sqBip2nAqntfL6y6V219BfmBdN` |
| `marketplace` | `xCeFkpeadNjjYyL3BStmgDceC9ZjeRW3DtErAJ1nQ2y` |
| `magic_token` | `2ug8zVrkg3zkpCqEuR4AR2KBEkhSLr49pbcScTLrDKTL` |

## Архітектура

### `resource_manager`

Відповідає за:

1. `GameConfig`
2. resource mint-и Token-2022
3. контрольований mint / burn ресурсів
4. CPI mint rewards для `search`

Основний PDA:

1. `game-config`
2. `resource-authority`
3. `resource-mint::<resource_index>`

### `search`

Відповідає за:

1. створення `Player` PDA
2. cooldown 60 секунд
3. генерацію 3 ресурсів
4. CPI mint через `resource_manager`

Основний PDA:

1. `player::<owner>`
2. `search-authority`

### `item_nft`

Відповідає за:

1. `ItemMetadata`
2. register / transfer / burn metadata

Основний PDA:

1. `item-metadata::<item_mint>`

### `crafting`

Відповідає за:

1. перевірку рецепта
2. burn потрібних ресурсів через CPI в `resource_manager`
3. реєстрацію `ItemMetadata` через CPI в `item_nft`

Підтримані рецепти:

| Item type | Recipe |
|---|---|
| `0` `KozakSabre` | `1 WOOD + 3 IRON + 1 LEATHER` |
| `1` `ElderStaff` | `2 WOOD + 1 GOLD + 1 DIAMOND` |
| `2` `KharakternykArmor` | `2 IRON + 1 GOLD + 4 LEATHER` |
| `3` `BattleBracelet` | `4 IRON + 2 GOLD + 2 DIAMOND` |

### `magic_token`

Відповідає за:

1. `MagicTokenConfig`
2. reward mint Token-2022
3. контрольований mint тільки для `marketplace`

Основний PDA:

1. `magic-token-config`
2. `magic-token-authority`
3. `magic-token-mint`

### `marketplace`

Відповідає за:

1. payout seller-у в `MagicToken`
2. продаж предмета за ціною з `GameConfig.item_prices`
3. burn/deactivate `ItemMetadata`

Основний PDA:

1. `marketplace-authority`

## Основні акаунти

### `GameConfig`

```rust
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,
    pub item_prices: [u64; 4],
    pub search_program: Pubkey,
    pub crafting_program: Pubkey,
    pub bump: u8,
}
```

### `Player`

```rust
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub last_found_resources: [u8; 3],
    pub bump: u8,
}
```

### `ItemMetadata`

```rust
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}
```

### `MagicTokenConfig`

```rust
pub struct MagicTokenConfig {
    pub admin: Pubkey,
    pub marketplace_program: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}
```

## Тести

Поточні інтеграційні тести покривають:

1. `search`:
   ініціалізацію player state, cooldown, CPI mint ресурсів.
2. `resource_manager`:
   `GameConfig`, resource mint initialization, mint, burn.
3. `item_nft`:
   register, transfer, invalid item type.
4. `crafting`:
   burn recipe resources і створення `ItemMetadata`.
5. `magic_token + marketplace`:
   reward mint через CPI і продаж предмета з payout seller-у.

## Структура тестів

1. [tests/placeholder.ts](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/tests/placeholder.ts)
2. [tests/resource-manager.ts](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/tests/resource-manager.ts)
3. [tests/item-nft.ts](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/tests/item-nft.ts)
4. [tests/crafting.ts](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/tests/crafting.ts)
5. [tests/magic-token.ts](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/tests/magic-token.ts)

## Локальний запуск

### 1. Встановити залежності

```bash
yarn install
```

### 2. Зібрати програми

```bash
anchor build
```

### 3. Запустити тести

```bash
anchor test
```

## Корисний скрипт

Є невеликий helper-скрипт, який друкує `Program ID` та основні PDA:

```bash
yarn ts-mocha -p ./tsconfig.json scripts/show-pdas.ts
```

Або напряму через `ts-node`, якщо він у тебе встановлений глобально.

## Деплой на Devnet

### 1. Перемкнути Solana CLI

```bash
solana config set --url devnet
solana config set --keypair ~/.config/solana/id.json
```

### 2. Переконатися, що в `Anchor.toml` коректні адреси програм

Програма вже прив’язана до локально згенерованих keypair-ів у `target/deploy`.

### 3. Отримати SOL на devnet

```bash
solana airdrop 2
```

### 4. Деплой

```bash
anchor deploy
```

## Приклади сценаріїв взаємодії

### Пошук ресурсів

1. Ініціалізувати `Player`
2. Викликати `search_and_mint_resources`
3. Отримати 3 ресурси на Token-2022 акаунти

### Крафт предмета

1. Підготувати ресурсні token accounts
2. Накопичити потрібні ресурси
3. Викликати `craft_item(item_type)`
4. Отримати `ItemMetadata`

### Продаж предмета

1. Мати `ItemMetadata`, де `owner == seller`
2. Викликати `marketplace::sell_item`
3. Отримати `MagicToken`
4. `ItemMetadata.owner` стає default pubkey, що позначає спалення/деактивацію

## Що ще можна доробити

1. Повний Metaplex NFT mint/burn flow замість metadata-only предметів.
2. Окремі deploy / bootstrap scripts для devnet.
3. Більше негативних тестів на access control.
4. Простий frontend для демонстрації гри.

## Файли з основною логікою

1. [programs/resource_manager/src/lib.rs](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/programs/resource_manager/src/lib.rs)
2. [programs/search/src/lib.rs](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/programs/search/src/lib.rs)
3. [programs/item_nft/src/lib.rs](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/programs/item_nft/src/lib.rs)
4. [programs/crafting/src/lib.rs](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/programs/crafting/src/lib.rs)
5. [programs/magic_token/src/lib.rs](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/programs/magic_token/src/lib.rs)
6. [programs/marketplace/src/lib.rs](/home/nick_tieliehin2004/SolanaProject/task_Solana_2026_bachelors/programs/marketplace/src/lib.rs)
