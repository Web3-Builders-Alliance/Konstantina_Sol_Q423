use anchor_lang::prelude::*;
// we don't need those anchor_spl::token imports anymore

pub mod contexts;
use contexts::*;

pub mod state;

declare_id!("9obik1Tntr8BDeoLj7KwnG7e6uQSDeqpX3s8WMeFUYeJ");

#[program]
pub mod anchor_escrow {
    use super::*;

    // seed to add some entropy to the ata so that we can have multiple
    pub fn make(ctx: Context<Make>, seed: u64, deposit_amount: u64, receive_amount: u64) -> Result<()> {
        ctx.accounts.deposit(deposit_amount)?;
        ctx.accounts.save(seed, receive_amount, &ctx.bumps)
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund()?;
        // We now are gonna close the vault too
        ctx.accounts.close_vault()
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw()?;
        ctx.accounts.close_vault()
    }
}
