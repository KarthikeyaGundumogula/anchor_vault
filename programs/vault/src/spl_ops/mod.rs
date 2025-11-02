use anchor_lang::{
    prelude::*
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface,TransferChecked,transfer_checked},
};

#[derive(Accounts)]
pub struct Operate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mint::token_program = token_program)]
    pub mint_acc: InterfaceAccount<'info, Mint>,
    #[account(
      mut,
      associated_token::mint = mint_acc,
      associated_token::authority = signer,
      associated_token::token_program = token_program,
    )]
    pub signer_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
      init_if_needed,
      payer = signer,
      associated_token::mint = mint_acc,
      associated_token::authority = signer,
      associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Operate<'info> {

    pub fn lock(&mut self, deposit_amount: u64) -> Result<()> {
        let transfer_accounts = TransferChecked{
          from: self.signer_ata.to_account_info(),
          mint:self.mint_acc.to_account_info(),
          to:self.vault.to_account_info(),
          authority: self.signer.to_account_info(),
        };

        let transfer_program = self.token_program.to_account_info();
        let transfer_ctx = CpiContext::new(transfer_program, transfer_accounts);
        transfer_checked(transfer_ctx, deposit_amount, self.mint_acc.decimals)        
    }

    pub fn unlock(&mut self, withdraw_amount: u64) -> Result<()> {
        let transfer_accounts = TransferChecked{
          from: self.vault.to_account_info(),
          mint:self.mint_acc.to_account_info(),
          to:self.signer_ata.to_account_info(),
          authority: self.signer.to_account_info(),
        };

        let transfer_program = self.token_program.to_account_info();
        let transfer_ctx = CpiContext::new(transfer_program, transfer_accounts);
        transfer_checked(transfer_ctx, withdraw_amount, self.mint_acc.decimals)        
    }
}
