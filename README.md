# GODCOIN (GOD)

Official GODCOIN project on Solana.

**Mint Address:** `HDaxFGHaJJi6mCMBVeZj8mwfiFpwE8snv6uEfuYabCcm`

---

## Token Metadata Setup

Follow these steps to get the GODCOIN name, symbol (GOD), and logo displaying on Jupiter, Raydium, Phantom, and other Solana platforms.

### Prerequisites

- Node.js 18+
- Your update authority keypair JSON file (the wallet that created the token)
- Your GODCOIN logo image (PNG recommended, 512x512px)

### Step 1: Install Dependencies

```bash
npm install
```

### Step 2: Place Your Logo

Copy your logo image into the assets folder:

```bash
cp /path/to/your/godcoin-logo.png token-metadata/assets/godcoin-logo.png
```

**Logo requirements:**
- Format: PNG (preferred), JPG, or SVG
- Size: 512x512 pixels recommended (minimum 256x256)
- Background: transparent or solid
- File size: under 1MB

### Step 3: Upload Logo & Metadata to Arweave

This uploads your logo and metadata JSON to permanent decentralized storage:

```bash
node scripts/upload-logo.js token-metadata/assets/godcoin-logo.png /path/to/your-keypair.json
```

This will print a **Metadata URI** — save it for the next step.

> **Note:** Uploading to Arweave costs a small amount of SOL (usually < 0.01 SOL). Make sure your wallet has some SOL for the upload fee.

### Step 4: Set On-Chain Metadata

Use the metadata URI from Step 3 to write the name, symbol, and logo to the blockchain:

```bash
node scripts/update-metadata.js <metadata-uri-from-step-3> /path/to/your-keypair.json
```

Example:
```bash
node scripts/update-metadata.js https://arweave.net/abc123xyz ~/my-wallet.json
```

### Step 5: Verify

After the transaction confirms, your token should display as **GODCOIN (GOD)** with logo on:
- Jupiter
- Raydium
- Phantom Wallet
- Solscan / Solana Explorer
- DEXScreener

Some platforms may take a few minutes to refresh their cache.

---

## Optional: Jupiter Verified Token List

For the verified checkmark on Jupiter, submit a PR to [Jupiter's Token List](https://github.com/jup-ag/token-list) with your token details after on-chain metadata is set.

---

## Project Structure

```
godcoin/
├── package.json                          # Dependencies
├── token-metadata/
│   ├── godcoin-metadata.json             # Off-chain metadata (name, symbol, image)
│   └── assets/                           # Logo files go here
├── scripts/
│   ├── upload-logo.js                    # Uploads logo + metadata to Arweave
│   └── update-metadata.js               # Sets on-chain Metaplex metadata
└── privacy-policy.html
```
