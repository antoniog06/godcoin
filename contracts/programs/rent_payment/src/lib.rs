use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("11111111111111111111111111111111"); // Replace with deployed program ID

#[program]
pub mod rent_payment {
    use super::*;

    /// Initialize a new property in the GODcoin community
    pub fn initialize_property(
        ctx: Context<InitializeProperty>,
        property_id: String,
        monthly_rent_usd: u64,
        total_units: u16,
    ) -> Result<()> {
        let property = &mut ctx.accounts.property;
        property.authority = ctx.accounts.authority.key();
        property.company_wallet = ctx.accounts.company_wallet.key();
        property.property_id = property_id;
        property.monthly_rent_usd = monthly_rent_usd;
        property.total_units = total_units;
        property.occupied_units = 0;
        property.is_active = true;
        property.created_at = Clock::get()?.unix_timestamp;
        property.bump = ctx.bumps.property;
        Ok(())
    }

    /// Pay rent in GOD tokens — transfers GOD from tenant to company wallet
    pub fn pay_rent(
        ctx: Context<PayRent>,
        amount_god: u64,
        amount_usd: u64,
        discount_applied: u16,
        discount_saved: u64,
        payment_month: u8,
        payment_year: u16,
    ) -> Result<()> {
        // Transfer GOD tokens from tenant to company wallet
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.tenant_token_account.to_account_info(),
                to: ctx.accounts.company_token_account.to_account_info(),
                authority: ctx.accounts.tenant.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, amount_god)?;

        // Record payment
        let payment = &mut ctx.accounts.payment_record;
        payment.tenant = ctx.accounts.tenant.key();
        payment.property_id = ctx.accounts.property.property_id.clone();
        payment.amount_god = amount_god;
        payment.amount_usd = amount_usd;
        payment.discount_applied = discount_applied;
        payment.discount_saved = discount_saved;
        payment.payment_month = payment_month;
        payment.payment_year = payment_year;
        payment.timestamp = Clock::get()?.unix_timestamp;
        payment.bump = ctx.bumps.payment_record;

        // Update tenant stats
        let tenant_account = &mut ctx.accounts.tenant_account;
        tenant_account.total_rent_paid = tenant_account
            .total_rent_paid
            .checked_add(amount_god)
            .unwrap();
        tenant_account.total_savings = tenant_account
            .total_savings
            .checked_add(discount_saved)
            .unwrap();

        emit!(RentPaid {
            tenant: ctx.accounts.tenant.key(),
            property_id: ctx.accounts.property.property_id.clone(),
            amount_god,
            amount_usd,
            discount_applied,
            month: payment_month,
            year: payment_year,
        });

        Ok(())
    }
}

// -- Accounts --

#[derive(Accounts)]
#[instruction(property_id: String)]
pub struct InitializeProperty<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PropertyAccount::SPACE,
        seeds = [b"property", property_id.as_bytes()],
        bump
    )]
    pub property: Account<'info, PropertyAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Company wallet to receive rent payments
    pub company_wallet: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(amount_god: u64, amount_usd: u64, discount_applied: u16, discount_saved: u64, payment_month: u8, payment_year: u16)]
pub struct PayRent<'info> {
    #[account(
        seeds = [b"property", property.property_id.as_bytes()],
        bump = property.bump,
    )]
    pub property: Account<'info, PropertyAccount>,

    #[account(mut)]
    pub tenant: Signer<'info>,

    #[account(
        mut,
        seeds = [b"tenant", tenant.key().as_ref(), property.key().as_ref()],
        bump = tenant_account.bump,
    )]
    pub tenant_account: Account<'info, TenantAccount>,

    #[account(
        init,
        payer = tenant,
        space = 8 + PaymentRecord::SPACE,
        seeds = [
            b"payment",
            tenant.key().as_ref(),
            &[payment_month],
            &payment_year.to_le_bytes(),
        ],
        bump
    )]
    pub payment_record: Account<'info, PaymentRecord>,

    #[account(mut)]
    pub tenant_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub company_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// -- State --

#[account]
pub struct PropertyAccount {
    pub authority: Pubkey,
    pub company_wallet: Pubkey,
    pub property_id: String,
    pub monthly_rent_usd: u64,
    pub total_units: u16,
    pub occupied_units: u16,
    pub is_active: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl PropertyAccount {
    pub const SPACE: usize = 32 + 32 + (4 + 64) + 8 + 2 + 2 + 1 + 8 + 1;
}

#[account]
pub struct TenantAccount {
    pub wallet: Pubkey,
    pub tier: u8,
    pub god_balance_snapshot: u64,
    pub join_date: i64,
    pub total_rent_paid: u64,
    pub total_savings: u64,
    pub is_active: bool,
    pub property: Pubkey,
    pub bump: u8,
}

impl TenantAccount {
    pub const SPACE: usize = 32 + 1 + 8 + 8 + 8 + 8 + 1 + 32 + 1;
}

#[account]
pub struct PaymentRecord {
    pub tenant: Pubkey,
    pub property_id: String,
    pub amount_god: u64,
    pub amount_usd: u64,
    pub discount_applied: u16,
    pub discount_saved: u64,
    pub payment_month: u8,
    pub payment_year: u16,
    pub timestamp: i64,
    pub bump: u8,
}

impl PaymentRecord {
    pub const SPACE: usize = 32 + (4 + 64) + 8 + 8 + 2 + 8 + 1 + 2 + 8 + 1;
}

// -- Events --

#[event]
pub struct RentPaid {
    pub tenant: Pubkey,
    pub property_id: String,
    pub amount_god: u64,
    pub amount_usd: u64,
    pub discount_applied: u16,
    pub month: u8,
    pub year: u16,
}
