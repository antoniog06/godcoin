# GODcoin Smart Contracts — Architecture

## Overview

These Solana programs (smart contracts) power the GODcoin community ecosystem: rent payments, holder discounts, membership verification, and community governance.

Built with the **Anchor framework** on Solana.

---

## Contract Architecture

```
┌──────────────────────────────────────────────────────┐
│                  GODcoin Ecosystem                    │
├──────────────────────────────────────────────────────┤
│                                                      │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │  Rent        │  │  Community   │  │  Discount  │  │
│  │  Payment     │  │  Registry    │  │  Engine    │  │
│  │  Program     │  │  Program     │  │  Program   │  │
│  └──────┬──────┘  └──────┬───────┘  └─────┬──────┘  │
│         │                │                 │         │
│         └────────────────┼─────────────────┘         │
│                          │                           │
│                   ┌──────┴───────┐                   │
│                   │   GOD Token  │                   │
│                   │   (SPL)      │                   │
│                   └──────────────┘                   │
│                                                      │
└──────────────────────────────────────────────────────┘
```

## Programs

### 1. Rent Payment Program (`rent_payment`)
Handles rent collection in GOD tokens.

**Instructions:**
- `initialize_property` — Register a new property with monthly rent amount
- `pay_rent` — Tenant sends GOD to company wallet as rent payment
- `record_payment` — Log payment on-chain with timestamp and amount
- `get_payment_history` — Query payment records for a tenant

**Accounts:**
- `PropertyAccount` — Stores property details, rent amount, company wallet
- `TenantAccount` — Tracks tenant payments, balance, status
- `PaymentRecord` — Individual payment receipt (PDA)

### 2. Community Registry Program (`community_registry`)
Manages community membership and tier verification.

**Instructions:**
- `register_member` — Register a wallet as community member
- `verify_tier` — Check GOD balance and assign tier (Bronze/Silver/Gold/Platinum)
- `update_tier` — Monthly tier recalculation based on holdings
- `revoke_membership` — Remove member who falls below minimum

**Accounts:**
- `MemberAccount` — Stores member wallet, tier, join date, status
- `CommunityStats` — Global stats (total members, tier distribution)

**Tier Thresholds:**
```
Bronze:   1,000 GOD   → 5% discount
Silver:   10,000 GOD  → 10% discount
Gold:     50,000 GOD  → 15% discount
Platinum: 100,000 GOD → 20% discount
```

### 3. Discount Engine Program (`discount_engine`)
Calculates and applies rent discounts based on membership tier.

**Instructions:**
- `calculate_discount` — Compute discounted rent based on tier
- `apply_discount` — Process rent payment with discount applied
- `get_savings` — Query total savings for a member

**Logic:**
```
rent_in_god = market_rent_usd / god_price_usd
discount = tier_discount_percentage
final_rent = rent_in_god * (1 - discount)
```

---

## Data Flow: Paying Rent

```
1. Tenant holds GOD in their wallet
         │
2. Tenant calls pay_rent instruction
         │
3. Community Registry verifies membership tier
         │
4. Discount Engine calculates final amount
         │
5. GOD transferred from tenant → Company Wallet
         │
6. Payment recorded on-chain (PaymentRecord PDA)
         │
7. Tenant receives confirmation + updated payment history
```

---

## Account Structures

### PropertyAccount
```rust
#[account]
pub struct PropertyAccount {
    pub authority: Pubkey,          // Company/owner
    pub company_wallet: Pubkey,     // Where rent GOD goes
    pub property_id: String,        // Unique property identifier
    pub monthly_rent_usd: u64,      // Rent in USD cents
    pub total_units: u16,           // Number of units
    pub occupied_units: u16,        // Currently occupied
    pub is_active: bool,
    pub created_at: i64,
    pub bump: u8,
}
```

### MemberAccount
```rust
#[account]
pub struct MemberAccount {
    pub wallet: Pubkey,             // Member's wallet
    pub tier: MemberTier,           // Bronze/Silver/Gold/Platinum
    pub god_balance_snapshot: u64,  // Last verified balance
    pub join_date: i64,
    pub total_rent_paid: u64,       // Lifetime rent paid in GOD
    pub total_savings: u64,         // Lifetime discount savings
    pub is_active: bool,
    pub property_id: Option<String>,// Current residence
    pub bump: u8,
}
```

### PaymentRecord
```rust
#[account]
pub struct PaymentRecord {
    pub tenant: Pubkey,
    pub property_id: String,
    pub amount_god: u64,            // GOD amount paid
    pub amount_usd: u64,            // Equivalent USD value
    pub discount_applied: u16,      // Discount % (500 = 5.00%)
    pub discount_saved: u64,        // GOD saved from discount
    pub payment_month: u8,
    pub payment_year: u16,
    pub timestamp: i64,
    pub tx_signature: String,
    pub bump: u8,
}
```

### MemberTier Enum
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum MemberTier {
    None,       // < 1,000 GOD
    Bronze,     // 1,000+ GOD
    Silver,     // 10,000+ GOD
    Gold,       // 50,000+ GOD
    Platinum,   // 100,000+ GOD
}
```

---

## Development Setup

### Prerequisites
- Rust 1.70+
- Solana CLI 1.17+
- Anchor 0.29+
- Node.js 18+

### Build
```bash
anchor build
```

### Test
```bash
anchor test
```

### Deploy
```bash
anchor deploy --provider.cluster mainnet
```

---

## Security Considerations

- All payment instructions require tenant signature
- Company wallet is set at initialization and can only be updated by authority
- Tier verification reads live token balance, not cached values
- Payment records are immutable PDAs (cannot be altered after creation)
- Discount calculations use on-chain price oracle data
