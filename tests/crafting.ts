import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("crafting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const crafting = anchor.workspace.Crafting as anchor.Program;
  const resourceManager = anchor.workspace.ResourceManager as anchor.Program;
  const itemNft = anchor.workspace.ItemNft as anchor.Program;
  const admin = provider.wallet;

  const TOKEN_2022_PROGRAM_ID = new anchor.web3.PublicKey(
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
  );
  const ASSOCIATED_TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
  );

  const [gameConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("game-config")],
    resourceManager.programId,
  );
  const [resourceAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("resource-authority")],
    resourceManager.programId,
  );
  const resourceMintPdas = Array.from({ length: 6 }, (_, index) =>
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("resource-mint"), Buffer.from([index])],
      resourceManager.programId,
    )[0],
  );

  const magicTokenMint = anchor.workspace.MagicToken.programId;
  const itemPrices = [10, 25, 40, 80].map((value) => new anchor.BN(value));
  const searchProgramId = anchor.workspace.Search.programId;
  const craftingProgramId = crafting.programId;

  async function ensureGameConfig(): Promise<void> {
    const existing = await resourceManager.account.gameConfig.fetchNullable(gameConfigPda);
    if (existing) {
      return;
    }

    await resourceManager.methods
      .initializeGameConfig(
        resourceMintPdas,
        magicTokenMint,
        itemPrices,
        searchProgramId,
        craftingProgramId,
      )
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  async function ensureResourceMint(resourceIndex: number): Promise<void> {
    const resourceMint = resourceMintPdas[resourceIndex];
    const existing = await provider.connection.getAccountInfo(resourceMint);
    if (existing) {
      return;
    }

    await resourceManager.methods
      .initializeResourceMint(resourceIndex)
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
        mintAuthority: resourceAuthorityPda,
        resourceMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  async function fundCrafter(): Promise<anchor.web3.Keypair> {
    const crafter = anchor.web3.Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      crafter.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig, "confirmed");
    return crafter;
  }

  async function ensureAta(
    owner: anchor.web3.Keypair,
    mint: anchor.web3.PublicKey,
  ): Promise<anchor.web3.PublicKey> {
    const [ata] = anchor.web3.PublicKey.findProgramAddressSync(
      [owner.publicKey.toBuffer(), TOKEN_2022_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    const existing = await provider.connection.getAccountInfo(ata);
    if (!existing) {
      const ix = new anchor.web3.TransactionInstruction({
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        keys: [
          { pubkey: owner.publicKey, isSigner: true, isWritable: true },
          { pubkey: ata, isSigner: false, isWritable: true },
          { pubkey: owner.publicKey, isSigner: false, isWritable: false },
          { pubkey: mint, isSigner: false, isWritable: false },
          { pubkey: anchor.web3.SystemProgram.programId, isSigner: false, isWritable: false },
          { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        ],
        data: Buffer.alloc(0),
      });
      await provider.sendAndConfirm(new anchor.web3.Transaction().add(ix), [owner]);
    }
    return ata;
  }

  it("burns recipe resources and creates item metadata for a sabre", async () => {
    await ensureGameConfig();

    const crafter = await fundCrafter();
    const recipe = [1, 3, 0, 1, 0, 0];
    const ownerTokenAccounts: anchor.web3.PublicKey[] = [];

    for (let index = 0; index < 6; index += 1) {
      await ensureResourceMint(index);
      const ata = await ensureAta(crafter, resourceMintPdas[index]);
      ownerTokenAccounts.push(ata);

      if (recipe[index] > 0) {
        await resourceManager.methods
          .mintResource(index, new anchor.BN(recipe[index]))
          .accounts({
            admin: admin.publicKey,
            gameConfig: gameConfigPda,
            mintAuthority: resourceAuthorityPda,
            resourceMint: resourceMintPdas[index],
            recipientTokenAccount: ata,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .rpc();
      }
    }

    const itemMint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item-metadata"), itemMint.publicKey.toBuffer()],
      itemNft.programId,
    );

    await crafting.methods
      .craftItem(0)
      .accounts({
        owner: crafter.publicKey,
        gameConfig: gameConfigPda,
        resourceMint0: resourceMintPdas[0],
        resourceMint1: resourceMintPdas[1],
        resourceMint2: resourceMintPdas[2],
        resourceMint3: resourceMintPdas[3],
        resourceMint4: resourceMintPdas[4],
        resourceMint5: resourceMintPdas[5],
        ownerTokenAccount0: ownerTokenAccounts[0],
        ownerTokenAccount1: ownerTokenAccounts[1],
        ownerTokenAccount2: ownerTokenAccounts[2],
        ownerTokenAccount3: ownerTokenAccounts[3],
        ownerTokenAccount4: ownerTokenAccounts[4],
        ownerTokenAccount5: ownerTokenAccounts[5],
        itemMint: itemMint.publicKey,
        itemMetadata: itemMetadataPda,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        resourceManagerProgram: resourceManager.programId,
        itemNftProgram: itemNft.programId,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([crafter])
      .rpc();

    const itemMetadata = await itemNft.account.itemMetadata.fetch(itemMetadataPda);
    expect(itemMetadata.itemType).to.eq(0);
    expect(itemMetadata.owner.toBase58()).to.eq(crafter.publicKey.toBase58());
    expect(itemMetadata.mint.toBase58()).to.eq(itemMint.publicKey.toBase58());

    for (let index = 0; index < 6; index += 1) {
      const balance = await provider.connection.getTokenAccountBalance(ownerTokenAccounts[index]);
      expect(Number(balance.value.amount)).to.eq(0);
    }
  });
});
