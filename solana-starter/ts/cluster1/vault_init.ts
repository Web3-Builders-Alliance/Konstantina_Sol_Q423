import {
  Connection,
  Keypair,
  SystemProgram,
  PublicKey,
  Commitment,
} from "@solana/web3.js";
import { Program, Wallet, AnchorProvider, Address } from "@coral-xyz/anchor";
import { WbaVault, IDL } from "./programs/wba_vault";
import wallet from "./wallet/wba-wallet.json";
/// J8qKEmQpadFeBuXAVseH8GNrvsyBhMT8MHSVD3enRgJz

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

// Commitment
const commitment: Commitment = "confirmed";

// Create a devnet connection
const connection = new Connection("https://api.devnet.solana.com");

// Create our anchor provider
const provider = new AnchorProvider(connection, new Wallet(keypair), {
  commitment,
});

// Create our program
const program = new Program<WbaVault>(IDL, "D51uEDHLbWAxNfodfQDv7qkp8WZtxrhi3uganGbNos7o" as Address, provider);

// Create a random keypair
const vaultState = Keypair.generate(); // not a PDA - could we do it with a PDA?
// The purpose of the vaultState is to generate the following PDAs
// We can generate several from this
console.log(`Vault public key: ${vaultState.publicKey.toBase58()}`);
// it's all derived from this so that's all we need to keep for the next scripts

// Create the PDA for our enrollment account
// Seeds are "auth", vaultState
const vaultAuth = PublicKey.findProgramAddressSync(
  [Buffer.from("auth"), vaultState.publicKey.toBuffer()],
  program.programId
)[0];

// Create the vault key
// Seeds are "vault", vaultAuth
const vault = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), vaultAuth.toBuffer()],
  program.programId
)[0];

// Execute our enrollment transaction
(async () => {
  try {
    const signature = await program.methods.initialize() // initialize is an instruction that's defined in the IDL
      .accounts({ // we're not signing here so it's the public keys only - but signer:true - but we're in the accounts section, not where u sign at
          owner: keypair.publicKey, // the vault belongs to me now
          vaultState: vaultState.publicKey,
          vaultAuth,
          vault,
          systemProgram: SystemProgram.programId
      }).signers([keypair, vaultState]).rpc(); // vault state needs to only sign in the init, after that my keypair has been set as the owner of the vault and will be the only keypair signing
      // that why vaultState had to be a keypair and not a pda?
    console.log(`Init success! Check out your TX here:\n\nhttps://explorer.solana.com/tx/${signature}?cluster=devnet`);
  } catch (e) {
    console.error(`Oops, something went wrong: ${e}`);
  }
})();
