import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as nacl from "tweetnacl";
import {
  Connection,
  Ed25519Program,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { DiceGame } from "../target/types/dice_game";
import * as crypto from "crypto";
import { assert } from "chai";

describe("dice-game", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.diceGame as Program<DiceGame>;

  let house = Keypair.generate();
  let player = Keypair.generate();
  const vault_seeds = [Buffer.from("vault"), house.publicKey.toBuffer()];
  let vault_pda = PublicKey.findProgramAddressSync(
    vault_seeds,
    program.programId
  );

  let seeds = new anchor.BN(1);
  let bet_pda = PublicKey.findProgramAddressSync(
    [Buffer.from("bet"), vault_pda[0].toBuffer(), u128ToLeBytes(seeds)],
    program.programId
  );
  before(async () => {
    await airdrop(
      provider,
      program.provider.connection,
      house.publicKey,
      LAMPORTS_PER_SOL * 10
    );
    await airdrop(
      provider,
      program.provider.connection,
      player.publicKey,
      LAMPORTS_PER_SOL * 10
    );
  });
  it("Initialize", async () => {
    let amount = new anchor.BN(LAMPORTS_PER_SOL * 2);
    await program.methods
      .initialize(amount)
      .accountsStrict({
        vault: vault_pda[0],
        house: house.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([house])
      .rpc({ commitment: "confirmed" });
    await provider.connection.getAccountInfo(vault_pda[0], "confirmed");
    const balance = await provider.connection.getBalance(
      vault_pda[0],
      "confirmed"
    );
    assert.equal(balance, Number(amount));
  });
  it("place bet", async () => {
    let bet_amount = new anchor.BN(LAMPORTS_PER_SOL);
    let roll = 3;
    await program.methods
      .placeBet(seeds, bet_amount, roll)
      .accountsStrict({
        house: house.publicKey,
        vault: vault_pda[0],
        bet: bet_pda[0],
        player: player.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc({ commitment: "confirmed" });
    const bet_account = await program.account.bet.fetch(
      bet_pda[0],
      "confirmed"
    );
    assert.equal(bet_account.amount.toString(), bet_amount.toString());
    assert.equal(bet_account.roll, roll);
    assert.equal(bet_account.player.toString(), player.publicKey.toString());
    assert.equal(bet_account.seed.toString(), seeds.toString());
  });
  it("settle bet", async () => {
    let betAccount = await anchor
      .getProvider()
      .connection.getAccountInfo(bet_pda[0], "confirmed");

    const parsedBet = await program.account.bet.fetch(bet_pda[0], "confirmed");

    const prePlayerBalance = await provider.connection.getBalance(
      player.publicKey,
      "confirmed"
    );
    const preVaultBalance = await provider.connection.getBalance(
      vault_pda[0],
      "confirmed"
    );

    const message = Buffer.concat([
      parsedBet.player.toBuffer(),
      u128ToLeBytes(new anchor.BN(parsedBet.seed)),
      u64ToLeBytes(new anchor.BN(parsedBet.slot)),
      Buffer.from([parsedBet.roll]),
      u64ToLeBytes(new anchor.BN(parsedBet.amount)),
    ]);

    const sig = Buffer.from(nacl.sign.detached(message, house.secretKey));

    let sig_ix = Ed25519Program.createInstructionWithPrivateKey({
      privateKey: house.secretKey,
      message: message,
    });

    const digest = crypto.createHash("sha256").update(sig).digest();
    const upper = u128FromLeBytes(digest.subarray(0, 16));
    const lower = u128FromLeBytes(digest.subarray(16, 32));
    const roll = Number((upper + lower) % BigInt(100)) + 1;

    const HOUSE_EDGE = BigInt(150);
    let expectedPayout = BigInt(0);
    if (roll <= parsedBet.roll) {
      expectedPayout =
        (BigInt(parsedBet.amount.toString()) * (BigInt(10000) - HOUSE_EDGE)) /
        BigInt(parsedBet.roll) /
        BigInt(100);
    }
    const expectedPayoutNum = Number(expectedPayout.toString());

    const ix = await program.methods
      .resolveBet(sig)
      .accountsStrict({
        house: house.publicKey,
        vault: vault_pda[0],
        bet: bet_pda[0],
        player: player.publicKey,
        instructionSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .signers([house])
      .instruction();

    const tx = new Transaction().add(sig_ix).add(ix);

    await sendAndConfirmTransaction(provider.connection, tx, [house], {
      commitment: "confirmed",
    });

    const postPlayerBalance = await provider.connection.getBalance(
      player.publicKey,
      "confirmed"
    );
    const postVaultBalance = await provider.connection.getBalance(
      vault_pda[0],
      "confirmed"
    );

    const closedBet = await provider.connection.getAccountInfo(bet_pda[0]);
    assert.isNull(closedBet);

    const rentRefund = betAccount!.lamports;
    const playerDelta = postPlayerBalance - prePlayerBalance;
    const vaultDelta = preVaultBalance - postVaultBalance;

    assert.equal(playerDelta, rentRefund + expectedPayoutNum);
    assert.equal(vaultDelta, expectedPayoutNum);
  });
  it("refund bet", async () => {
    const refundSeeds = new anchor.BN(2);
    const refundBetPda = PublicKey.findProgramAddressSync(
      [Buffer.from("bet"), vault_pda[0].toBuffer(), u128ToLeBytes(refundSeeds)],
      program.programId
    );

    const betAmount = new anchor.BN(LAMPORTS_PER_SOL);

    await program.methods
      .placeBet(refundSeeds, betAmount, 10)
      .accountsStrict({
        house: house.publicKey,
        vault: vault_pda[0],
        bet: refundBetPda[0],
        player: player.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([player])
      .rpc();

    try {
      await program.methods
        .refundBet()
        .accountsStrict({
          player: player.publicKey,
          bet: refundBetPda[0],
          vault: vault_pda[0],
          house: house.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([player])
        .rpc();

      assert.fail("Refund should have failed due to timeout not reached");
    } catch (err: any) {
      const errorMsg = err.error?.errorMessage || err.toString();

      assert.include(
        errorMsg,
        "Time not reached yet",
        "Expected TimeoutNotReached error"
      );
    }
  });
});

async function airdrop(
  provider: anchor.AnchorProvider,
  connection: anchor.web3.Connection,
  key: PublicKey,
  amount: number
) {
  await connection.confirmTransaction(
    await connection.requestAirdrop(key, amount),
    "confirmed"
  );
}

function u64ToLeBytes(n: anchor.BN): Buffer {
  return n.toArrayLike(Buffer, "le", 8);
}

function u128ToLeBytes(n: anchor.BN): Buffer {
  return n.toArrayLike(Buffer, "le", 16);
}

function u128FromLeBytes(buf: Buffer): bigint {
  let res = BigInt(0);
  for (let i = 0; i < buf.length; i++) {
    res |= BigInt(buf[i]) << BigInt(8 * i);
  }
  return res;
}
