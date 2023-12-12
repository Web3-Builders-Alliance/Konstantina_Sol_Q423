use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, TokenAccount, Token, Transfer, transfer, close_account, CloseAccount}, associated_token::AssociatedToken};
use crate::state::Escrow;

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

impl<'info> Refund<'info> {
    pub fn refund(&mut self) -> Result<()> {
        // we don't need seed because it's saved
        // we don't need deposit or receive amounts because we can find what amount is in the vault

        // refund: vault -> maker
        let accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        // let binding = self.escrow.seed.to_le_bytes();

        let signer_seeds: [&[&[u8]]; 1] = [
            &[
                b"escrow",
                self.maker.to_account_info().key.as_ref(),
                &self.escrow.seed.to_le_bytes()[..],
                &[self.escrow.bump]
            ]
        ];

        // ! CpiContext::new_with_signer - use when signing on behalf of a PDA
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            &signer_seeds
        );

        transfer(cpi_ctx, self.vault.amount)
    }

    pub fn close_vault(&mut self) -> Result<()> {
        // same seeds
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
            destination: self.maker.to_account_info(), // where to send the lamports from closing this account
            authority: self.escrow.to_account_info()
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            close_accounts,
            &signer_seeds // same signer seeds because same signer
        );

        close_account(cpi_ctx)
    }
}