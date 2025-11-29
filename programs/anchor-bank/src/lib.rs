use core::str;

use anchor_lang::prelude::*;

declare_id!("94msdEJKKMwvnQ8sdxSRBY1zczLhgPddTZZxw6JWs2zy");

#[program]
pub mod anchor_bank {

    use super::*;

    pub fn create(ctx: Context<Create>) -> Result<()> {
        ctx.accounts.piggy_bank.owner = ctx.accounts.signer.key();
        ctx.accounts.piggy_bank.bump = ctx.bumps.piggy_bank;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        msg!("Depositing {} lamports...", amount);

        // 1. Prepare the CPI Context (The "Letter" to the System Program)
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.signer.to_account_info(),
                to: ctx.accounts.piggy_bank.to_account_info(),
            },
        );

        let result = anchor_lang::system_program::transfer(cpi_context, amount);

        match result {
            Result::Err(_e) => {
                msg!("Deposit failed");
                Err(_e.into())
            }
            Result::Ok(_) => {
                msg!("Deposit of {} lamports successful", amount);
                msg!("Success!");
                Ok(())
            }
        }
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        if ctx.accounts.piggy_bank.to_account_info().lamports() - amount < 0.02 as u64 * 1_000_000_000
        {
            return Err(ProgramError::InsufficientFunds.into());
        }

        msg!("Withdrawing funds {}", amount);

        // let signer = ctx.accounts.signer.key();
        // let bump = ctx.accounts.piggy_bank.bump;
        // let signer_seeds = &[signer.as_ref(), b"deposit", &[bump]];
        // let signer_seeds = &[&signer_seeds[..]];

        // // &'a [&'b [&'c [u8]]]

        // let cpi_context = CpiContext::new_with_signer(
        //     ctx.accounts.system_program.to_account_info(),
        //     anchor_lang::system_program::Transfer {
        //         from: ctx.accounts.piggy_bank.to_account_info(),
        //         to: ctx.accounts.signer.to_account_info(),
        //     },
        //     signer_seeds,
        // );

        // let res = anchor_lang::system_program::transfer(cpi_context, amount);
        // match res {
        //     Result::Err(_e) => {
        //         msg!("Withdrawal failed, insufficient funds");
        //         return Err(_e.into());
        //     }
        //     Result::Ok(_) => {
        //         msg!("Withdrawal of {} lamports successful", amount);
        //         return Ok(());
        //     }
        // }

        **ctx
            .accounts
            .piggy_bank
            .to_account_info()
            .try_borrow_mut_lamports()? -= amount;

        **ctx.accounts.signer.to_account_info().try_borrow_mut_lamports()? +=
            amount;
        Ok(())
    }
}

#[account()]
pub struct PiggyBank {
    pub owner: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Create<'info> {
    #[account(init, seeds=[signer.key().as_ref(),b"deposit"], bump,payer=signer, space=8+32+1)]
    pub piggy_bank: Account<'info, PiggyBank>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [signer.key().as_ref(), b"deposit"], 
        bump = piggy_bank.bump
    )]
    pub piggy_bank: Account<'info, PiggyBank>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds=[signer.key().as_ref(),b"deposit"], 
        bump=piggy_bank.bump
    )]
    pub piggy_bank: Account<'info, PiggyBank>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
