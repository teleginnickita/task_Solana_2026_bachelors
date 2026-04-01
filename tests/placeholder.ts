import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { describe, it } from "mocha";

describe("search", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Search as anchor.Program;
  const owner = provider.wallet;

  const [playerPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("player"), owner.publicKey.toBuffer()],
    program.programId,
  );

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
});
