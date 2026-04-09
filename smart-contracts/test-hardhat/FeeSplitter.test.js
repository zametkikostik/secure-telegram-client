const { expect } = require("chai");
const { ethers } = require("hardhat");
const { time, loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

/**
 * FeeSplitter Hardhat + Chai Tests
 *
 * Full test suite for Fee Splitter smart contract
 */

async function deployFeeSplitterFixture() {
  const [owner, escrowContract, teamWallet, treasuryWallet, marketingWallet, reserveWallet, arbiter1, arbiter2, ...otherAccounts] = await ethers.getSigners();

  // Deploy FeeSplitter
  const FeeSplitter = await ethers.getContractFactory("FeeSplitter");
  const feeSplitter = await FeeSplitter.deploy(
    owner.address,
    escrowContract.address,
    teamWallet.address,
    treasuryWallet.address,
    marketingWallet.address,
    reserveWallet.address
  );
  await feeSplitter.waitForDeployment();

  return {
    feeSplitter,
    owner,
    escrowContract,
    teamWallet,
    treasuryWallet,
    marketingWallet,
    reserveWallet,
    arbiter1,
    arbiter2,
    otherAccounts,
  };
}

describe("FeeSplitter", function () {
  // ========================================================================
  // Constructor & Initialization Tests
  // ========================================================================

  describe("Constructor & Initialization", function () {
    it("Should initialize with correct default values", async function () {
      const { feeSplitter, owner, escrowContract } = await loadFixture(deployFeeSplitterFixture);

      expect(await feeSplitter.owner()).to.equal(owner.address);
      expect(await feeSplitter.escrowContract()).to.equal(escrowContract.address);

      // Default percentages: 40/25/15/10/10
      expect(await feeSplitter.teamPercent()).to.equal(40);
      expect(await feeSplitter.treasuryPercent()).to.equal(25);
      expect(await feeSplitter.marketingPercent()).to.equal(15);
      expect(await feeSplitter.arbitersPercent()).to.equal(10);
      expect(await feeSplitter.reservePercent()).to.equal(10);
    });

    it("Should emit events for shareholder setup", async function () {
      const { feeSplitter, teamWallet, treasuryWallet, marketingWallet, reserveWallet } = await loadFixture(deployFeeSplitterFixture);

      await expect(feeSplitter.deploymentTransaction())
        .to.emit(feeSplitter, "ShareholderUpdated")
        .withArgs("Team", teamWallet.address, 40);

      await expect(feeSplitter.deploymentTransaction())
        .to.emit(feeSplitter, "ShareholderUpdated")
        .withArgs("Treasury", treasuryWallet.address, 25);
    });

    it("Should fail with invalid owner", async function () {
      const { escrowContract, teamWallet, treasuryWallet, marketingWallet, reserveWallet } = await loadFixture(deployFeeSplitterFixture);

      const FeeSplitter = await ethers.getContractFactory("FeeSplitter");

      await expect(
        FeeSplitter.deploy(
          ethers.ZeroAddress,
          escrowContract.address,
          teamWallet.address,
          treasuryWallet.address,
          marketingWallet.address,
          reserveWallet.address
        )
      ).to.be.revertedWith("Invalid owner");
    });

    it("Should fail with invalid escrow", async function () {
      const { owner, teamWallet, treasuryWallet, marketingWallet, reserveWallet } = await loadFixture(deployFeeSplitterFixture);

      const FeeSplitter = await ethers.getContractFactory("FeeSplitter");

      await expect(
        FeeSplitter.deploy(
          owner.address,
          ethers.ZeroAddress,
          teamWallet.address,
          treasuryWallet.address,
          marketingWallet.address,
          reserveWallet.address
        )
      ).to.be.revertedWith("Invalid escrow");
    });
  });

  // ========================================================================
  // Fee Reception Tests
  // ========================================================================

  describe("Fee Reception", function () {
    it("Should receive ETH fee from escrow", async function () {
      const { feeSplitter, escrowContract } = await loadFixture(deployFeeSplitterFixture);

      const feeAmount = ethers.parseEther("1");

      // Send ETH from escrow contract (simulate)
      await escrowContract.sendTransaction({
        to: feeSplitter.target,
        value: feeAmount,
      });

      const stats = await feeSplitter.stats();
      expect(stats.totalFeesReceived).to.equal(feeAmount);
      expect(stats.pendingBalance).to.equal(feeAmount);
    });

    it("Should fail receiving from non-escrow", async function () {
      const { feeSplitter, owner } = await loadFixture(deployFeeSplitterFixture);

      await expect(
        owner.sendTransaction({
          to: feeSplitter.target,
          value: ethers.parseEther("1"),
        })
      ).to.be.revertedWith("Only escrow contract");
    });
  });

  // ========================================================================
  // Distribution Tests
  // ========================================================================

  describe("Distribution", function () {
    it("Should distribute ETH fees correctly", async function () {
      const { feeSplitter, escrowContract, teamWallet, treasuryWallet, marketingWallet, reserveWallet } = await loadFixture(deployFeeSplitterFixture);

      const feeAmount = ethers.parseEther("10");

      // Receive fee
      await escrowContract.sendTransaction({
        to: feeSplitter.target,
        value: feeAmount,
      });

      // Get balances before
      const teamBefore = await ethers.provider.getBalance(teamWallet.address);
      const treasuryBefore = await ethers.provider.getBalance(treasuryWallet.address);
      const marketingBefore = await ethers.provider.getBalance(marketingWallet.address);
      const reserveBefore = await ethers.provider.getBalance(reserveWallet.address);

      // Distribute
      await feeSplitter.distribute();

      // Verify payouts (40/25/15/10/10 split)
      const teamAfter = await ethers.provider.getBalance(teamWallet.address);
      const treasuryAfter = await ethers.provider.getBalance(treasuryWallet.address);
      const marketingAfter = await ethers.provider.getBalance(marketingWallet.address);
      const reserveAfter = await ethers.provider.getBalance(reserveWallet.address);

      // Expected amounts (minus rounding)
      const teamExpected = ethers.parseEther("4"); // 40%
      const treasuryExpected = ethers.parseEther("2.5"); // 25%
      const marketingExpected = ethers.parseEther("1.5"); // 15%
      const reserveExpected = ethers.parseEther("1"); // 10%

      expect(teamAfter - teamBefore).to.be.closeTo(teamExpected, ethers.parseEther("0.001"));
      expect(treasuryAfter - treasuryBefore).to.be.closeTo(treasuryExpected, ethers.parseEther("0.001"));
      expect(marketingAfter - marketingBefore).to.be.closeTo(marketingExpected, ethers.parseEther("0.001"));
      expect(reserveAfter - reserveBefore).to.be.closeTo(reserveExpected, ethers.parseEther("0.001"));

      // Verify stats updated
      const stats = await feeSplitter.stats();
      expect(stats.totalDistributed).to.be.gt(0);
      expect(stats.distributionCount).to.equal(1);
      expect(stats.pendingBalance).to.equal(0);
    });

    it("Should fail distributing with no pending balance", async function () {
      const { feeSplitter } = await loadFixture(deployFeeSplitterFixture);

      await expect(feeSplitter.distribute())
        .to.be.revertedWith("No pending balance");
    });

    it("Should emit Distributed event", async function () {
      const { feeSplitter, escrowContract } = await loadFixture(deployFeeSplitterFixture);

      await escrowContract.sendTransaction({
        to: feeSplitter.target,
        value: ethers.parseEther("1"),
      });

      await expect(feeSplitter.distribute())
        .to.emit(feeSplitter, "Distributed");
    });
  });

  // ========================================================================
  // Admin Functions Tests
  // ========================================================================

  describe("Admin Functions", function () {
    it("Should update shares", async function () {
      const { feeSplitter, owner } = await loadFixture(deployFeeSplitterFixture);

      // New split: 50/20/10/10/10
      await feeSplitter.connect(owner).updateShares(50, 20, 10, 10, 10);

      expect(await feeSplitter.teamPercent()).to.equal(50);
      expect(await feeSplitter.treasuryPercent()).to.equal(20);
      expect(await feeSplitter.marketingPercent()).to.equal(10);
      expect(await feeSplitter.arbitersPercent()).to.equal(10);
      expect(await feeSplitter.reservePercent()).to.equal(10);
    });

    it("Should fail updating shares from non-owner", async function () {
      const { feeSplitter, owner } = await loadFixture(deployFeeSplitterFixture);

      await expect(
        feeSplitter.connect(owner).updateShares(50, 20, 10, 10, 10)
      ).to.not.be.reverted;

      const nonOwner = (await ethers.getSigners())[9];
      await expect(
        feeSplitter.connect(nonOwner).updateShares(50, 20, 10, 10, 10)
      ).to.be.revertedWith("Only owner");
    });

    it("Should fail updating shares that don't sum to 100", async function () {
      const { feeSplitter, owner } = await loadFixture(deployFeeSplitterFixture);

      await expect(
        feeSplitter.connect(owner).updateShares(50, 30, 20, 10, 10) // = 120
      ).to.be.revertedWith("Shares must sum to 100");
    });

    it("Should update shareholder wallet", async function () {
      const { feeSplitter, owner, teamWallet } = await loadFixture(deployFeeSplitterFixture);

      const newTeamWallet = (await ethers.getSigners())[9].address;

      await feeSplitter.connect(owner).updateShareholderWallet("Team", newTeamWallet);

      const shareholders = await feeSplitter.getShareholders();
      expect(shareholders[0].wallet).to.equal(newTeamWallet);
    });

    it("Should update escrow contract", async function () {
      const { feeSplitter, owner, escrowContract } = await loadFixture(deployFeeSplitterFixture);

      const newEscrow = (await ethers.getSigners())[9].address;

      await expect(
        feeSplitter.connect(owner).updateEscrowContract(newEscrow)
      ).to.emit(feeSplitter, "EscrowContractUpdated")
        .withArgs(escrowContract.address, newEscrow);

      expect(await feeSplitter.escrowContract()).to.equal(newEscrow);
    });

    it("Should toggle pause", async function () {
      const { feeSplitter, owner, escrowContract } = await loadFixture(deployFeeSplitterFixture);

      // Pause
      await feeSplitter.connect(owner).togglePause();
      expect(await feeSplitter.paused()).to.be.true;

      // Should fail distributing when paused
      await escrowContract.sendTransaction({
        to: feeSplitter.target,
        value: ethers.parseEther("1"),
      });

      await expect(feeSplitter.distribute())
        .to.be.revertedWith("Contract paused");

      // Unpause
      await feeSplitter.connect(owner).togglePause();
      expect(await feeSplitter.paused()).to.be.false;

      // Should work again
      await feeSplitter.distribute();
    });

    it("Should transfer ownership", async function () {
      const { feeSplitter, owner } = await loadFixture(deployFeeSplitterFixture);

      const newOwner = (await ethers.getSigners())[9];

      await feeSplitter.connect(owner).transferOwnership(newOwner.address);

      expect(await feeSplitter.owner()).to.equal(newOwner.address);
    });
  });

  // ========================================================================
  // Arbiter Pool Tests
  // ========================================================================

  describe("Arbiter Pool", function () {
    it("Should add arbiter to pool", async function () {
      const { feeSplitter, owner, arbiter1 } = await loadFixture(deployFeeSplitterFixture);

      await expect(
        feeSplitter.connect(owner).addArbiter(arbiter1.address, 100)
      ).to.emit(feeSplitter, "ArbiterAdded")
        .withArgs(arbiter1.address, 100);

      const poolInfo = await feeSplitter.getArbiterPoolInfo();
      expect(poolInfo.totalShares).to.equal(100);
    });

    it("Should remove arbiter from pool", async function () {
      const { feeSplitter, owner, arbiter1 } = await loadFixture(deployFeeSplitterFixture);

      await feeSplitter.connect(owner).addArbiter(arbiter1.address, 100);
      await expect(
        feeSplitter.connect(owner).removeArbiter(arbiter1.address)
      ).to.emit(feeSplitter, "ArbiterRemoved")
        .withArgs(arbiter1.address, 100);

      const poolInfo = await feeSplitter.getArbiterPoolInfo();
      expect(poolInfo.totalShares).to.equal(0);
    });

    it("Should calculate arbiter share", async function () {
      const { feeSplitter, owner, escrowContract, arbiter1, arbiter2 } = await loadFixture(deployFeeSplitterFixture);

      // Add arbiters with shares
      await feeSplitter.connect(owner).addArbiter(arbiter1.address, 60);
      await feeSplitter.connect(owner).addArbiter(arbiter2.address, 40);

      // Receive and distribute fees
      await escrowContract.sendTransaction({
        to: feeSplitter.target,
        value: ethers.parseEther("10"),
      });
      await feeSplitter.distribute();

      // Calculate shares
      const arb1Share = await feeSplitter.calculateArbiterShare(arbiter1.address);
      const arb2Share = await feeSplitter.calculateArbiterShare(arbiter2.address);

      // Arbiter pool gets 10% of 10 ETH = 1 ETH
      // Arb1: 60% of 1 ETH = 0.6 ETH
      // Arb2: 40% of 1 ETH = 0.4 ETH
      expect(arb1Share).to.be.closeTo(ethers.parseEther("0.6"), ethers.parseEther("0.001"));
      expect(arb2Share).to.be.closeTo(ethers.parseEther("0.4"), ethers.parseEther("0.001"));
    });

    it("Should arbiter withdraw", async function () {
      const { feeSplitter, owner, escrowContract, arbiter1 } = await loadFixture(deployFeeSplitterFixture);

      // Add arbiter
      await feeSplitter.connect(owner).addArbiter(arbiter1.address, 100);

      // Receive and distribute
      await escrowContract.sendTransaction({
        to: feeSplitter.target,
        value: ethers.parseEther("10"),
      });
      await feeSplitter.distribute();

      const arbiterBalanceBefore = await ethers.provider.getBalance(arbiter1.address);

      // Withdraw
      await expect(feeSplitter.connect(arbiter1).arbiterWithdraw())
        .to.emit(feeSplitter, "ArbiterWithdrawal");

      const arbiterBalanceAfter = await ethers.provider.getBalance(arbiter1.address);
      expect(arbiterBalanceAfter - arbiterBalanceBefore).to.be.gt(0);
    });

    it("Should fail arbiter withdraw with no share", async function () {
      const { feeSplitter, arbiter1 } = await loadFixture(deployFeeSplitterFixture);

      await expect(
        feeSplitter.connect(arbiter1).arbiterWithdraw()
      ).to.be.revertedWith("No share");
    });
  });

  // ========================================================================
  // View Functions Tests
  // ========================================================================

  describe("View Functions", function () {
    it("Should get shareholders", async function () {
      const { feeSplitter, teamWallet, treasuryWallet, marketingWallet, reserveWallet } = await loadFixture(deployFeeSplitterFixture);

      const shareholders = await feeSplitter.getShareholders();

      expect(shareholders[0].wallet).to.equal(teamWallet.address);
      expect(shareholders[1].wallet).to.equal(treasuryWallet.address);
      expect(shareholders[2].wallet).to.equal(marketingWallet.address);
      expect(shareholders[3].wallet).to.equal(reserveWallet.address);
    });

    it("Should get distribution history", async function () {
      const { feeSplitter, escrowContract } = await loadFixture(deployFeeSplitterFixture);

      // Receive and distribute twice
      for (let i = 0; i < 2; i++) {
        await escrowContract.sendTransaction({
          to: feeSplitter.target,
          value: ethers.parseEther("1"),
        });
        await feeSplitter.distribute();
      }

      const history = await feeSplitter.getDistributionHistory(0, 10);
      expect(history.length).to.equal(2);
    });
  });

  // ========================================================================
  // Edge Cases Tests
  // ========================================================================

  describe("Edge Cases", function () {
    it("Should handle multiple distributions", async function () {
      const { feeSplitter, escrowContract, teamWallet } = await loadFixture(deployFeeSplitterFixture);

      const teamBefore = await ethers.provider.getBalance(teamWallet.address);

      // Distribute multiple times
      for (let i = 0; i < 5; i++) {
        await escrowContract.sendTransaction({
          to: feeSplitter.target,
          value: ethers.parseEther("1"),
        });
        await feeSplitter.distribute();
      }

      const teamAfter = await ethers.provider.getBalance(teamWallet.address);
      expect(teamAfter - teamBefore).to.be.gt(0);

      const stats = await feeSplitter.stats();
      expect(stats.distributionCount).to.equal(5);
    });

    it("Should handle zero fee amount gracefully", async function () {
      const { feeSplitter } = await loadFixture(deployFeeSplitterFixture);

      await expect(feeSplitter.distribute())
        .to.be.revertedWith("No pending balance");
    });
  });
});
