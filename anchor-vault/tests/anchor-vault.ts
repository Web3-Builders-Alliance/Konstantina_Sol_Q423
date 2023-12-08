import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorVault } from "../target/types/anchor_vault";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js";

describe("anchor-vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.AnchorVault as Program<AnchorVault>;

  const connection = anchor.getProvider().connection;

  const signer = Keypair.generate();

  const vault = PublicKey.findProgramAddressSync([
    Buffer.from("vault"),
    signer.publicKey.toBuffer()],
    program.programId
  )[0];

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block
    })
    return signature
  }

  const log = async(signature: string): Promise<string> => {
    console.log(`Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899`);
    return signature;
  }

  it("Airdrop",async () => {
    await connection.requestAirdrop(signer.publicKey, 10 * LAMPORTS_PER_SOL)
      .then(confirm)
      .then(log);
  })

  it("Deposit", async () => {
    const tx = await program.methods
      .deposit(new BN(1e9))
      .accounts({
        signer: signer.publicKey,
        vault,
        systemProgam: SystemProgram.programId
      })
      .signers([signer])
      .rpc()
      .then(confirm)
      .then(log);
  });

  it("Withdraw", async () => {
    const tx = await program.methods
      .withdraw(new BN(1e9))
      .accounts({
        signer: signer.publicKey,
        vault,
        systemProgam: SystemProgram.programId
      })
      .signers([signer])
      .rpc()
      .then(confirm)
      .then(log);
  });
});
