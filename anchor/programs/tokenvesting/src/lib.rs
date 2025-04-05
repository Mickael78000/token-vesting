#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TransferChecked};
use anchor_spl::{
    //associated_token::AssociatedToken,
    token_interface::{TokenInterface, TokenAccount}  // Key interface
};



declare_id!("CRU6xLpQ4Nnte84KMuPvENoXy29ZwUnM2XcwhZtT6kXt");
#[program]
pub mod tokenvesting {
    use super::*;
    
    pub fn create_vesting_account(
        ctx: Context<CreateVestingAccount>,
        company: String,
        
     
    ) -> Result<()> {
        let me = 32;
        require!(&company.len() <= &me, ErrorCode::NameTooLong);
        
        *ctx.accounts.tokenvesting_account = TokenvestingAccount {
            owner: ctx.accounts.signer.key(),
            mint: ctx.accounts.mint.key(),
            treasury_token_account: ctx.accounts.treasury_token_account.key(),
            company: company.clone(),
            treasury_bump: ctx.bumps.treasury_token_account,
            bump: ctx.bumps.tokenvesting_account,
        };
     
        msg!("Received company name: {}", &company);
        msg!("Company name length: {}", &company.len());
   
        Ok(())
    }

    pub fn create_employee_account(
        ctx: Context<CreateEmployeeAccount>,
        start_time: i64,
        end_time: i64,
        total_amount: u64,
        cliff_time: i64,
        company: String,
        
     
    ) -> Result<()> {
        
        require!(company.len() <= 32, ErrorCode::NameTooLong);
        *ctx.accounts.employee_account = EmployeeAccount {
            beneficiary: ctx.accounts.beneficiary.key(),
            start_time,
            end_time,
            cliff_time,
            tokenvesting_account: ctx.accounts.tokenvesting_account.key(),
            total_amount,
            total_withdrawn: 0,
            bump: ctx.bumps.employee_account,
        };
        Ok(())
    }

    pub fn claim_tokens(
        ctx: Context<ClaimTokens>,
        _company: String,
        
    ) -> Result<()> {
        let employee_account = &mut ctx.accounts.employee_account;
        let now = Clock::get()?.unix_timestamp;

        if now < employee_account.cliff_time {
            return Err(ErrorCode::ClaimNotAvailableYet.into());
        }
        let time_since_start = now.saturating_sub(employee_account.start_time);
        let total_tokenvesting_time = employee_account.end_time.saturating_sub(employee_account.start_time);
        
        if total_tokenvesting_time == 0 {
            return Err(ErrorCode::InvalidVestingPeriod.into());
        }

        let vested_amount = if now >= employee_account.end_time {
            employee_account.total_amount
            } else {
                match employee_account.total_amount.checked_mul(time_since_start as u64) {
                    Some(product) => 
                        product / (total_tokenvesting_time as u64)
                    ,
                    None => {
                        return Err(ErrorCode::CalculationOverflow.into());
                    }
                }
            };

            let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
            if claimable_amount == 0 {
                return Err(ErrorCode::NothingToClaim.into());
            };

            let transfer_cpi_accounts = TransferChecked{
                from: ctx.accounts.treasury_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.employee_token_account.to_account_info(),
                authority: ctx.accounts.tokenvesting_account.to_account_info(), // PDA authority
                
            };

            let cpi_program = ctx.accounts.token_program.to_account_info();

            let signer_seeds: &[&[&[u8]]] = &[
                 &[b"tokenvesting_treasury",b"company",
                 &[ctx.accounts.tokenvesting_account.treasury_bump],],
            ];
          

            let cpi_context = CpiContext::new(
                cpi_program,
                transfer_cpi_accounts,
            ).with_signer(signer_seeds);
            
            let decimals = ctx.accounts.mint.decimals;

            anchor_spl::token_interface::transfer_checked(
                cpi_context,
                claimable_amount as u64,
                decimals,
            )?;

            employee_account.total_withdrawn += claimable_amount;
        
        Ok(())
    }
    // pub fn claim_tokens(
    //     ctx: Context<ClaimTokens>,
    //     company: String,
    // ) -> Result<()> {
        
    //     require!(company.len() <= 32, ErrorCode::NameTooLong);
    //     let employee_account = &mut ctx.accounts.employee_account;
    //     let now = Clock::get()?.unix_timestamp;
    
    //     // 1. Time validation
    //     require!(now >= employee_account.cliff_time, ErrorCode::ClaimNotAvailableYet);
    //     require!(employee_account.end_time > employee_account.start_time, ErrorCode::InvalidVestingPeriod);
    
    //     // 2. Calculate vested amount
    //     let vested_amount = if now >= employee_account.end_time {
    //         employee_account.total_amount
    //     } else {
    //         let time_elapsed = now.saturating_sub(employee_account.start_time) as u64;
    //         let total_duration = (employee_account.end_time - employee_account.start_time) as u64;
            
    //         employee_account.total_amount
    //             .checked_mul(time_elapsed)
    //             .ok_or(ErrorCode::CalculationOverflow)?
    //             .checked_div(total_duration)
    //             .ok_or(ErrorCode::CalculationOverflow)?
    //     };
    
