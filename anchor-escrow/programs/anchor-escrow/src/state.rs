use anchor_lang::prelude::*;

#[account]
pub struct Escrow {
    pub seed: u64,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive_amount: u64,
    pub bump: u8, // we don't have to save the keys but it's good practice
    pub vault_bump: u8 // what if we don't add this -> we could but we're saving compute
}

impl Space for Escrow {
    const INIT_SPACE: usize = 32 + 8 + 32 + 1 + 1 + 8 + 8; // anchor adds a discriminator of 8 bytes
}

// we didn't create a state folder because we only have one piece of state, the Escrow struct
