import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with account:", deployer.address);

  // Deploy P2PEscrow
  const P2PEscrow = await ethers.getContractFactory("P2PEscrow");
  const p2pEscrow = await P2PEscrow.deploy(deployer.address);
  await p2pEscrow.waitForDeployment();
  console.log("P2PEscrow deployed to:", await p2pEscrow.getAddress());

  // Deploy FeeSplitter
  const FeeSplitter = await ethers.getContractFactory("FeeSplitter");
  const feeSplitter = await FeeSplitter.deploy(deployer.address);
  await feeSplitter.waitForDeployment();
  console.log("FeeSplitter deployed to:", await feeSplitter.getAddress());

  console.log("\n✅ Deployment complete!");
  console.log("\nSave these addresses:");
  console.log("P2PEscrow:", await p2pEscrow.getAddress());
  console.log("FeeSplitter:", await feeSplitter.getAddress());
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