    //     // 3. Calculate claimable amount
    //     let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);
    //     require!(claimable_amount > 0, ErrorCode::NothingToClaim);
    
    //     // 4. Prepare token transfer
    //     let binding: &[&[&[u8]]] = &[
    //         &[
    //             b"tokenvesting_treasury",
    //             ctx.accounts.tokenvesting_account.company.as_bytes(),
    //             &[ctx.accounts.tokenvesting_account.treasury_bump]
    //         ]
    //     ];
        
    //     let transfer_ctx = CpiContext::new(
    //         ctx.accounts.token_program.to_account_info(),
    //         TransferChecked {
    //             from: ctx.accounts.treasury_token_account.to_account_info(),
    //             mint: ctx.accounts.mint.to_account_info(),
    //             to: ctx.accounts.employee_token_account.to_account_info(),
    //             authority: ctx.accounts.tokenvesting_account.to_account_info(), // PDA authority
    //         },
    //     ).with_signer(&binding);
    
    //     // 5. Execute transfer with decimals handling
    //     let decimals = ctx.accounts.mint.decimals;
    //     token_interface::transfer_checked(
    //         transfer_ctx,
    //         claimable_amount,
    //         decimals,
    //     )?;
    
    //     // 6. Update state
    //     employee_account.total_withdrawn = employee_account.total_withdrawn
    //         .checked_add(claimable_amount)
    //         .ok_or(ErrorCode::CalculationOverflow)?;
        
    //     msg!("Deriving PDA with seeds:");
    //     msg!("- Prefix: tokenvesting_treasury");
    //     msg!("- Company: {}", company);
    //     msg!("- Bump: {}", ctx.accounts.tokenvesting_account.treasury_bump);
    
    //     Ok(())
    // }
        
    


}

#[derive(Accounts)]
//#[instruction(company: String)]
pub struct CreateEmployeeAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub beneficiary: SystemAccount<'info>,

    #[account(
        has_one = owner
    )]
    pub tokenvesting_account: Account<'info, TokenvestingAccount>,
    
    #[account(
        mut,
        seeds = [b"tokenvesting_treasury",b"company"],
        bump,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    
     
    #[account(
        init,
        payer = owner,
        space = 8 + EmployeeAccount::INIT_SPACE,
        seeds = [b"employee_tokenvesting",beneficiary.key().as_ref(), tokenvesting_account.key().as_ref()],
        bump,
    )]
    pub employee_account: Account<'info, EmployeeAccount>,
    
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(company: String)]
pub struct ClaimTokens<'info> {
    /// CHECK: PDA validated through signature derivation
    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(
        mut,
        token::authority = tokenvesting_account, 
        seeds = [b"tokenvesting_treasury",b"company"],
        bump = tokenvesting_account.treasury_bump,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"company"],
        bump = tokenvesting_account.bump,
        has_one = treasury_token_account,
        has_one = mint,
    )]
    pub tokenvesting_account: Account<'info, TokenvestingAccount>,
    
    #[account(
        mut,
        seeds = [b"employee_tokenvesting",beneficiary.key().as_ref(),tokenvesting_account.key().as_ref()],
        bump = employee_account.bump,
        has_one = beneficiary,
        has_one = tokenvesting_account,
    )]
    pub employee_account: Account<'info, EmployeeAccount>,
    
    pub mint: InterfaceAccount<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = beneficiary,
        token::mint = mint,
        token::authority = beneficiary,
        token::token_program = token_program,
    )]
    pub employee_token_account: InterfaceAccount<'info, TokenAccount>,
    
    pub token_program: Interface<'info, TokenInterface>,

   // pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}




#[derive(Accounts)]
#[instruction(company: String)]
pub struct CreateVestingAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init, 
        payer = signer, 
        space = 8 + TokenvestingAccount::INIT_SPACE,
        seeds = [company.as_bytes()],
        bump,
    )]
    pub tokenvesting_account: Account<'info, TokenvestingAccount>,
    
    pub mint: InterfaceAccount<'info, Mint>,     
   
    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = tokenvesting_account, 
        seeds = [b"tokenvesting_treasury",company.as_bytes()],
        bump,
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>, 
    
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
       
}

#[account]
#[derive(InitSpace)]
pub struct TokenvestingAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub treasury_token_account: Pubkey,
    #[max_len(32)]
    pub company: String,
    pub treasury_bump: u8,
    pub bump: u8,

}

#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount {
    pub beneficiary: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub cliff_time: i64,
    pub tokenvesting_account: Pubkey,
    pub total_amount: u64,
    pub total_withdrawn: u64,
    pub bump: u8,

}

#[error_code]
pub enum ErrorCode {
    #[msg("Claim not available yet")]
    ClaimNotAvailableYet,
    #[msg("Invalid vesting period")]
    InvalidVestingPeriod,
    #[msg("Calculation overflow")]
    CalculationOverflow,
    #[msg("Nothing to claim")]
    NothingToClaim,
    #[msg("Company name exceeds maximum allowed length")]
    NameTooLong,
    
  
}



