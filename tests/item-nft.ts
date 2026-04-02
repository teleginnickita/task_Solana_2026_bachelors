import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";

describe("item_nft", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ItemNft as anchor.Program;
  const owner = provider.wallet;

  async function createFundedWallet(): Promise<anchor.web3.Keypair> {
    const wallet = anchor.web3.Keypair.generate();
    const signature = await provider.connection.requestAirdrop(
      wallet.publicKey,
      anchor.web3.LAMPORTS_PER_SOL,
    );
    await provider.connection.confirmTransaction(signature, "confirmed");
    return wallet;
  }

  it("registers metadata for a supported item type", async () => {
    const mint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item-metadata"), mint.publicKey.toBuffer()],
      program.programId,
    );

    await program.methods
      .registerItemMetadata(0)
      .accounts({
        owner: owner.publicKey,
        mint: mint.publicKey,
        itemMetadata: itemMetadataPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const itemMetadata = await program.account.itemMetadata.fetch(itemMetadataPda);

    expect(itemMetadata.itemType).to.eq(0);
    expect(itemMetadata.owner.toBase58()).to.eq(owner.publicKey.toBase58());
    expect(itemMetadata.mint.toBase58()).to.eq(mint.publicKey.toBase58());
  });

  it("transfers protocol ownership to a new wallet", async () => {
    const mint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item-metadata"), mint.publicKey.toBuffer()],
      program.programId,
    );
    const newOwner = await createFundedWallet();

    await program.methods
      .registerItemMetadata(1)
      .accounts({
        owner: owner.publicKey,
        mint: mint.publicKey,
        itemMetadata: itemMetadataPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    await program.methods
      .transferItemMetadata()
      .accounts({
        owner: owner.publicKey,
        newOwner: newOwner.publicKey,
        itemMetadata: itemMetadataPda,
        mint: mint.publicKey,
      })
      .rpc();

    const itemMetadata = await program.account.itemMetadata.fetch(itemMetadataPda);
    expect(itemMetadata.owner.toBase58()).to.eq(newOwner.publicKey.toBase58());
  });

  it("rejects unsupported item types", async () => {
    const mint = anchor.web3.Keypair.generate();
    const [itemMetadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("item-metadata"), mint.publicKey.toBuffer()],
      program.programId,
    );

    try {
      await program.methods
        .registerItemMetadata(9)
        .accounts({
          owner: owner.publicKey,
          mint: mint.publicKey,
          itemMetadata: itemMetadataPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      expect.fail("Expected unsupported item type to fail");
    } catch (error) {
      expect(`${error}`).to.include("InvalidItemType");
    }
  });
});
