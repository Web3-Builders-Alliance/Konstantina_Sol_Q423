use anchor_lang::{prelude::*, solana_program::hash::hash};

declare_id!("votezpw8ZFbkcXNXifYz7i86YjpoYvea6PNRetsstYH");
// anchor generated a public key for our program, we'll change it to a vanity address we created with sol-keygen grind for fun
// we then need to change it in:
// a) declare_id macro
// b) Anchor.toml
// c) add the private key in target/deploy (as long as this happens it will deploy to the right address)

#[program]
pub mod anchor_vote_solb {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, _url: String) -> Result<()> {
        ctx.accounts.vote.score = 0;
        // we save the bump
        ctx.accounts.vote.bump = ctx.bumps.vote;
        Ok(())
    }

    pub fn upvote(ctx: Context<Vote>, _url: String) -> Result<()> {
        ctx. accounts.vote.score += 1;
        Ok(())
    }

    pub fn downvote(ctx: Context<Vote>, _url: String) -> Result<()> {
        ctx. accounts.vote.score -= 1;
        Ok(())
    }
}

#[derive(Accounts)]
// instruction allows to take the inputs of your functions and use them in the account struct
#[instruction(_url:String)]
pub struct Initialize <'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = VoteState::INIT_SPACE,
        seeds = [hash(_url.as_bytes()).to_bytes().as_ref()], // because we want to use in seeds and a _url might be over the max length allowed
        bump
    )]
    vote: Account<'info, VoteState>,
    system_program: Program<'info, System> // we need the system program to init
}

#[derive(Accounts)]
// instruction allows to take the inputs of your functions and use them in the account struct
#[instruction(_url:String)]
pub struct Vote <'info>{
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut, // we don't need payer or space
        seeds = [hash(_url.as_bytes()).to_bytes().as_ref()],
        bump = vote.bump
    )]
    vote: Account<'info, VoteState>,
    system_program: Program<'info, System> // we need the system program to init
}

// there are procedural macros which run at complile time
#[account]
pub struct VoteState {
    score: i64,
    bump: u8
}

impl Space for VoteState {
    const INIT_SPACE: usize = 8 + 8 + 1;
    // score 8 + bump 1
    // anchor is gonna add an 8 byte discriminator to every account
    // gives unique prefix so that we don't have name collision
}
