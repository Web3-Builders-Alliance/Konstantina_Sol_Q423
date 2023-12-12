use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, Transfer, transfer, CloseAccount, close_account}, associated_token::AssociatedToken};

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
            vault_bump: ctx.bumps.vault
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

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        // we don't need seed because it's saved
        // we don't need deposit or receive amounts because we can find what amount is in the vault

        // refund: vault -> maker
        let accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.maker_ata_a.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info()
        };

        let binding = ctx.accounts.escrow.seed.to_le_bytes();

        let signer_seeds = [
            &[
                b"escrow",
                ctx.accounts.maker.to_account_info().key.as_ref(),
                binding.as_ref()
            ][..]
        ];

        // ! CpiContext::new_with_signer - use when signing on behalf of a PDA
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            accounts,
            &signer_seeds
        );

        transfer(cpi_ctx, ctx.accounts.vault.amount)?;

        // We now are gonna close the vault too
        let close_accounts = CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.maker.to_account_info(), // where to send the lamports from closing this account
            authority: ctx.accounts.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer( // example of shadowing
            ctx.accounts.token_program.to_account_info(),
            close_accounts,
            &signer_seeds // same signer seeds because same signer
        );

        close_account(cpi_ctx)
    }
}

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make <'info>{
    #[account(mut)]
    maker: Signer<'info>, // signer

    mint_a: Account<'info, Mint>,
    mint_b: Account<'info, Mint>,

    #[account(
        // init, ... maker_ata should already exist if thety're making
        // payer=maker,
        mut,
        associated_token::mint = mint_a, // guaranteed to match mint_a
        associated_token::authority = maker
    )]
    maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        // init_if_needed, // although normally it shouldn't exist
        init,
        payer=maker,
        seeds = [b"vault", escrow.key().as_ref()], // now we don't have to put the seed in
        // by using the escrow key we're inheriting the entropy of the seed
        bump,
        token::mint = mint_a, // why not an associated token here?
        token::authority = escrow, // why not an associated token here?
        // we would even set the authority to vault
    )]
    vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = Escrow::INIT_SPACE,
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    escrow: Account<'info, Escrow>, // an account storing our escrow details

    associated_token_program: Program<'info, AssociatedToken>, // needed because our escrow uses spl tokens
    token_program: Program<'info, Token>, // wouldn't be needed if we were just using sol
    system_program: Program<'info, System> // must have system program if we're initializing any account
}

#[derive(Accounts)]
pub struct Refund <'info>{
    #[account(mut)]
    maker: Signer<'info>, // signer

    mint_a: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker
    )]
    maker_ata_a: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"vault", escrow.key().as_ref()],
        bump = escrow.vault_bump,
        token::mint = mint_a,
        token::authority = escrow
    )]
    vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        close = maker, // Marks the account as closed at the end of the instruction’s execution and sends its lamports to the specified account
        // we still need to provide the seed constraints even though the account is initialized
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()], // now we didn't pass seed by instruction but it's saved in the Escrow struct
        bump = escrow.bump,
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
    vault_bump: u8 // what if we don't add this -> we could but we're saving compute
}

impl Space for Escrow {
    const INIT_SPACE: usize = 32 + 8 + 32 + 1 + 1 + 8 + 8; // anchor adds a discriminator of 8 bytes
}