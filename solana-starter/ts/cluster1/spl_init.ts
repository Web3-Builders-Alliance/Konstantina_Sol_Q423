import { Keypair, Connection, Commitment } from "@solana/web3.js";
import { createMint } from '@solana/spl-token';
import wallet from "../wba-wallet.json"

const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

// npm install inside the ts folder duh

const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

(async () => {
    try {
        const mint = await createMint(connection, keypair, keypair.publicKey, null, 6);
        console.log(mint.toBase58());
    } catch(error) {
        console.log(`Oops, something went wrong: ${error}`)
    }
})()
