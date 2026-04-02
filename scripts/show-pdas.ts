import { PublicKey } from "@solana/web3.js";

const PROGRAM_IDS = {
  resourceManager: new PublicKey("CcvkCG2poiGbhLzkodhefhJbMbBW9AxHdKPR3eHrxfvf"),
  itemNft: new PublicKey("6wUan26ACFhc3DhFPWh3K3QGxhHMAubKAyUtqRfJz9ej"),
  crafting: new PublicKey("GqTny3DGUaCXESufnpUQXG1p8QFodc1aCrYG1qvPkqXd"),
  search: new PublicKey("3kx233sHmZfTrMHJ66sqBip2nAqntfL6y6V219BfmBdN"),
  marketplace: new PublicKey("xCeFkpeadNjjYyL3BStmgDceC9ZjeRW3DtErAJ1nQ2y"),
  magicToken: new PublicKey("2ug8zVrkg3zkpCqEuR4AR2KBEkhSLr49pbcScTLrDKTL"),
};

function printPda(label: string, pubkey: PublicKey) {
  console.log(`${label}: ${pubkey.toBase58()}`);
}

function derivePda(seeds: Buffer[], programId: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(seeds, programId)[0];
}

console.log("Program IDs");
printPda("resource_manager", PROGRAM_IDS.resourceManager);
printPda("item_nft", PROGRAM_IDS.itemNft);
printPda("crafting", PROGRAM_IDS.crafting);
printPda("search", PROGRAM_IDS.search);
printPda("marketplace", PROGRAM_IDS.marketplace);
printPda("magic_token", PROGRAM_IDS.magicToken);

console.log("");
console.log("PDAs");
printPda(
  "resource_manager::game_config",
  derivePda([Buffer.from("game-config")], PROGRAM_IDS.resourceManager),
);
printPda(
  "resource_manager::resource_authority",
  derivePda([Buffer.from("resource-authority")], PROGRAM_IDS.resourceManager),
);
printPda(
  "search::search_authority",
  derivePda([Buffer.from("search-authority")], PROGRAM_IDS.search),
);
printPda(
  "marketplace::marketplace_authority",
  derivePda([Buffer.from("marketplace-authority")], PROGRAM_IDS.marketplace),
);
printPda(
  "magic_token::config",
  derivePda([Buffer.from("magic-token-config")], PROGRAM_IDS.magicToken),
);
printPda(
  "magic_token::authority",
  derivePda([Buffer.from("magic-token-authority")], PROGRAM_IDS.magicToken),
);
printPda(
  "magic_token::mint",
  derivePda([Buffer.from("magic-token-mint")], PROGRAM_IDS.magicToken),
);
