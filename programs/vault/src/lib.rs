use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("5KonJwMy2VtkEnV6FBT2vpGTuGvc3CLofbA1RxxrSVUA");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Transact>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Transact>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        seeds = [b"state",user.key().as_ref()],
        bump,
        space = VaultState::INIT_SPACE +8
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"vault",vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: InitializeBumps) -> Result<()> {
        let rent_exempt = Rent::get()?.minimum_balance(0);
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, rent_exempt)?;
        self.vault_state.vault_bump = bumps.vault;
        self.vault_state.state_bump = bumps.vault_state;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Transact<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"vault",vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(
        seeds = [b"state",signer.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    pub system_program: Program<'info, System>,
}

impl<'info> Transact<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let balance = self.vault.get_lamports();
        let rent_exempt = Rent::get()?.minimum_balance(0);
        require_gte!(balance, rent_exempt + amount, VaultError::RentExemptError);

        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info(),
        };
        let seeds = &[
            b"vault".as_ref(),
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"state",signer.key().as_ref()],
        bump = vault_state.state_bump,
        close = signer
    )]
    pub vault_state: Account<'info, VaultState>,
    #[account(
        mut,
        seeds = [b"state",vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,
}

impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[account]
#[derive(InitSpace)] // here this InitSpace macro doesn't consider the solana descriminator
pub struct VaultState {
    pub state_bump: u8,
    pub vault_bump: u8,
}

#[error_code]
pub enum VaultError {
    #[msg("Balance unable to cover the Rent")]
    RentExemptError,
}