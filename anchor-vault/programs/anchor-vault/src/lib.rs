use anchor_lang::prelude::*;
use anchor_lang::system_program::{Transfer, transfer};

declare_id!("9ri4ddvn5PVouDM1eX4KhCo4a3SAcrNKyKgunKce43Gm");

#[program]
pub mod anchor_vault {
    use super::*;

    pub fn deposit(ctx: Context<Vault>, lamports: u64) -> Result<()> {
        let accounts = Transfer {
            from: ctx.accounts.signer.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };

        let transfer_ctx = CpiContext::new(ctx.accounts.system_progam.to_account_info() , accounts);

        transfer(transfer_ctx, lamports)
    }

    pub fn withdraw(ctx: Context<Vault>, lamports: u64) -> Result<()> {
        // we could remove lamports and have withdraw be a function that empties and closes the vault
        let accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.signer.to_account_info(),
        };

        let seeds = &[
            b"vault",
            ctx.accounts.signer.to_account_info().key.as_ref(),
            &[ctx.bumps.vault]
            ];

        let signer_seeds = &[&seeds[..]];

        let transfer_ctx = CpiContext::new_with_signer( // bc pda
            ctx.accounts.system_progam.to_account_info(),
            accounts,
            signer_seeds
        );

        transfer(transfer_ctx, lamports)
    }

}


#[derive(Accounts)]
pub struct Vault <'info>{
    #[account(mut)] // someone needs to sign for the vault
    signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault", signer.key().as_ref()],
        bump
    )]
    vault: SystemAccount<'info>,
    system_progam: Program<'info, System>
}
