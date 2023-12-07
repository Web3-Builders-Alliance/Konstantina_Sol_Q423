import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorVoteSolb } from "../target/types/anchor_vote_solb";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { createHash } from "crypto";

describe("anchor-vote-solb", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();

  const program = anchor.workspace.AnchorVoteSolb as Program<AnchorVoteSolb>;

  const signer = Keypair.generate(); // we could do anchor.Keypair but we're doing it directly from web3.js

  const site = "google.com";

  const hash = createHash('sha256');

  hash.update(Buffer.from(site));

  let seeds = [hash.digest()];

  const vote = PublicKey.findProgramAddressSync(seeds, program.programId)[0];

  const confirm = async (signature:string) => {
    const block = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature,
      ...block
    })
    return signature;
  }

  const log = async(signature: string): Promise<string> => {
    console.log(`Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${provider.connection.rpcEndpoint}`);
    return signature;
  }

  it("Airdrop",async () => {
    await provider.connection.requestAirdrop(signer.publicKey, LAMPORTS_PER_SOL * 10).then(confirm).then(log);
    console.log("Signer balance", await provider.connection.getBalance(signer.publicKey));
  });

  // after we run initialize once we don't want to run it again so we skip with xit
  xit("Initialize", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize(site)
      .accounts({
        signer: signer.publicKey,
        vote,
        systemProgram: SystemProgram.programId
      })
      .signers([signer])
      .rpc()
      .then(confirm).then(log);
  });
  // error 0x0 means you're attempting to initialize an already initialized account

  it("Upvote", async () => {
    const tx = await program.methods
      .upvote(site)
      .accounts({
        signer: signer.publicKey,
        vote,
        systemProgram: SystemProgram.programId
      })
      .signers([signer])
      .rpc()
      .then(confirm).then(log);
  });
  // https://solana.stackexchange.com/questions/3179/what-is-the-likely-cause-of-the-error-the-program-could-not-deserialize-the-giv

  it("Downvote", async () => {
    const tx = await program.methods
      .downvote(site)
      .accounts({
        signer: signer.publicKey,
        vote,
        systemProgram: SystemProgram.programId
      })
      .signers([signer])
      .rpc()
      .then(confirm).then(log);
  });
});
