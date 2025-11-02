import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { assert } from "chai";

import { Keypair, PublicKey, Connection, Commitment } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import wallet from "../../Turbin3-wallet.json";

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);
// Mint address
const mint = new PublicKey("CBNf8NcUxMupnPoVnphAqgEeWoL1wZG5KmFzUrNvSgcW");

describe("vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.vault as Program<Vault>;
  let vaultPDA: anchor.web3.PublicKey;
  let vaultStatePDA: anchor.web3.PublicKey;
  let user1 = anchor.web3.Keypair.generate();
  let user2 = anchor.web3.Keypair.generate();

  before(async () => {
    [vaultStatePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("state"), user1.publicKey.toBuffer()],
      program.programId
    );

    // Derive the PDA for vault and vault state
    [vaultPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vaultStatePDA.toBuffer()],
      program.programId
    );

    // Airdrop SOL to user1 and user2
    await getAirdrop(user1.publicKey);
    await getAirdrop(user2.publicKey);

    //------ Test Details ------//
    console.log("User1 Public Key: ", user1.publicKey.toBase58());
    console.log("User2 Public Key: ", user2.publicKey.toBase58());
    console.log("Vault State PDA: ", vaultStatePDA.toBase58());
    console.log("Vault PDA: ", vaultPDA.toBase58());
  });

  it("Is initialized!", async () => {
    let amount = new anchor.BN(10000);
    // Add your test here.
    const tx = await program.methods
      .initialize(amount)
      .accounts({
        user: user1.publicKey,
        vaultState: vaultStatePDA,
        systemProgram: anchor.web3.SystemProgram.programId,
        vault: vaultPDA,
      } as any)
      .signers([user1])
      .rpc();

    const vaultStateAccount = await program.account.vaultState.fetch(
      vaultStatePDA
    );
    console.log(
      "Vault State Account: ",
      vaultStateAccount.maximumHoldings.toNumber()
    );
  });

  it("Deposit!", async () => {
    const preVaultBalance = await program.provider.connection.getBalance(
      vaultPDA
    );
    let depositAmount = new anchor.BN(5000);
    const tx = await program.methods
      .deposit(depositAmount)
      .accounts({
        signer: user1.publicKey,
        vaultState: vaultStatePDA,
        vault: vaultPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([user1])
      .rpc();

    const postVaultBalance = await program.provider.connection.getBalance(
      vaultPDA
    );
    assert.equal(postVaultBalance, preVaultBalance + depositAmount.toNumber());
  });

  it("Withdraw!", async () => {
    const preVaultBalance = await program.provider.connection.getBalance(
      vaultPDA
    );
    let withdrawAmount = new anchor.BN(2000);
    const tx = await program.methods
      .withdraw(withdrawAmount)
      .accounts({
        signer: user1.publicKey,
        vaultState: vaultStatePDA,
        vault: vaultPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([user1])
      .rpc();
    const postVaultBalance = await program.provider.connection.getBalance(
      vaultPDA
    );
    assert.equal(postVaultBalance, preVaultBalance - withdrawAmount.toNumber());
  });

  const getAirdrop = async (user: anchor.web3.PublicKey) => {
    const tx = await program.provider.connection.requestAirdrop(
      user,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await program.provider.connection.confirmTransaction(tx);
  };
});

describe("vault - SPL Operations!", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.vault as Program<Vault>;
  let vaultPDA: anchor.web3.PublicKey;
  let vaultStatePDA: anchor.web3.PublicKey;
  let user1 = anchor.web3.Keypair.generate();

  it("Create a new token mint and deploy it to devnet", async () => {
    (async () => {
      try {
        // Create an ATA
        const ata = await getOrCreateAssociatedTokenAccount(
          connection,
          keypair,
          mint,
          keypair.publicKey
        );
        console.log(`Your data is : ${ata.address.toBase58()}`);

        // Mint to ATA
        const mintTx = await mintTo(
          connection,
          keypair,
          mint,
          ata.address,
          keypair.publicKey,
          1000000
        );
        console.log(`Your mint txid: ${mintTx}`);
      } catch (error) {
        console.log(`Oops, something went wrong: ${error}`);
      }
    })();
  })
});
