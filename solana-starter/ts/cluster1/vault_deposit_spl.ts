import {
  Connection,
  Keypair,
  SystemProgram,
  PublicKey,
  Commitment,
} from "@solana/web3.js";
import {
  Program,
  Wallet,
  AnchorProvider,
  Address,
  BN,
} from "@coral-xyz/anchor";
import { WbaVault, IDL } from "./programs/wba_vault";
import wallet from "./wallet/wba-wallet.json";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

// Commitment
const commitment: Commitment = "finalized";

// Create a devnet connection
const connection = new Connection("https://api.devnet.solana.com");

// Create our anchor provider
const provider = new AnchorProvider(connection, new Wallet(keypair), {
  commitment,
});

// Create our program
const program = new Program<WbaVault>(IDL, "D51uEDHLbWAxNfodfQDv7qkp8WZtxrhi3uganGbNos7o" as Address, provider);

const vaultState = new PublicKey("2Zqm3K5oDAndEqz9uebfV5p6w54tnyxNgEaog2YgTpLG");

const vaultAuth = PublicKey.findProgramAddressSync(
  [Buffer.from("auth"), vaultState.toBuffer()],
  program.programId
)[0];

const vault = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), vaultAuth.toBuffer()],
  program.programId
)[0];

// Mint address
const mint = new PublicKey("34FiTENzA8PL7w1tegAveneckP8iWdDXTVNh8kBCiFhH"); // the token I created in spl_init
const token_decimals = 1_000_000n; // bc we used createMint with 6 decimals in spl_init

(async () => {
  try {
    // Get the token account of the fromWallet address, and if it does not exist, create it
    const ownerAta = await getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        mint,
        keypair.publicKey
    );

    // Get the token account of the fromWallet address, and if it does not exist, create it
    const vaultAta = await getOrCreateAssociatedTokenAccount(
        connection,
        keypair,
        mint,
        vaultAuth, // because that's how the program is written
        true // vaultAuth is a PDA so we have to allow owner off-curve
    );
    // got TokenAccountNotFoundError first time I ran this - expected when getOrCreateAssociatedTokenAccount that's not already created?

    const signature = await program.methods
      .depositSpl(new BN(1n * token_decimals)) // we'll deposit 1 Konstantina's coin (run spl_mint if you don't have enough)
      .accounts({
          owner: keypair.publicKey,
          ownerAta: ownerAta.address,
          vaultState,
          vaultAuth,
          vaultAta: vaultAta.address,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId
          // if you're wondering why these aren't hardcoded in the program side, these aren't arguments
          // you just have to pass all associated accounts in each transaction
      })
      .signers([
          keypair
      ]).rpc();
    console.log(`Deposit success! Check out your TX here:\n\nhttps://explorer.solana.com/tx/${signature}?cluster=devnet`);
  } catch (e) {
    console.error(`Oops, something went wrong: ${e}`);
  }
})();
