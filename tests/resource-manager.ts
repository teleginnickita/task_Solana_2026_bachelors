import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("resource_manager", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ResourceManager as anchor.Program;
  const admin = provider.wallet;

  const GAME_CONFIG_SEED = Buffer.from("game-config");
  const RESOURCE_MINT_SEED = Buffer.from("resource-mint");
  const RESOURCE_AUTHORITY_SEED = Buffer.from("resource-authority");
  const TOKEN_2022_PROGRAM_ID = new anchor.web3.PublicKey(
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
  );
  const ASSOCIATED_TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
  );

  const [gameConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [GAME_CONFIG_SEED],
    program.programId,
  );
  const [resourceAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [RESOURCE_AUTHORITY_SEED],
    program.programId,
  );
  const resourceMintPdas = Array.from({ length: 6 }, (_, index) =>
    anchor.web3.PublicKey.findProgramAddressSync(
      [RESOURCE_MINT_SEED, Buffer.from([index])],
      program.programId,
    )[0],
  );

  const magicTokenMint = anchor.workspace.MagicToken.programId;
  const itemPrices = [10, 25, 40, 80].map((value) => new anchor.BN(value));
  const searchProgramId = anchor.workspace.Search.programId;
  const craftingProgramId = anchor.workspace.Crafting.programId;

  async function ensureGameConfig(): Promise<void> {
    const existing = await program.account.gameConfig.fetchNullable(gameConfigPda);
    if (existing) {
      return;
    }

    await program.methods
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

  it("initializes game config", async () => {
    await ensureGameConfig();

    const gameConfig = await program.account.gameConfig.fetch(gameConfigPda);

    expect(gameConfig.admin.toBase58()).to.eq(admin.publicKey.toBase58());
    expect(gameConfig.magicTokenMint.toBase58()).to.eq(magicTokenMint.toBase58());
    expect(gameConfig.resourceMints.map((key: anchor.web3.PublicKey) => key.toBase58())).to.deep.eq(
      resourceMintPdas.map((key) => key.toBase58()),
    );
    expect(gameConfig.itemPrices.map((value: anchor.BN) => value.toNumber())).to.deep.eq([10, 25, 40, 80]);
    expect(gameConfig.searchProgram.toBase58()).to.eq(searchProgramId.toBase58());
    expect(gameConfig.craftingProgram.toBase58()).to.eq(craftingProgramId.toBase58());
  });

  it("initializes a token-2022 mint and mints a resource to the admin", async () => {
    await ensureGameConfig();

    const resourceIndex = 0;
    const resourceMint = resourceMintPdas[resourceIndex];
    const [recipientTokenAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        admin.publicKey.toBuffer(),
        TOKEN_2022_PROGRAM_ID.toBuffer(),
        resourceMint.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    const existingMint = await provider.connection.getAccountInfo(resourceMint);
    if (!existingMint) {
      await program.methods
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

    const createAtaIx = new anchor.web3.TransactionInstruction({
      programId: ASSOCIATED_TOKEN_PROGRAM_ID,
      keys: [
        { pubkey: admin.publicKey, isSigner: true, isWritable: true },
        { pubkey: recipientTokenAccount, isSigner: false, isWritable: true },
        { pubkey: admin.publicKey, isSigner: false, isWritable: false },
        { pubkey: resourceMint, isSigner: false, isWritable: false },
        { pubkey: anchor.web3.SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: TOKEN_2022_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      data: Buffer.alloc(0),
    });

    const ataInfo = await provider.connection.getAccountInfo(recipientTokenAccount);
    if (!ataInfo) {
      await provider.sendAndConfirm(
        new anchor.web3.Transaction().add(createAtaIx),
        [],
      );
    }

    await program.methods
      .mintResource(resourceIndex, new anchor.BN(3))
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
        mintAuthority: resourceAuthorityPda,
        resourceMint,
        recipientTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const mintInfo = await provider.connection.getParsedAccountInfo(resourceMint);
    const tokenBalance = await provider.connection.getTokenAccountBalance(
      recipientTokenAccount,
    );
    const mintData = mintInfo.value?.data;

    expect(mintData).to.not.eq(null);
    if (!mintData || !("parsed" in mintData)) {
      expect.fail("Expected parsed mint account data");
    }

    expect(mintData.parsed.info.decimals).to.eq(0);
    expect(mintData.parsed.info.mintAuthority).to.eq(resourceAuthorityPda.toBase58());
    expect(Number(tokenBalance.value.amount)).to.eq(3);
  });

  it("burns a portion of the admin resource balance", async () => {
    await ensureGameConfig();

    const resourceIndex = 0;
    const resourceMint = resourceMintPdas[resourceIndex];
    const [ownerTokenAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        admin.publicKey.toBuffer(),
        TOKEN_2022_PROGRAM_ID.toBuffer(),
        resourceMint.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    await program.methods
      .burnResource(resourceIndex, new anchor.BN(2))
      .accounts({
        owner: admin.publicKey,
        gameConfig: gameConfigPda,
        resourceMint,
        ownerTokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const tokenBalance = await provider.connection.getTokenAccountBalance(
      ownerTokenAccount,
    );

    expect(Number(tokenBalance.value.amount)).to.eq(1);
  });
});
