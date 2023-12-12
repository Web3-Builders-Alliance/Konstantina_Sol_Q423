use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, Transfer, transfer}, associated_token::AssociatedToken};
use crate::state::Escrow;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make <'info>{
    #[account(mut)]
    pub maker: Signer<'info>, // signer

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        // init, ... maker_ata should already exist if thety're making
        // payer=maker,
        mut,
        associated_token::mint = mint_a, // guaranteed to match mint_a
        associated_token::authority = maker
    )]
    pub maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        // init_if_needed, // although normally it shouldn't exist
        init,
        payer=maker,
        seeds = [b"vault", escrow.key().as_ref()], // now we don't have to put the seed in
        // by using the escrow key we're inheriting the entropy of the seed
        bump,
        token::mint = mint_a, // why not an associated token here? we could, instead of having the seeds but not sure of the effective difference
        token::authority = escrow, // why not an associated token here?
        // we could even set the authority to vault
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE,
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>, // an account storing our escrow details

    pub associated_token_program: Program<'info, AssociatedToken>, // needed because our escrow uses spl tokens
    pub token_program: Program<'info, Token>, // wouldn't be needed if we were just using sol
    pub system_program: Program<'info, System> // must have system program if we're initializing any account
}

impl<'info> Make<'info> {
    // When we refactored ctx.accoutns -> self
    // ctx -> removed
    // self refers to the instance of the struct we're calling the fn against (Make struct)

    pub fn save(&mut self, seed: u64, receive_amount: u64, bumps: &MakeBumps) -> Result<()> { // how does the MakeBumps work??
        // save state
        self.escrow.set_inner(Escrow {
            seed,
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            receive_amount,
            bump: bumps.escrow,
            vault_bump: bumps.vault
        });

        Ok(())
    }

    // we decided to split the make into two functions
    pub fn deposit(&mut self, deposit_amount: u64) -> Result<()> {
        let accounts = Transfer {
            from: self.maker_ata_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info()
        };

        // the token accounts belogn to the token program so the token program
        // is the one we're gonna cpi into
        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer(cpi_ctx, deposit_amount)
    }
}