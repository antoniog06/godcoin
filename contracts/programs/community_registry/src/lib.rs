use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

declare_id!("11111111111111111111111111111111"); // Replace with deployed program ID

/// GOD token mint address
const GOD_MINT: &str = "HDaxFGHaJJi6mCMBVeZj8mwfiFpwE8snv6uEfuYabCcm";

/// Tier thresholds (in smallest GOD unit, assuming 9 decimals)
const BRONZE_THRESHOLD: u64 = 1_000_000_000_000;   // 1,000 GOD
const SILVER_THRESHOLD: u64 = 10_000_000_000_000;   // 10,000 GOD
const GOLD_THRESHOLD: u64 = 50_000_000_000_000;     // 50,000 GOD
const PLATINUM_THRESHOLD: u64 = 100_000_000_000_000; // 100,000 GOD

/// Discount basis points per tier (500 = 5%)
const BRONZE_DISCOUNT: u16 = 500;
const SILVER_DISCOUNT: u16 = 1000;
const GOLD_DISCOUNT: u16 = 1500;
const PLATINUM_DISCOUNT: u16 = 2000;

#[program]
pub mod community_registry {
    use super::*;

    /// Register a new community member
    pub fn register_member(ctx: Context<RegisterMember>) -> Result<()> {
        let member = &mut ctx.accounts.member;
        member.wallet = ctx.accounts.wallet.key();
        member.join_date = Clock::get()?.unix_timestamp;
        member.total_rent_paid = 0;
        member.total_savings = 0;
        member.is_active = true;
        member.bump = ctx.bumps.member;

        // Check initial tier based on GOD balance
        let balance = ctx.accounts.god_token_account.amount;
        member.tier = calculate_tier(balance);
        member.god_balance_snapshot = balance;

        // Update community stats
        let stats = &mut ctx.accounts.community_stats;
        stats.total_members += 1;
        update_tier_count(stats, member.tier, true);

        emit!(MemberRegistered {
            wallet: member.wallet,
            tier: member.tier,
            balance,
        });

        Ok(())
    }

    /// Verify and update a member's tier based on current GOD holdings
    pub fn update_tier(ctx: Context<UpdateTier>) -> Result<()> {
        let member = &mut ctx.accounts.member;
        let old_tier = member.tier;

        let balance = ctx.accounts.god_token_account.amount;
        let new_tier = calculate_tier(balance);

        member.tier = new_tier;
        member.god_balance_snapshot = balance;

        // Update community stats
        if old_tier != new_tier {
            let stats = &mut ctx.accounts.community_stats;
            update_tier_count(stats, old_tier, false);
            update_tier_count(stats, new_tier, true);

            emit!(TierChanged {
                wallet: member.wallet,
                old_tier,
                new_tier,
                balance,
            });
        }

        // Revoke membership if below minimum
        if balance < BRONZE_THRESHOLD {
            member.is_active = false;
            let stats = &mut ctx.accounts.community_stats;
            stats.total_members -= 1;
        }

        Ok(())
    }

    /// Get the discount percentage for a member's current tier
    pub fn get_discount(ctx: Context<GetDiscount>) -> Result<u16> {
        let member = &ctx.accounts.member;
        require!(member.is_active, ErrorCode::MemberInactive);

        let discount = match member.tier {
            1 => BRONZE_DISCOUNT,
            2 => SILVER_DISCOUNT,
            3 => GOLD_DISCOUNT,
            4 => PLATINUM_DISCOUNT,
            _ => 0,
        };

        Ok(discount)
    }
}

// -- Helper Functions --

fn calculate_tier(balance: u64) -> u8 {
    if balance >= PLATINUM_THRESHOLD {
        4 // Platinum
    } else if balance >= GOLD_THRESHOLD {
        3 // Gold
    } else if balance >= SILVER_THRESHOLD {
        2 // Silver
    } else if balance >= BRONZE_THRESHOLD {
        1 // Bronze
    } else {
        0 // None
    }
}

fn update_tier_count(stats: &mut CommunityStats, tier: u8, increment: bool) {
    let count = match tier {
        1 => &mut stats.bronze_count,
        2 => &mut stats.silver_count,
        3 => &mut stats.gold_count,
        4 => &mut stats.platinum_count,
        _ => return,
    };
    if increment {
        *count += 1;
    } else if *count > 0 {
        *count -= 1;
    }
}

// -- Accounts --

#[derive(Accounts)]
pub struct RegisterMember<'info> {
    #[account(
        init,
        payer = wallet,
        space = 8 + MemberAccount::SPACE,
        seeds = [b"member", wallet.key().as_ref()],
        bump
    )]
    pub member: Account<'info, MemberAccount>,

    #[account(mut)]
    pub wallet: Signer<'info>,

    /// The member's GOD token account (to verify balance)
    pub god_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"community_stats"],
        bump = community_stats.bump,
    )]
    pub community_stats: Account<'info, CommunityStats>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateTier<'info> {
    #[account(
        mut,
        seeds = [b"member", member.wallet.as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, MemberAccount>,

    /// The member's GOD token account (to verify current balance)
    pub god_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"community_stats"],
        bump = community_stats.bump,
    )]
    pub community_stats: Account<'info, CommunityStats>,
}

#[derive(Accounts)]
pub struct GetDiscount<'info> {
    #[account(
        seeds = [b"member", member.wallet.as_ref()],
        bump = member.bump,
    )]
    pub member: Account<'info, MemberAccount>,
}

// -- State --

#[account]
pub struct MemberAccount {
    pub wallet: Pubkey,
    pub tier: u8,               // 0=None, 1=Bronze, 2=Silver, 3=Gold, 4=Platinum
    pub god_balance_snapshot: u64,
    pub join_date: i64,
    pub total_rent_paid: u64,
    pub total_savings: u64,
    pub is_active: bool,
    pub bump: u8,
}

impl MemberAccount {
    pub const SPACE: usize = 32 + 1 + 8 + 8 + 8 + 8 + 1 + 1;
}

#[account]
pub struct CommunityStats {
    pub total_members: u64,
    pub bronze_count: u64,
    pub silver_count: u64,
    pub gold_count: u64,
    pub platinum_count: u64,
    pub total_rent_collected: u64,
    pub total_properties: u16,
    pub bump: u8,
}

impl CommunityStats {
    pub const SPACE: usize = 8 + 8 + 8 + 8 + 8 + 8 + 2 + 1;
}

// -- Events --

#[event]
pub struct MemberRegistered {
    pub wallet: Pubkey,
    pub tier: u8,
    pub balance: u64,
}

#[event]
pub struct TierChanged {
    pub wallet: Pubkey,
    pub old_tier: u8,
    pub new_tier: u8,
    pub balance: u64,
}

// -- Errors --

#[error_code]
pub enum ErrorCode {
    #[msg("Member account is inactive. Minimum 1,000 GOD required.")]
    MemberInactive,
    #[msg("Insufficient GOD balance for this tier.")]
    InsufficientBalance,
}
