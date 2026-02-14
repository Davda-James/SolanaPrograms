import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { QuadraticVoting } from "../target/types/quadratic_voting";
import { assert, expect } from "chai";

describe("quadratic voting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.QuadraticVoting as Program<QuadraticVoting>;

  const admin = provider.wallet;
  const voter = Keypair.generate();

  let daoPda: PublicKey;
  let proposalPda: PublicKey;
  let votePda: PublicKey;
  let mint: PublicKey;
  let voterAta: PublicKey;

  it("airdrop voter", async () => {
    const sig = await provider.connection.requestAirdrop(
      voter.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);
  });

  it("initialize dao", async () => {
    [daoPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("dao"), Buffer.from("TEST DAO"), admin.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initializeDao("TEST DAO")
      .accounts({
        admin: admin.publicKey,
        daoAccount: daoPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const dao = await program.account.dao.fetch(daoPda);
    assert.equal(dao.name, "TEST DAO");
    assert.equal(dao.proposalCount.toNumber(), 0);
  });

  it("create mint and mint tokens to voter", async () => {
    mint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      0
    );

    const ata = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin.payer,
      mint,
      voter.publicKey
    );

    voterAta = ata.address;

    await mintTo(
      provider.connection,
      admin.payer,
      mint,
      voterAta,
      admin.publicKey,
      100
    );

    const account = await getAccount(provider.connection, voterAta);
    assert.equal(Number(account.amount), 100);
  });

  it("create proposal", async () => {
    [proposalPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("proposal"),
        daoPda.toBuffer(),
        new anchor.BN(0).toArrayLike(Buffer, "le", 8),
        admin.publicKey.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .createProposal("Test proposal")
      .accounts({
        creator: admin.publicKey,
        daoAccount: daoPda,
        proposal: proposalPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const proposal = await program.account.proposal.fetch(proposalPda);

    assert.equal(proposal.description, "Test proposal");
    assert.equal(proposal.yesVotes.toNumber(), 0);
    assert.equal(proposal.noVotes.toNumber(), 0);
    assert.equal(proposal.dao.toBase58(), daoPda.toBase58());

    const dao = await program.account.dao.fetch(daoPda);
    assert.equal(dao.proposalCount.toNumber(), 1);
  });
  it("vote with quadratic cost", async () => {
    const votes = 3;
    const expectedCost = votes * votes;

    [votePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vote"), voter.publicKey.toBuffer(), proposalPda.toBuffer()],
      program.programId
    );

    await program.methods
      .vote(true, new anchor.BN(votes))
      .accounts({
        voter: voter.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        voteAccount: votePda,
        mint,
        creatorTokenAccount: voterAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([voter])
      .rpc();

    const ataAccount = await getAccount(provider.connection, voterAta);
    assert.equal(Number(ataAccount.amount), 100 - expectedCost);

    const proposal = await program.account.proposal.fetch(proposalPda);
    assert.equal(proposal.yesVotes.toNumber(), votes);
    assert.equal(proposal.noVotes.toNumber(), 0);

    const voteAccount = await program.account.vote.fetch(votePda);
    assert.equal(voteAccount.voteCredits.toNumber(), votes);
    assert.equal(voteAccount.voteType, true);
    assert.equal(voteAccount.authority.toBase58(), voter.publicKey.toBase58());
  });
  it("fails on double voting", async () => {
    try {
      await program.methods
        .vote(true, new anchor.BN(1))
        .accounts({
          voter: voter.publicKey,
          dao: daoPda,
          proposal: proposalPda,
          voteAccount: votePda,
          mint,
          creatorTokenAccount: voterAta,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter])
        .rpc();
      assert.fail("Should raise error for double voting");
    } catch (err) {
      assert.ok(true, "Double voting not prevented");
    }
  });
});
