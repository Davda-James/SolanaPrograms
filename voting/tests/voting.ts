import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";

import { Voting } from "../target/types/voting";
import { assert } from "chai";

async function getPdaAddr(seeds: Array<Buffer>, programId: anchor.web3.PublicKey) {
  const PDAaddr = await anchor.web3.PublicKey.findProgramAddressSync( 
    seeds,
    programId
  );
  return PDAaddr;
}
describe("voting", async () => {
  // Configure the client to use the local cluster.
  const connection = new anchor.web3.Connection("http://127.0.0.1:8899", "confirmed");
  const adminKeypair = anchor.web3.Keypair.generate();
  const provider = new anchor.AnchorProvider(connection, new anchor.Wallet(adminKeypair), anchor.AnchorProvider.defaultOptions());
  anchor.setProvider(provider);

  const program = anchor.workspace.Voting as Program<Voting>;

  // constants
  const LAMPORTS_PER_SOL = anchor.web3.LAMPORTS_PER_SOL;

  // generating key pair
  const state = anchor.web3.Keypair.generate();
  const electionCreatorKeypair = anchor.web3.Keypair.generate();
  const proposalCreatorKeypair = anchor.web3.Keypair.generate();
  const voterKeypair = anchor.web3.Keypair.generate();
  
  let feeVault: [anchor.web3.PublicKey, number];
  let electionAccount: [anchor.web3.PublicKey, number];
  let proposalAccount: [anchor.web3.PublicKey, number];
  let voterAccount: [anchor.web3.PublicKey, number];

  before(async () => {
    await airdrop(connection, adminKeypair.publicKey, 10 * LAMPORTS_PER_SOL);
    await airdrop(connection, electionCreatorKeypair.publicKey, 10 * LAMPORTS_PER_SOL);
    await airdrop(connection, proposalCreatorKeypair.publicKey, 10 * LAMPORTS_PER_SOL);
    await airdrop(connection, voterKeypair.publicKey, 10 * LAMPORTS_PER_SOL);
    // derive FeeVault PDA now that program exists
    feeVault = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("FeeVault"), adminKeypair.publicKey.toBuffer()], program.programId);
  });

  it("Initializing global state and fee vault by admin", async () => {
    const txSig =  await program.methods.initialize(new anchor.BN(0.01 * LAMPORTS_PER_SOL), 20)
    .accounts({
      state: state.publicKey,
      feeVault: feeVault[0],
      systemProgram: anchor.web3.SystemProgram.programId,
      admin: adminKeypair.publicKey
    })
    .signers([adminKeypair, state])
    .rpc({ commitment: "confirmed" });

    const stateAccount = await program.account.state.fetch(state.publicKey);

    assert.equal(adminKeypair.publicKey.toBase58(), stateAccount.admin.toBase58());
    assert.equal(stateAccount.electionCount.toNumber(), 0);
    assert.equal(stateAccount.platformFee.toNumber(), 0.01 * LAMPORTS_PER_SOL);
    assert.equal(stateAccount.platformProposalBps, 20);
  });

  it("initializing a new election", async()=> {
    const stateAccount = await program.account.state.fetch(state.publicKey);

    electionAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("Election"), electionCreatorKeypair.publicKey.toBuffer(), stateAccount.electionCount.toArrayLike(Buffer, "le", 8)], program.programId);
    
    const txSig =  await program.methods.initializeElection("First Election" ,new anchor.BN(0.01 * LAMPORTS_PER_SOL)).accounts({
      state: state.publicKey,
      election: electionAccount[0],
      creator: electionCreatorKeypair.publicKey,
      admin: adminKeypair.publicKey,
      feeVault: feeVault[0],
      systemProgram: anchor.web3.SystemProgram.programId,  
    }).signers([electionCreatorKeypair]).rpc({ commitment: "confirmed"});

  const updatedStateAccount = await program.account.state.fetch(state.publicKey);
  const electionAccData = await program.account.election.fetch(electionAccount[0]);
  
  assert.equal(stateAccount.electionCount.toNumber()+1, updatedStateAccount.electionCount.toNumber());
  assert.equal(electionAccData.owner.toBase58(), electionCreatorKeypair.publicKey.toBase58());
  assert.equal(electionAccData.proposalCount.toNumber(), 0);
  assert.equal(electionAccData.name, "First Election");
  assert.equal(electionAccData.proposalFee.toNumber(), new anchor.BN(0.01 * LAMPORTS_PER_SOL).toNumber());
  })

  it("initializing the new proposal on given election", async()=> {
    const electionAccData = await program.account.election.fetch(electionAccount[0]);
    proposalAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("Proposals"), electionAccount[0].toBuffer(), electionAccData.proposalCount.toArrayLike(Buffer, "le", 8)], program.programId);

    const txSig = await program.methods.createProposal("First Proposal to First Election")
    .accounts(
      {
        proposal: proposalAccount[0],
        state: state.publicKey,
        feeVault: feeVault[0],
        election: electionAccount[0],
        owner: electionCreatorKeypair.publicKey,
        creator: proposalCreatorKeypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }
    ).signers([proposalCreatorKeypair]).rpc({ commitment: "confirmed" });
    const updatedElectionAccData = await program.account.election.fetch(electionAccount[0]);
    const proposalAccData = await program.account.proposal.fetch(proposalAccount[0]);

    assert.equal(electionAccData.proposalCount.toNumber()+1, updatedElectionAccData.proposalCount.toNumber());
    assert.equal(proposalAccData.election.toBase58(), electionAccount[0].toBase58());
    assert.equal(proposalAccData.creator.toBase58(), proposalCreatorKeypair.publicKey.toBase58());
    assert.equal(proposalAccData.name, "First Proposal to First Election");
    assert.equal(proposalAccData.voteCount.toNumber(), 0);
  })

  it("initializing voter account for given election", async()=> {
    voterAccount = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("Voter"), voterKeypair.publicKey.toBuffer(), electionAccount[0].toBuffer()], program.programId);
    const txSig = await program.methods.initializeVoter()
    .accounts(
      {
        voterAccount: voterAccount[0],
        election: electionAccount[0],
        voter: voterKeypair.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }
    ).signers([voterKeypair]).rpc({ commitment: "confirmed" });
    const voterAccData = await program.account.voter.fetch(voterAccount[0]);

    assert.equal(voterAccData.election.toBase58(), electionAccount[0].toBase58());
    assert.equal(voterAccData.voter.toBase58(), voterKeypair.publicKey.toBase58());
    assert.equal(voterAccData.hasVoted, false);
  });

  it("voter voting on to first proposal of first election", async() => {
    const proposalAccDataBefore = await program.account.proposal.fetch(proposalAccount[0]);

    await program.methods.voteOnProposal()
    .accounts(
      {
        voterAccount: voterAccount[0],
        election: electionAccount[0], 
        proposal: proposalAccount[0],
        voter: voterKeypair.publicKey,  
      }
    ).signers([voterKeypair]).rpc({ commitment: "confirmed" });

    const proposalAccDataAfter = await program.account.proposal.fetch(proposalAccount[0]);
    const voterAccData = await program.account.voter.fetch(voterAccount[0]);

    assert.equal(proposalAccDataBefore.voteCount.toNumber()+1, proposalAccDataAfter.voteCount.toNumber());
    assert.equal(voterAccData.hasVoted, true);
  })

  it("should not be able to vote on already voted", async()=> {
    try{
      await program.methods.voteOnProposal()
      .accounts(
        {
          voterAccount: voterAccount[0],
          election: electionAccount[0],
          proposal: proposalAccount[0],
          voter: voterKeypair.publicKey,
        }
      ).signers([voterKeypair]).rpc({ commitment: "confirmed" });
    } catch(err) {
      assert.equal(err.error.errorCode.code, "AlreadyVoted");
    }
  });
});

async function airdrop(connection: any , publicKey: anchor.web3.PublicKey, amount: number) {
    await connection.confirmTransaction(await connection.requestAirdrop(publicKey, amount), "confirmed");
}



// 'unhappy magnet pig novel chronic calm drill affair live left music cake'