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
        let piggy_bank = &ctx.accounts.piggy_bank;
        let signer = &ctx.accounts.signer;

        let current_balance = piggy_bank.to_account_info().lamports();
        // Dynamic rent check
        let rent = Rent::get()?;
        let min_balance = rent.minimum_balance(piggy_bank.to_account_info().data_len());

        if amount > current_balance {
            return Err(ProgramError::InsufficientFunds.into());
        }

        let remaining_balance = current_balance.checked_sub(amount).unwrap();

        // If withdrawal would leave less than rent-exempt minimum, close the account entirely
        if remaining_balance < min_balance {
            msg!(
                "Withdrawal leaves {} < rent {}. Closing account and transferring full balance.",
                remaining_balance,
                min_balance
            );
            **piggy_bank.to_account_info().try_borrow_mut_lamports()? = 0;
            **signer.to_account_info().try_borrow_mut_lamports()? = signer
                .lamports()
                .checked_add(current_balance)
                .ok_or(ErrorCode::Overflow)?;
        } else {
            msg!("Withdrawing funds {}", amount);
            **piggy_bank.to_account_info().try_borrow_mut_lamports()? -= amount;
            **signer.to_account_info().try_borrow_mut_lamports()? += amount;
        }

        Ok(())
    }
    pub fn reset(ctx: Context<Reset>) -> Result<()> {
        msg!("Resetting account...");
        let piggy_bank = &ctx.accounts.piggy_bank;
        let signer = &ctx.accounts.signer;

        // Close the account by transferring all lamports to the signer
        let dest_starting_lamports = signer.lamports();
        let src_lamports = piggy_bank.lamports();

        **piggy_bank.to_account_info().try_borrow_mut_lamports()? = 0;
        **signer.to_account_info().try_borrow_mut_lamports()? = dest_starting_lamports
            .checked_add(src_lamports)
            .ok_or(ErrorCode::Overflow)?;

        msg!("Account reset successful");
        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow.")]
    Overflow,
}

#[account()]
pub struct PiggyBank {
    pub owner: Pubkey,
    pub bump: u8,
}

#[derive(Accounts)]
pub struct Reset<'info> {
    /// CHECK: We are resetting this account, so we don't care about its data.
    #[account(mut, seeds=[signer.key().as_ref(), b"deposit"], bump)]
    pub piggy_bank: UncheckedAccount<'info>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
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
