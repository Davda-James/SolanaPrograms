import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { Amm } from "../target/types/amm";
import { assert } from "chai";
import {
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

describe("amm", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.amm as Program<Amm>;

  const seed = new anchor.BN(12346789);
  const fee = 20;
  const user = provider.wallet;
  let mint_x: PublicKey, mint_y: PublicKey;
  let user_ata_x: PublicKey, user_ata_y: PublicKey;
  let vault_ata_x: PublicKey, vault_ata_y: PublicKey, user_ata_lp: PublicKey;
  const config_pda = PublicKey.findProgramAddressSync(
    [Buffer.from("config"), seed.toArrayLike(Buffer, "le", 8)],
    program.programId
  )[0];
  const mint_lp = PublicKey.findProgramAddressSync(
    [Buffer.from("lp"), config_pda.toBuffer()],
    program.programId
  )[0];


  before(async () => {
    await airdrop(provider, user.publicKey, 2 * LAMPORTS_PER_SOL);
    mint_x = await createMint(
      provider.connection,
      user.payer,
      user.publicKey,
      null,
      6
    );
    mint_y = await createMint(
      provider.connection,
      user.payer,
      user.publicKey,
      null,
      6
    );
    vault_ata_x = getAssociatedTokenAddressSync(
      mint_x,
      config_pda,
      true
    );

    vault_ata_y = getAssociatedTokenAddressSync(
      mint_y,
      config_pda,
      true
    );
    user_ata_x = (await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user.payer,
      mint_x,
      user.publicKey
    )).address;
    await mintTo(
      provider.connection,
      user.payer,
      mint_x,
      user_ata_x,
      user.payer,
      1_000_000
    );
    user_ata_y = (await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user.payer,
      mint_y,
      user.publicKey
    )).address;

    await mintTo(
      provider.connection,
      user.payer,
      mint_y,
      user_ata_y,
      user.payer,
      1_000_000
    );
    user_ata_lp = getAssociatedTokenAddressSync(
      mint_lp,
      user.publicKey
    );
  });

  it("config initialized!", async () => {
    await program.methods.initializeConfig(seed, fee)
      .accountsStrict({
        mintLp: mint_lp,
        mintX: mint_x,
        mintY: mint_y,
        vaultX: vault_ata_x,
        vaultY: vault_ata_y,
        config: config_pda,
        user: user.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).signers([user.payer]).rpc();
    const configAccount = await program.account.config.fetch(config_pda);
    assert.equal(configAccount.fee, fee);
    assert.equal(configAccount.seed.toString(), seed.toString());
    assert.equal(configAccount.authority?.toBase58(), user.publicKey.toBase58());
    assert.equal(configAccount.mintX.toBase58(), mint_x.toBase58());
    assert.equal(configAccount.mintY.toBase58(), mint_y.toBase58());
    assert.equal(configAccount.locked, false);
  });

  it("add liquidity to pool", async () => {
    let deposit_amount = 1_000_000;
    let max_x = 30, max_y = 20;
    await program.methods.deposit(new anchor.BN(deposit_amount), new anchor.BN(max_x), new anchor.BN(max_y))
      .accountsStrict({
        config: config_pda,
        mintLp: mint_lp,
        mintX: mint_x,
        mintY: mint_y,
        vaultX: vault_ata_x,
        vaultY: vault_ata_y,
        depositerAtaX: user_ata_x,
        depositerAtaY: user_ata_y,
        depositerAtaLp: user_ata_lp,
        depositer: user.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).signers([user.payer]).rpc({ commitment: "confirmed" });

    const user_ata_lp_account = await getAccount(provider.connection, user_ata_lp);
    const vault_x_account = await getAccount(provider.connection, vault_ata_x, 'confirmed');
    const vault_y_account = await getAccount(provider.connection, vault_ata_y, 'confirmed');
    const mint_lp_account = await getMint(provider.connection, mint_lp, 'confirmed');
    assert.equal(user_ata_lp_account.amount > 0, true);
    assert.equal(vault_x_account.amount == BigInt(30), true);
    assert.equal(vault_y_account.amount == BigInt(20), true);
    assert.equal(mint_lp_account.supply == user_ata_lp_account.amount, true);
  })

  it("withdraw liquidity from pool", async () => {
    let withdraw_amount = 500_000;
    let min_x = 10, min_y = 5;
    await program.methods.withdraw(new anchor.BN(withdraw_amount), new anchor.BN(min_x), new anchor.BN(min_y))
      .accountsStrict({
        config: config_pda,
        mintLp: mint_lp,
        mintX: mint_x,
        mintY: mint_y,
        vaultX: vault_ata_x,
        vaultY: vault_ata_y,
        withdrawerAtaX: user_ata_x,
        withdrawerAtaY: user_ata_y,
        withdrawerAtaLp: user_ata_lp,
        withdrawer: user.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).signers([user.payer]).rpc({ commitment: "confirmed" });

    const user_ata_lp_account = await getAccount(provider.connection, user_ata_lp, 'confirmed');
    const vault_x_account = await getAccount(provider.connection, vault_ata_x, 'confirmed');
    const vault_y_account = await getAccount(provider.connection, vault_ata_y, 'confirmed');
    const mint_lp_account = await getMint(provider.connection, mint_lp, 'confirmed');
    assert.equal(user_ata_lp_account.amount.toString() == BigInt(500_000).toString(), true);
    assert.equal(mint_lp_account.supply.toString() == user_ata_lp_account.amount.toString(), true);
  })
  it("swap x for y", async () => {
    let swap_amount = 10;
    let min_y = 3;
    const user_ata_oldx_account = await getAccount(provider.connection, user_ata_x, 'confirmed');
    const user_ata_oldy_account = await getAccount(provider.connection, user_ata_y, 'confirmed');
    await program.methods.swap(true, new anchor.BN(swap_amount), new anchor.BN(min_y))
      .accountsStrict({
        user: user.publicKey,
        mintX: mint_x,
        mintY: mint_y,
        vaultX: vault_ata_x,
        vaultY: vault_ata_y,
        userAtaX: user_ata_x,
        userAtaY: user_ata_y,
        config: config_pda,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).signers([user.payer]).rpc({ commitment: "confirmed" });

    const user_ata_x_account = await getAccount(provider.connection, user_ata_x, 'confirmed');
    const user_ata_y_account = await getAccount(provider.connection, user_ata_y, 'confirmed');
    const vault_ata_x_account = await getAccount(provider.connection, vault_ata_x, 'confirmed');
    const vault_ata_y_account = await getAccount(provider.connection, vault_ata_y, 'confirmed');
    assert.equal(user_ata_oldx_account.amount > user_ata_x_account.amount, true);
    assert.equal(user_ata_y_account.amount > user_ata_oldy_account.amount, true);
  })
  it("swap y for x", async () => {
    let swap_amount = 100_000;
    let min_x = 4;
    const user_ata_oldx_account = await getAccount(provider.connection, user_ata_x, 'confirmed');
    const user_ata_oldy_account = await getAccount(provider.connection, user_ata_y, 'confirmed');

    await program.methods.swap(false, new anchor.BN(swap_amount), new anchor.BN(min_x))
      .accountsStrict({
        user: user.publicKey,
        mintX: mint_x,
        mintY: mint_y,
        vaultX: vault_ata_x,
        vaultY: vault_ata_y,
        userAtaX: user_ata_x,
        userAtaY: user_ata_y,
        config: config_pda,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).signers([user.payer]).rpc({ commitment: "confirmed" });

    const user_ata_x_account = await getAccount(provider.connection, user_ata_x, 'confirmed');
    const user_ata_y_account = await getAccount(provider.connection, user_ata_y, 'confirmed');
    assert.equal(user_ata_oldx_account.amount < user_ata_x_account.amount, true);
    assert.equal(user_ata_y_account.amount < user_ata_oldy_account.amount, true);
  });
});


async function airdrop(provider: anchor.AnchorProvider, publicKey: anchor.web3.PublicKey, amount: number) {
  const airdropSignature = await provider.connection.requestAirdrop(publicKey, amount);
  await provider.connection.confirmTransaction(airdropSignature, "confirmed");
}