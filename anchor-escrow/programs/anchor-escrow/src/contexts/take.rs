use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, Transfer, transfer, CloseAccount, close_account}, associated_token::AssociatedToken};
use crate::state::Escrow;

#[derive(Accounts)]
pub struct Take <'info>{
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        init_if_needed, // the taker might not have an account for the receiving token mint_a already!
        payer = taker,
        associated_token::mint = mint_a,
        associated_token::authority = taker
    )]
    pub taker_ata_a: Account<'info, TokenAccount>,

    #[account(
        mut, // the taker must have a mint_b ata already or else they wouldn't be taking the escrow
        associated_token::mint = mint_b,
        associated_token::authority = taker
    )]
    pub taker_ata_b: Account<'info, TokenAccount>,

    #[account(
        init_if_needed, // the maker might not have an account for the receiving token mint_b already!
        payer=taker,
        associated_token::mint = mint_b,
        associated_token::authority = maker
    )]
    pub maker_ata_b: Account<'info, TokenAccount>,

    #[account(
        mut, // because we're gonna close the account
        // setting close = maker would be ok too
        close = taker, // we choose to give that to the taker to offset that they will likely have to pay for creating maker's ata for mint_b
        seeds = [b"escrow".as_ref(), maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
        // the has_one constraints are really important here!
        // someone malicious could deposit any other token insted of mint_b and take mint_a
        has_one = mint_a,
        has_one = mint_b
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        seeds = [b"vault", escrow.key().as_ref()],
        bump = escrow.vault_bump,
        token::mint = mint_a,
        token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System> // must have system program if we're initializing any account
}

impl<'info> Take<'info> {
    // Send money from taker to maker
    pub fn deposit(&mut self) -> Result<()> {
        let accounts = Transfer {
            from: self.taker_ata_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), accounts);

        transfer(cpi_ctx, self.escrow.receive_amount)
    }

    // Send money from vault to taker
    pub fn withdraw(&mut self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [
            &[
                b"escrow",
                self.maker.to_account_info().key.as_ref(),
                &self.escrow.seed.to_le_bytes()[..],
                &[self.escrow.bump]
            ]
        ];

        let accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        // ! CpiContext::new_with_signer - use when signing on behalf of a PDA
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds
        );

        transfer(cpi_ctx, self.vault.amount)
    }

    // Close the vault
    pub fn close_vault(&mut self) -> Result<()> {
        let signer_seeds: [&[&[u8]]; 1] = [
            &[
                b"escrow",
                self.maker.to_account_info().key.as_ref(),
                &self.escrow.seed.to_le_bytes()[..],
                &[self.escrow.bump]
            ]
        ];

        let close_accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(), // where to send the lamports from closing this account
            authority: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            &signer_seeds
        );

        close_account(cpi_ctx)
    }
}