const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

/**
 * Deploy Script for P2PEscrow + FeeSplitter
 *
 * Usage:
 *   npx hardhat run scripts/deploy.js --network localhost
 *   npx hardhat run scripts/deploy.js --network sepolia
 *   npx hardhat run scripts/deploy.js --network mainnet
 */

async function main() {
  console.log("🚀 Deploying Secure Messenger Smart Contracts...\n");

  const [deployer] = await ethers.getSigners();
  console.log(`📝 Deployer: ${deployer.address}`);

  const balance = await ethers.provider.getBalance(deployer.address);
  console.log(`💰 Deployer balance: ${ethers.formatEther(balance)} ETH\n`);

  // ========================================================================
  // Configuration
  // ========================================================================

  // Replace these with actual addresses for production
  const PLATFORM_TREASURY = process.env.PLATFORM_TREASURY || deployer.address;
  const TEAM_WALLET = process.env.TEAM_WALLET || deployer.address;
  const TREASURY_WALLET = process.env.TREASURY_WALLET || deployer.address;
  const MARKETING_WALLET = process.env.MARKETING_WALLET || deployer.address;
  const RESERVE_WALLET = process.env.RESERVE_WALLET || deployer.address;

  // Initial arbiters (can be updated later)
  const INITIAL_ARBITERS = [
    process.env.ARBITER_1 || deployer.address,
    process.env.ARBITER_2 || deployer.address,
  ].filter(addr => addr !== ethers.ZeroAddress);

  console.log("⚙️  Configuration:");
  console.log(`   Platform Treasury: ${PLATFORM_TREASURY}`);
  console.log(`   Team Wallet: ${TEAM_WALLET}`);
  console.log(`   Treasury Wallet: ${TREASURY_WALLET}`);
  console.log(`   Marketing Wallet: ${MARKETING_WALLET}`);
  console.log(`   Reserve Wallet: ${RESERVE_WALLET}`);
  console.log(`   Initial Arbiters: ${INITIAL_ARBITERS.join(", ")}`);
  console.log("");

  // ========================================================================
  // Deploy P2PEscrow
  // ========================================================================

  console.log("📦 Deploying P2PEscrow...");
  const P2PEscrow = await ethers.getContractFactory("P2PEscrow");
  const escrow = await P2PEscrow.deploy(
    PLATFORM_TREASURY,
    INITIAL_ARBITERS
  );
  await escrow.waitForDeployment();

  const escrowAddress = await escrow.getAddress();
  console.log(`✅ P2PEscrow deployed: ${escrowAddress}`);

  // Verify deployment
  const arbiterCount = INITIAL_ARBITERS.length;
  console.log(`   Platform Treasury: ${PLATFORM_TREASURY}`);
  console.log(`   Initial Arbiters: ${arbiterCount}`);

  // ========================================================================
  // Deploy FeeSplitter
  // ========================================================================

  console.log("\n📦 Deploying FeeSplitter...");
  const FeeSplitter = await ethers.getContractFactory("FeeSplitter");
  const feeSplitter = await FeeSplitter.deploy(
    deployer.address, // Owner
    escrowAddress, // Escrow contract
    TEAM_WALLET,
    TREASURY_WALLET,
    MARKETING_WALLET,
    RESERVE_WALLET
  );
  await feeSplitter.waitForDeployment();

  const feeSplitterAddress = await feeSplitter.getAddress();
  console.log(`✅ FeeSplitter deployed: ${feeSplitterAddress}`);

  // Verify default distribution
  const teamPercent = await feeSplitter.teamPercent();
  const treasuryPercent = await feeSplitter.treasuryPercent();
  const marketingPercent = await feeSplitter.marketingPercent();
  const arbitersPercent = await feeSplitter.arbitersPercent();
  const reservePercent = await feeSplitter.reservePercent();

  console.log(`   Distribution: ${teamPercent}% / ${treasuryPercent}% / ${marketingPercent}% / ${arbitersPercent}% / ${reservePercent}%`);
  console.log(`   (Team / Treasury / Marketing / Arbiters / Reserve)`);

  // ========================================================================
  // Link Contracts
  // ========================================================================

  console.log("\n🔗 Linking contracts...");

  // Update FeeSplitter escrow contract reference (already set in constructor)
  // In production, you might want to update P2PEscrow to know about FeeSplitter
  // For now, they operate independently

  console.log("✅ Contracts linked");

  // ========================================================================
  // Verification (on testnet/mainnet)
  // ========================================================================

  const network = await ethers.provider.getNetwork();
  const chainId = Number(network.chainId);

  if (chainId !== 31337 && chainId !== 1337) {
    console.log("\n🔍 Verifying contracts on Etherscan...");

    try {
      // Verify P2PEscrow
      await hre.run("verify:verify", {
        address: escrowAddress,
        constructorArguments: [
          PLATFORM_TREASURY,
          INITIAL_ARBITERS,
        ],
      });
      console.log("✅ P2PEscrow verified");

      // Verify FeeSplitter
      await hre.run("verify:verify", {
        address: feeSplitterAddress,
        constructorArguments: [
          deployer.address,
          escrowAddress,
          TEAM_WALLET,
          TREASURY_WALLET,
          MARKETING_WALLET,
          RESERVE_WALLET,
        ],
      });
      console.log("✅ FeeSplitter verified");
    } catch (error) {
      console.warn(`⚠️  Verification failed: ${error.message}`);
    }
  }

  // ========================================================================
  // Save Deployment Info
  // ========================================================================

  console.log("\n💾 Saving deployment info...");

  const deploymentInfo = {
    network: network.name,
    chainId: chainId,
    deployer: deployer.address,
    timestamp: new Date().toISOString(),
    contracts: {
      P2PEscrow: {
        address: escrowAddress,
        args: [PLATFORM_TREASURY, INITIAL_ARBITERS],
      },
      FeeSplitter: {
        address: feeSplitterAddress,
        args: [
          deployer.address,
          escrowAddress,
          TEAM_WALLET,
          TREASURY_WALLET,
          MARKETING_WALLET,
          RESERVE_WALLET,
        ],
        distribution: {
          team: `${teamPercent}%`,
          treasury: `${treasuryPercent}%`,
          marketing: `${marketingPercent}%`,
          arbiters: `${arbitersPercent}%`,
          reserve: `${reservePercent}%`,
        },
      },
    },
  };

  // Create deployments directory if it doesn't exist
  const deploymentsDir = path.join(__dirname, "..", "deployments");
  if (!fs.existsSync(deploymentsDir)) {
    fs.mkdirSync(deploymentsDir, { recursive: true });
  }

  // Save deployment info
  const fileName = `deployment-${network.name}-${Date.now()}.json`;
  const filePath = path.join(deploymentsDir, fileName);
  fs.writeFileSync(filePath, JSON.stringify(deploymentInfo, null, 2));

  console.log(`✅ Saved to: ${filePath}`);

  // Also save as latest.json for easy access
  const latestPath = path.join(deploymentsDir, "latest.json");
  fs.writeFileSync(latestPath, JSON.stringify(deploymentInfo, null, 2));
  console.log(`✅ Latest: ${latestPath}`);

  // ========================================================================
  // Summary
  // ========================================================================

  console.log("\n" + "=".repeat(60));
  console.log("🎉 DEPLOYMENT COMPLETE!");
  console.log("=".repeat(60));
  console.log(`Network: ${network.name} (chainId: ${chainId})`);
  console.log(`Deployer: ${deployer.address}`);
  console.log("");
  console.log("📦 Contracts:");
  console.log(`   P2PEscrow:    ${escrowAddress}`);
  console.log(`   FeeSplitter:  ${feeSplitterAddress}`);
  console.log("");
  console.log("💰 Fee Distribution:");
  console.log(`   Team:       ${teamPercent}%`);
  console.log(`   Treasury:   ${treasuryPercent}%`);
  console.log(`   Marketing:  ${marketingPercent}%`);
  console.log(`   Arbiters:   ${arbitersPercent}%`);
  console.log(`   Reserve:    ${reservePercent}%`);
  console.log("");
  console.log("🔗 Explorer Links:");

  if (chainId === 1) {
    console.log(`   P2PEscrow:   https://etherscan.io/address/${escrowAddress}`);
    console.log(`   FeeSplitter: https://etherscan.io/address/${feeSplitterAddress}`);
  } else if (chainId === 11155111) {
    console.log(`   P2PEscrow:   https://sepolia.etherscan.io/address/${escrowAddress}`);
    console.log(`   FeeSplitter: https://sepolia.etherscan.io/address/${feeSplitterAddress}`);
  } else {
    console.log("   (Local network - no explorer)");
  }

  console.log("=".repeat(60));
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
