/**
 * Create or update on-chain Metaplex token metadata for GODCOIN.
 *
 * This sets the NAME, SYMBOL, and metadata URI (which contains the logo)
 * directly on-chain so that Jupiter, Raydium, Phantom, and other Solana
 * apps display your token branding correctly.
 *
 * Usage:
 *   node scripts/update-metadata.js <metadata-uri> <path-to-keypair.json>
 *
 * Example:
 *   node scripts/update-metadata.js https://arweave.net/abc123 ~/my-wallet.json
 */

const fs = require("fs");
const {
  createUmi,
} = require("@metaplex-foundation/umi-bundle-defaults");
const {
  keypairIdentity,
  publicKey,
} = require("@metaplex-foundation/umi");
const {
  createV1,
  updateV1,
  findMetadataPda,
  fetchMetadataFromSeeds,
  TokenStandard,
} = require("@metaplex-foundation/mpl-token-metadata");

// ── Configuration ──────────────────────────────────────────────────────
const MINT_ADDRESS = "HDaxFGHaJJi6mCMBVeZj8mwfiFpwE8snv6uEfuYabCcm";
const TOKEN_NAME = "GODCOIN";
const TOKEN_SYMBOL = "GOD";
// ────────────────────────────────────────────────────────────────────────

async function main() {
  const metadataUri = process.argv[2];
  const keypairPath = process.argv[3];

  if (!metadataUri || !keypairPath) {
    console.error(
      "Usage: node scripts/update-metadata.js <metadata-uri> <path-to-keypair.json>"
    );
    console.error(
      "  e.g. node scripts/update-metadata.js https://arweave.net/abc123 ~/my-wallet.json"
    );
    process.exit(1);
  }

  if (!fs.existsSync(keypairPath)) {
    console.error(`Keypair file not found: ${keypairPath}`);
    process.exit(1);
  }

  // Set up Umi with Solana mainnet
  const umi = createUmi("https://api.mainnet-beta.solana.com");

  // Load update authority keypair
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  const keypair = umi.eddsa.createKeypairFromSecretKey(
    new Uint8Array(keypairData)
  );
  umi.use(keypairIdentity(keypair));

  const mintPubkey = publicKey(MINT_ADDRESS);

  console.log("Token Mint:       ", MINT_ADDRESS);
  console.log("Update Authority: ", keypair.publicKey);
  console.log("Name:             ", TOKEN_NAME);
  console.log("Symbol:           ", TOKEN_SYMBOL);
  console.log("Metadata URI:     ", metadataUri);

  // Check if metadata account already exists
  let metadataExists = false;
  try {
    const existing = await fetchMetadataFromSeeds(umi, { mint: mintPubkey });
    metadataExists = true;
    console.log("\nExisting metadata found. Updating...");
    console.log("  Current name:  ", existing.name);
    console.log("  Current symbol:", existing.symbol);
    console.log("  Current URI:   ", existing.uri);
  } catch {
    console.log("\nNo existing metadata found. Creating new metadata...");
  }

  if (metadataExists) {
    // ── Update existing metadata ──
    await updateV1(umi, {
      mint: mintPubkey,
      data: {
        name: TOKEN_NAME,
        symbol: TOKEN_SYMBOL,
        uri: metadataUri,
        sellerFeeBasisPoints: 0,
        creators: null,
        collection: null,
        uses: null,
      },
    }).sendAndConfirm(umi);

    console.log("\nMetadata UPDATED successfully!");
  } else {
    // ── Create new metadata ──
    await createV1(umi, {
      mint: mintPubkey,
      name: TOKEN_NAME,
      symbol: TOKEN_SYMBOL,
      uri: metadataUri,
      sellerFeeBasisPoints: { basisPoints: 0n, identifier: "%", decimals: 2 },
      tokenStandard: TokenStandard.Fungible,
    }).sendAndConfirm(umi);

    console.log("\nMetadata CREATED successfully!");
  }

  // ── Verify ──
  console.log("\nVerifying on-chain metadata...");
  const final = await fetchMetadataFromSeeds(umi, { mint: mintPubkey });
  console.log("========================================");
  console.log("ON-CHAIN METADATA VERIFIED");
  console.log("========================================");
  console.log("Name:    ", final.name);
  console.log("Symbol:  ", final.symbol);
  console.log("URI:     ", final.uri);
  console.log("========================================");
  console.log(
    "\nYour token should now display as GODCOIN (GOD) with logo on:"
  );
  console.log("  - Jupiter");
  console.log("  - Raydium");
  console.log("  - Phantom Wallet");
  console.log("  - Solscan / Solana Explorer");
  console.log("  - DEXScreener");
  console.log("\nNote: Some platforms may take a few minutes to refresh.");
}

main().catch((err) => {
  console.error("Error:", err.message || err);
  process.exit(1);
});
