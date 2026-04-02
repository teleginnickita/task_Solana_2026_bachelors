import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("magic_token + marketplace", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const magicToken = anchor.workspace.MagicToken as anchor.Program;
  const marketplace = anchor.workspace.Marketplace as anchor.Program;
  const itemNft = anchor.workspace.ItemNft as anchor.Program;
  const resourceManager = anchor.workspace.ResourceManager as anchor.Program;
  const admin = provider.wallet;

  const TOKEN_2022_PROGRAM_ID = new anchor.web3.PublicKey(
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
  );
  const ASSOCIATED_TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
  );

  const [magicTokenConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("magic-token-config")],
    magicToken.programId,
  );
  const [magicTokenAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("magic-token-authority")],
    magicToken.programId,
  );
  const [magicTokenMintPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("magic-token-mint")],
    magicToken.programId,
  );
  const [marketplaceAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("marketplace-authority")],
    marketplace.programId,
  );
  const [gameConfigPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("game-config")],
    resourceManager.programId,
  );
  const resourceMintPdas = Array.from({ length: 6 }, (_, index) =>
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("resource-mint"), Buffer.from([index])],
      resourceManager.programId,
    )[0],
  );
  const itemPrices = [10, 25, 40, 80].map((value) => new anchor.BN(value));

  async function ensureGameConfig(): Promise<void> {
    const existing = await resourceManager.account.gameConfig.fetchNullable(gameConfigPda);
    if (existing) {
      return;
    }

    await resourceManager.methods
      .initializeGameConfig(
        resourceMintPdas,
        magicTokenMintPda,
        itemPrices,
        anchor.workspace.Search.programId,
        anchor.workspace.Crafting.programId,
      )
      .accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  async function ensureMagicConfig(): Promise<void> {
    const existing = await magicToken.account.magicTokenConfig.fetchNullable(magicTokenConfigPda);
    if (existing) {
      return;
    }

    await magicToken.methods
      .initializeMagicTokenConfig(marketplace.programId, magicTokenMintPda)
      .accounts({
        admin: admin.publicKey,
        magicTokenConfig: magicTokenConfigPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  async function ensureMagicMint(): Promise<void> {
    const existing = await provider.connection.getAccountInfo(magicTokenMintPda);
    if (existing) {
      return;
    }

    await magicToken.methods
      .initializeMagicTokenMint()
      .accounts({
        admin: admin.publicKey,
        magicTokenConfig: magicTokenConfigPda,
        mintAuthority: magicTokenAuthorityPda,
        magicTokenMint: magicTokenMintPda,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  async function fundSeller(): Promise<anchor.web3.Keypair> {
    const seller = anchor.web3.Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      seller.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(sig, "confirmed");
    return seller;
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

  it("initializes config and mint, then mints seller rewards through marketplace CPI", async () => {
    await ensureGameConfig();
    await ensureMagicConfig();
    await ensureMagicMint();

    const seller = await fundSeller();
    const sellerAta = await ensureAta(seller, magicTokenMintPda);

    await marketplace.methods
      .mintSellerReward(new anchor.BN(7))
      .accounts({
        marketplaceAuthority: marketplaceAuthorityPda,
        magicTokenConfig: magicTokenConfigPda,
        magicTokenAuthority: magicTokenAuthorityPda,
        magicTokenMint: magicTokenMintPda,
        recipientTokenAccount: sellerAta,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        magicTokenProgram: magicToken.programId,
      })
      .rpc();

    const config = await magicToken.account.magicTokenConfig.fetch(magicTokenConfigPda);
    const mintInfo = await provider.connection.getParsedAccountInfo(magicTokenMintPda);
    const tokenBalance = await provider.connection.getTokenAccountBalance(sellerAta);
    const mintData = mintInfo.value?.data;

    expect(config.marketplaceProgram.toBase58()).to.eq(marketplace.programId.toBase58());
    expect(config.mint.toBase58()).to.eq(magicTokenMintPda.toBase58());
    expect(mintData).to.not.eq(null);
    if (!mintData || !("parsed" in mintData)) {
      expect.fail("Expected parsed mint account data");
    }
    expect(mintData.parsed.info.decimals).to.eq(0);
    expect(mintData.parsed.info.mintAuthority).to.eq(magicTokenAuthorityPda.toBase58());
    expect(Number(tokenBalance.value.amount)).to.eq(7);
  });

  it("sells an item and pays out magic tokens based on game config price", async () => {
    await ensureGameConfig();
    await ensureMagicConfig();
    await ensureMagicMint();

    const seller = await fundSeller();
    const sellerAta = await ensureAta(seller, magicTokenMintPda);
    const itemMint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item-metadata"), itemMint.publicKey.toBuffer()],
      itemNft.programId,
    );

    await itemNft.methods
      .registerItemMetadata(1)
      .accounts({
        owner: seller.publicKey,
        mint: itemMint.publicKey,
        itemMetadata: itemMetadataPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([seller])
      .rpc();

    await marketplace.methods
      .sellItem()
      .accounts({
        seller: seller.publicKey,
        marketplaceAuthority: marketplaceAuthorityPda,
        gameConfig: gameConfigPda,
        magicTokenConfig: magicTokenConfigPda,
        magicTokenAuthority: magicTokenAuthorityPda,
        magicTokenMint: magicTokenMintPda,
        recipientTokenAccount: sellerAta,
        itemMetadata: itemMetadataPda,
        itemMint: itemMint.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        magicTokenProgram: magicToken.programId,
        itemNftProgram: itemNft.programId,
      })
      .signers([seller])
      .rpc();

    const updatedMetadata = await itemNft.account.itemMetadata.fetch(itemMetadataPda);
    const tokenBalance = await provider.connection.getTokenAccountBalance(sellerAta);

    expect(updatedMetadata.owner.toBase58()).to.eq(anchor.web3.PublicKey.default.toBase58());
    expect(Number(tokenBalance.value.amount)).to.eq(25);
  });
});
