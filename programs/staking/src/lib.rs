use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("Ewc2vCCi2zEnQCe62cz4vuob5dcU9XjnEnPYcTnkPbPy");

#[program]
pub mod staking {
    use anchor_lang::solana_program::{clock, example_mocks::solana_sdk::system_program};

    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;
        pda_account.owner = ctx.accounts.payer.key();
        pda_account.staked_amount = 0;
        pda_account.total_points = 0;
        pda_account.last_update_time = clock.unix_timestamp;
        pda_account.bump = ctx.bump.pda_account;
        
        Ok(());
    }
    pub fn Stake( ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        //updating points before changing stake amount
        update_points(pda_account, clock.unix_timestamp)?;

        //transfer SOl from user to pda account 
        let cpi = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.pda_account.to_account_info()
            },
        );
        system_program::transfer(cpi, amount)?;
        
        // Update staked amount
        pda_account.staked_amount = pda_account.staked_amount.checked_add(amount)
            .ok_or(StakeError::Overflow)?;
        
        msg!("Staked {} lamports. Total staked: {}, Total points: {}", 
             amount, pda_account.staked_amount, pda_account.total_points / 1_000_000);
        Ok(())
    }
    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()>{
    require!(amount > 0, StakeError::InvalidAmount);
    
    let pda_account = &mut ctx.accounts.pda_account;
    let clock = Clock::get()?;
    require!(pda_account.staked_amount >= amount, StakeError::InsufficientStake);

     // update points before changing staked amount
    update_points(pda_account, clock.unix_timestamp)?;
    
    //transfer SOL from pda to user account
    let seeds = &[
        b"client1",
        ctx.accounts.user.key().as_ref(),
        &[pda_account.bump],
    ];
    let signer = &[&seeds[..]];
    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.pda_account.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        },
        signer,
    );
     system_program::transfer(cpi_context, amount)?;
     // Update staked amount
    pda_account.staked_amount = pda_account.staked_amount.checked_sub(amount)
    .ok_or(StakeError::Underflow)?;
      
    msg!("Unstaked {} lamports. Remaining staked: {}, Total points: {}", 
    amount, pda_account.staked_amount, pda_account.total_points / 1_000_000);
    Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePdaAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
#[account(
    init,
    payer = user,
    space = 8 + 32 + 8 + 8 + 1,
    seeds = [b"client1", user.key().as_ref()],
    bump,
)]
pub pda_account: Account<'info, StakeAccount>,
#[account(mut)]
pub user: Signer<'info>,
pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
     mut,
     seeds = [b"client1", user.key().as_ref()],
     bump = pda_account.bump,
     constraint = pda_account.owner == user.key() @StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
    
    pub system_program: Program<'info, System>,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub total_points: u64,
    pub last_update_time: i64,
    pub bump: u8,
}


#[error_code]
pub enum StakeError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}