import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { describe, it } from "mocha";

describe("search", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Search as anchor.Program;
  const resourceManager = anchor.workspace.ResourceManager as anchor.Program;
  const owner = provider.wallet;
  const TOKEN_2022_PROGRAM_ID = new anchor.web3.PublicKey(
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
  );
  const ASSOCIATED_TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
  );

  const [playerPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("player"), owner.publicKey.toBuffer()],
    program.programId,
  );
  const [searchAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("search-authority")],
    program.programId,
  );
  const [gameConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("game-config")],
    resourceManager.programId,
  );
  const [resourceManagerAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
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

  function predictedResources(ownerPubkey: anchor.web3.PublicKey, lastSearchTimestamp: number): number[] {
    const ownerBytes = ownerPubkey.toBytes();
    const timestampBias = new anchor.BN(lastSearchTimestamp).toArrayLike(Buffer, "le", 8)[0];
    const base = (ownerBytes[0] + timestampBias) % 256;

    return [0, 1, 2].map((index) => (base + index * 2) % 6);
  }

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
        program.programId,
        anchor.workspace.Crafting.programId,
      )
      .accounts({
        admin: owner.publicKey,
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
        admin: owner.publicKey,
        gameConfig: gameConfigPda,
        mintAuthority: resourceManagerAuthorityPda,
        resourceMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  async function ensureAta(
    ataOwner: anchor.web3.Keypair,
    mint: anchor.web3.PublicKey,
  ): Promise<anchor.web3.PublicKey> {
    const [ata] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        ataOwner.publicKey.toBuffer(),
        TOKEN_2022_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    const existing = await provider.connection.getAccountInfo(ata);
    if (!existing) {
      const createAtaIx = new anchor.web3.TransactionInstruction({
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        keys: [
          { pubkey: ataOwner.publicKey, isSigner: true, isWritable: true },
          { pubkey: ata, isSigner: false, isWritable: true },
          { pubkey: ataOwner.publicKey, isSigner: false, isWritable: false },
          { pubkey: mint, isSigner: false, isWritable: false },
          { pubkey: anchor.web3.SystemProgram.programId, isSigner: false, isWritable: false },
          { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
        ],
        data: Buffer.alloc(0),
      });

      await provider.sendAndConfirm(new anchor.web3.Transaction().add(createAtaIx), [ataOwner]);
    }

    return ata;
  }

  it("initializes player state", async () => {
    await program.methods
      .initializePlayer()
      .accounts({
        owner: owner.publicKey,
        player: playerPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const player = await program.account.player.fetch(playerPda);

    expect(player.owner.toBase58()).to.eq(owner.publicKey.toBase58());
    expect(player.lastSearchTimestamp.toNumber()).to.eq(0);
    expect(player.lastFoundResources.map(Number)).to.deep.eq([0, 0, 0]);
  });

  it("performs one search and enforces cooldown", async () => {
    await program.methods
      .searchResources()
      .accounts({
        owner: owner.publicKey,
        player: playerPda,
      })
      .rpc();

    const player = await program.account.player.fetch(playerPda);

    expect(player.lastSearchTimestamp.toNumber()).to.be.greaterThan(0);
    expect(player.lastFoundResources).to.have.length(3);
    for (const resourceId of player.lastFoundResources) {
      expect(Number(resourceId)).to.be.at.least(0);
      expect(Number(resourceId)).to.be.lessThan(6);
    }

    try {
      await program.methods
        .searchResources()
        .accounts({
          owner: owner.publicKey,
          player: playerPda,
        })
        .rpc();

      expect.fail("Expected cooldown error on immediate repeated search");
    } catch (error) {
      expect(`${error}`).to.include("SearchCooldownActive");
    }
  });

  it("searches and mints three resources through resource_manager CPI", async () => {
    const ownerForMinting = anchor.web3.Keypair.generate();
    const airdropSignature = await provider.connection.requestAirdrop(
      ownerForMinting.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(airdropSignature, "confirmed");

    const [ownerPlayerPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), ownerForMinting.publicKey.toBuffer()],
      program.programId,
    );

    await ensureGameConfig();
    await program.methods
      .initializePlayer()
      .accounts({
        owner: ownerForMinting.publicKey,
        player: ownerPlayerPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([ownerForMinting])
      .rpc();

    const expectedResources = predictedResources(ownerForMinting.publicKey, 0);
    for (const resourceIndex of expectedResources) {
      await ensureResourceMint(resourceIndex);
    }

    const resourceMint0 = resourceMintPdas[expectedResources[0]];
    const resourceMint1 = resourceMintPdas[expectedResources[1]];
    const resourceMint2 = resourceMintPdas[expectedResources[2]];
    const recipientTokenAccount0 = await ensureAta(ownerForMinting, resourceMint0);
    const recipientTokenAccount1 = await ensureAta(ownerForMinting, resourceMint1);
    const recipientTokenAccount2 = await ensureAta(ownerForMinting, resourceMint2);

    await program.methods
      .searchAndMintResources()
      .accounts({
        owner: ownerForMinting.publicKey,
        player: ownerPlayerPda,
        searchAuthority: searchAuthorityPda,
        gameConfig: gameConfigPda,
        resourceManagerAuthority: resourceManagerAuthorityPda,
        resourceMint0,
        resourceMint1,
        resourceMint2,
        recipientTokenAccount0,
        recipientTokenAccount1,
        recipientTokenAccount2,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        resourceManagerProgram: resourceManager.programId,
      })
      .signers([ownerForMinting])
      .rpc();

    const player = await program.account.player.fetch(ownerPlayerPda);
    expect(player.lastFoundResources.map(Number)).to.deep.eq(expectedResources);

    const balance0 = await provider.connection.getTokenAccountBalance(recipientTokenAccount0);
    const balance1 = await provider.connection.getTokenAccountBalance(recipientTokenAccount1);
    const balance2 = await provider.connection.getTokenAccountBalance(recipientTokenAccount2);

    expect(Number(balance0.value.amount)).to.eq(1);
    expect(Number(balance1.value.amount)).to.eq(1);
    expect(Number(balance2.value.amount)).to.eq(1);
  });
});
