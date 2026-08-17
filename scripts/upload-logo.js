/**
 * Upload GODCOIN logo and metadata JSON to Arweave via Irys (formerly Bundlr).
 *
 * Usage:
 *   node scripts/upload-logo.js <path-to-logo.png> <path-to-keypair.json>
 *
 * Example:
 *   node scripts/upload-logo.js ./token-metadata/assets/godcoin-logo.png ~/my-wallet.json
 *
 * This script will:
 *   1. Upload the logo image to Arweave
 *   2. Update the metadata JSON with the image URI
 *   3. Upload the metadata JSON to Arweave
 *   4. Print the final metadata URI (needed for on-chain update)
 */

const fs = require("fs");
const path = require("path");
const { createUmi } = require("@metaplex-foundation/umi-bundle-defaults");
const { irysUploader } = require("@metaplex-foundation/umi-uploader-irys");
const { createGenericFile } = require("@metaplex-foundation/umi");
const {
  keypairIdentity,
} = require("@metaplex-foundation/umi");

async function main() {
  const logoPath = process.argv[2];
  const keypairPath = process.argv[3];

  if (!logoPath || !keypairPath) {
    console.error(
      "Usage: node scripts/upload-logo.js <path-to-logo.png> <path-to-keypair.json>"
    );
    process.exit(1);
  }

  // Validate files exist
  if (!fs.existsSync(logoPath)) {
    console.error(`Logo file not found: ${logoPath}`);
    process.exit(1);
  }
  if (!fs.existsSync(keypairPath)) {
    console.error(`Keypair file not found: ${keypairPath}`);
    process.exit(1);
  }

  // Set up Umi with Solana mainnet
  const umi = createUmi("https://api.mainnet-beta.solana.com");

  // Load keypair
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  const keypair = umi.eddsa.createKeypairFromSecretKey(
    new Uint8Array(keypairData)
  );
  umi.use(keypairIdentity(keypair));
  umi.use(irysUploader());

  console.log("Wallet public key:", keypair.publicKey);

  // --- Step 1: Upload the logo image ---
  console.log("\n[1/3] Uploading logo image to Arweave...");
  const logoBuffer = fs.readFileSync(logoPath);
  const logoExtension = path.extname(logoPath).toLowerCase();
  const mimeTypes = {
    ".png": "image/png",
    ".jpg": "image/jpeg",
    ".jpeg": "image/jpeg",
    ".svg": "image/svg+xml",
    ".gif": "image/gif",
  };
  const contentType = mimeTypes[logoExtension] || "image/png";

  const logoFile = createGenericFile(logoBuffer, path.basename(logoPath), {
    contentType,
  });

  const [logoUri] = await umi.uploader.upload([logoFile]);
  console.log("Logo uploaded:", logoUri);

  // --- Step 2: Update metadata JSON ---
  console.log("\n[2/3] Updating metadata JSON...");
  const metadataPath = path.join(
    __dirname,
    "..",
    "token-metadata",
    "godcoin-metadata.json"
  );
  const metadata = JSON.parse(fs.readFileSync(metadataPath, "utf-8"));
  metadata.image = logoUri;
  metadata.properties.files[0].uri = logoUri;

  // Write updated metadata locally
  fs.writeFileSync(metadataPath, JSON.stringify(metadata, null, 2) + "\n");
  console.log("Local metadata updated with image URI");

  // --- Step 3: Upload metadata JSON ---
  console.log("\n[3/3] Uploading metadata JSON to Arweave...");
  const metadataFile = createGenericFile(
    Buffer.from(JSON.stringify(metadata)),
    "godcoin-metadata.json",
    { contentType: "application/json" }
  );

  const [metadataUri] = await umi.uploader.upload([metadataFile]);
  console.log("Metadata uploaded:", metadataUri);

  // --- Done ---
  console.log("\n========================================");
  console.log("UPLOAD COMPLETE");
  console.log("========================================");
  console.log("Logo URI:     ", logoUri);
  console.log("Metadata URI: ", metadataUri);
  console.log("\nUse this metadata URI in the next step:");
  console.log(
    `  node scripts/update-metadata.js ${metadataUri} <path-to-keypair.json>`
  );
  console.log("========================================");
}

main().catch((err) => {
  console.error("Error:", err.message || err);
  process.exit(1);
});
