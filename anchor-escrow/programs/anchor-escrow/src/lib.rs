use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, Transfer, transfer}, associated_token::AssociatedToken};

declare_id!("9obik1Tntr8BDeoLj7KwnG7e6uQSDeqpX3s8WMeFUYeJ");

#[program]
pub mod anchor_escrow {
    use super::*;

    // seed to add some entropy to the ata so that we can have multiple
    pub fn make(ctx: Context<Make>, seed: u64, deposit_amount: u64, receive_amount: u64) -> Result<()> {
        // save state
        ctx.accounts.escrow.set_inner(Escrow {
            seed,
            maker_token: ctx.accounts.mint_a.key(),
            taker_token: ctx.accounts.mint_b.key(),
            receive_amount,
            bump: ctx.bumps.escrow,
        });

        let accounts = Transfer {
            from: ctx.accounts.maker_ata_a.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.maker.to_account_info()
        };

        // the token accounts belogn to the token program so the token program
        // is the one we're gonna cpi into
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), accounts);

        transfer(cpi_ctx, deposit_amount)
    }

    // pub fn take(ctx: Context<Take>) -> Result<()> {
    //     Ok(())
    // }

    // pub fn refund(ctx: Context<Refund>) -> Result<()> {
    //     Ok(())
    // }
}

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make <'info>{
    #[account(mut)]
    maker: Signer<'info>, // signer

    mint_a: Account<'info, Mint>,
    mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer=maker,
        associated_token::mint = mint_a, // guaranteed to match mint_a
        associated_token::authority = maker
    )]
    maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer=maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    vault: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = Escrow::INIT_SPACE,
        payer = maker
    )]
    escrow: Account<'info, Escrow>,

    associated_token_program: Program<'info, AssociatedToken>, // needed because our escrow uses spl tokens
    token_program: Program<'info, Token>, // wouldn't be needed if we were just using sol
    system_program: Program<'info, System> // must have system program if we're initializing any account
}

#[account]
pub struct Escrow {
    seed: u64,
    maker_token: Pubkey,
    taker_token: Pubkey,
    receive_amount: u64,
    bump: u8,
}

impl Space for Escrow {
    const INIT_SPACE: usize = 32 + 8 + 32 + 1 + 8 + 8; // anchor adds a discriminator of 8 bytes
}