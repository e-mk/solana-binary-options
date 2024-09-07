import * as anchor from "@coral-xyz/anchor";
import { Program, BN, Wallet } from "@coral-xyz/anchor";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver"
import { BinaryOptionsTestTask,
 } from "../target/types/binary_options_test_task";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  Connection
} from "@solana/web3.js";
import {
  MINT_SIZE,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeMint2Instruction,
  createMintToInstruction,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from "@solana/spl-token";
import { randomBytes } from "crypto";
import assert from "assert";
import jsonWallet from "./key.json";

const SOL_PRICE_FEED_ID = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";

const confirmTx = async (signature: string) => {
  const latestBlockhash = await anchor
    .getProvider()
    .connection.getLatestBlockhash();
  await anchor.getProvider().connection.confirmTransaction({
    signature,
    ...latestBlockhash,
  });
  return signature;
};

const log = async (signature: string): Promise<string> => {
  console.log(
    `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${
      anchor.getProvider().connection.rpcEndpoint
    }`
  );
  return signature.toString();
};

async function getTokenAccountBalance(
  connection: Connection,
  pk: PublicKey
): Promise<bigint> {
  let amount = (await connection.getTokenAccountBalance(pk)).value.amount;

  return BigInt(amount);
}

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}


describe("binary-options-test-task", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const mmSeed = new BN(randomBytes(8));

  const provider = anchor.getProvider();
  const wallet = anchor.AnchorProvider.local().wallet as Wallet;
  const connection = provider.connection;
  const program = anchor.workspace.BinaryOptionsTestTask as Program<BinaryOptionsTestTask>;

  const adminKeypair = Keypair.fromSecretKey(new Uint8Array(jsonWallet));
  const marketMaker = Keypair.generate();
  const bettor = Keypair.generate();
  const mintAuth = Keypair.generate();
  const mint = Keypair.generate();
  const marketMakerAta = getAssociatedTokenAddressSync(mint.publicKey, marketMaker.publicKey, false, TOKEN_2022_PROGRAM_ID)
  const bettorAta = getAssociatedTokenAddressSync(mint.publicKey, bettor.publicKey, false, TOKEN_2022_PROGRAM_ID)
  const houseAta = getAssociatedTokenAddressSync(mint.publicKey, adminKeypair.publicKey, false, TOKEN_2022_PROGRAM_ID)

  const pythSolanaReceiver = new PythSolanaReceiver({ connection, wallet});
  const solUsdPriceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(0, SOL_PRICE_FEED_ID).toBase58();

  console.log("adminKeypair : " + adminKeypair.publicKey)
  console.log("marketMaker : " + marketMaker.publicKey)
  console.log("bettor : " + bettor.publicKey)
  console.log("solUsdPriceFeedAccount : " + solUsdPriceFeedAccount)

  const binaryOptions = PublicKey.findProgramAddressSync(
    [Buffer.from("binary_options"), marketMaker.publicKey.toBuffer(), mmSeed.toArrayLike(Buffer, "le", 8)],
    program.programId
  )[0];

  const vault = getAssociatedTokenAddressSync(mint.publicKey, binaryOptions, true, TOKEN_2022_PROGRAM_ID);

  it("Airdrop and create mints", async () => {
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    let tx = new Transaction();
    tx.instructions = [
      ...[marketMaker, bettor, mintAuth, adminKeypair].map((account) =>
        SystemProgram.transfer({
          fromPubkey: provider.publicKey,
          toPubkey: account.publicKey,
          lamports: 10 * LAMPORTS_PER_SOL,
        })
      ),
      SystemProgram.createAccount({
        fromPubkey: provider.publicKey,
        newAccountPubkey: mint.publicKey,
        lamports,
        space: MINT_SIZE,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      
      // { mint: mint.publicKey, authority: marketMaker.publicKey, ata: marketMakerAta }.
      createInitializeMint2Instruction(mint.publicKey, 6, mintAuth.publicKey, null, TOKEN_2022_PROGRAM_ID),
      
      // market maker
      createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, marketMakerAta, marketMaker.publicKey, mint.publicKey, TOKEN_2022_PROGRAM_ID),
      createMintToInstruction(mint.publicKey, marketMakerAta, mintAuth.publicKey, 1000, undefined, TOKEN_2022_PROGRAM_ID),

      // bettor
      createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, bettorAta, bettor.publicKey, mint.publicKey, TOKEN_2022_PROGRAM_ID),
      createMintToInstruction(mint.publicKey, bettorAta, mintAuth.publicKey, 1000, undefined, TOKEN_2022_PROGRAM_ID),

      // house
      createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, houseAta, adminKeypair.publicKey, mint.publicKey, TOKEN_2022_PROGRAM_ID),
      createMintToInstruction(mint.publicKey, houseAta, mintAuth.publicKey, 1, undefined, TOKEN_2022_PROGRAM_ID),
    ];

    await provider.sendAndConfirm(tx, [mint, mintAuth]).then(log);
  });


  it("Initialize_Binary_Option", async () => {

    // const timestamp = Date.now() + 60 * 60 * 1000;
    const timestamp = Date.now() + (2 * 1000);

    await program.methods
    // (seed: u64, amount: u64, condition: String, price_condition: u64, time_condition: i64)
      .initialize(mmSeed, new BN(1000), "less", new BN(2000), new BN(timestamp))
      .accountsPartial({ 
        authority: marketMaker.publicKey,
        mint: mint.publicKey,
        marketMakerAta: marketMakerAta,
        binaryOption: binaryOptions,
        vault,
        // associated_token_program,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        // system_program,
       })
      .signers([marketMaker])
      .rpc()
      .then(confirmTx)
      .then(log);

      assert.equal(
        await getTokenAccountBalance(provider.connection, vault),
        new BN(1000).toString(),
        "vault should have the right amount of tokens"
      );
  });

  it("Do_Bet", async () => {

    await program.methods
      .doBet()
      .accountsPartial({ 
        bettor: bettor.publicKey,
        mint: mint.publicKey,
        marketMaker: marketMaker.publicKey,
        binaryOption: binaryOptions,
        vault,
        // associated_token_program,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        // system_program,
       })
      .signers([bettor])
      .rpc()
      .then(confirmTx)
      .then(log);

      assert.equal(
        await getTokenAccountBalance(provider.connection, vault),
        new BN(2000).toString(),
        "vault should have the right amount of tokens"
      );
  });


  it("Resolve", async () => {

    await sleep(4000);

    await program.methods
      .resolve()
      .accountsPartial({ 
        signer: adminKeypair.publicKey,
        house: adminKeypair.publicKey,
        houseAta: houseAta,
        marketMaker: marketMaker.publicKey,
        bettor: bettor.publicKey,
        mint: mint.publicKey,
        binaryOption: binaryOptions,
        priceUpdate: solUsdPriceFeedAccount,
        vault,
        // associated_token_program,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        // system_program,
       })
      .signers([adminKeypair])
      .rpc()
      .then(confirmTx)
      .then(log);

      assert.equal(
        await getTokenAccountBalance(provider.connection, houseAta),
        new BN(21).toString(),
        "House should have the right amount of tokens"
      );

      assert.equal(
        await getTokenAccountBalance(provider.connection, marketMakerAta),
        new BN(1980).toString(),
        "Market Maker should have the right amount of tokens"
      );
  });
});
