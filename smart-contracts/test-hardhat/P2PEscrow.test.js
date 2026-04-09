const { expect } = require("chai");
const { ethers } = require("hardhat");
const { time, loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

/**
 * P2PEscrow Hardhat + Chai Tests
 *
 * Full test suite for P2P Escrow smart contract
 */

async function deployP2PEscrowFixture() {
  const [owner, platformTreasury, arbiter1, arbiter2, buyer, seller, buyer2, ...otherAccounts] = await ethers.getSigners();

  // Deploy Mock ERC-20
  const MockERC20 = await ethers.getContractFactory("MockERC20");
  const token = await MockERC20.deploy("Test Token", "TST", 18);
  await token.waitForDeployment();

  // Deploy P2PEscrow
  const P2PEscrow = await ethers.getContractFactory("P2PEscrow");
  const escrow = await P2PEscrow.deploy(
    platformTreasury.address,
    [arbiter1.address, arbiter2.address]
  );
  await escrow.waitForDeployment();

  // Fund buyers with ETH
  const fundTx = { to: buyer.address, value: ethers.parseEther("100") };
  await owner.sendTransaction(fundTx);
  await owner.sendTransaction({ to: buyer2.address, value: ethers.parseEther("100") });

  // Fund buyers with tokens
  await token.mint(buyer.address, ethers.parseEther("10000"));
  await token.mint(buyer2.address, ethers.parseEther("10000"));

  return {
    escrow,
    token,
    owner,
    platformTreasury,
    arbiter1,
    arbiter2,
    buyer,
    seller,
    buyer2,
    otherAccounts,
  };
}

describe("P2PEscrow", function () {
  // ========================================================================
  // Deal Creation Tests
  // ========================================================================

  describe("Deal Creation", function () {
    it("Should create a deal successfully", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60; // 7 days

      await expect(
        escrow.connect(buyer).createDeal(
          seller.address,
          0, // DigitalGoods
          deadline,
          ethers.keccak256(ethers.toUtf8Bytes("deal metadata")),
          "QmTest123"
        )
      ).to.emit(escrow, "DealCreated");

      const deal = await escrow.getDeal(0);
      expect(deal.buyer).to.equal(buyer.address);
      expect(deal.seller).to.equal(seller.address);
      expect(deal.state).to.equal(0); // Created
      expect(deal.messageHash).to.equal(ethers.keccak256(ethers.toUtf8Bytes("deal metadata")));
      expect(deal.ipfsMetadata).to.equal("QmTest123");
    });

    it("Should fail with invalid seller", async function () {
      const { escrow, buyer } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      await expect(
        escrow.connect(buyer).createDeal(
          ethers.ZeroAddress,
          0,
          deadline,
          ethers.ZeroHash,
          ""
        )
      ).to.be.revertedWith("Invalid seller");
    });

    it("Should fail with self-deal", async function () {
      const { escrow, buyer } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      await expect(
        escrow.connect(buyer).createDeal(
          buyer.address,
          0,
          deadline,
          ethers.ZeroHash,
          ""
        )
      ).to.be.revertedWith("Cannot deal with yourself");
    });

    it("Should fail with deadline too short", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 30 * 60; // 30 minutes

      await expect(
        escrow.connect(buyer).createDeal(
          seller.address,
          0,
          deadline,
          ethers.ZeroHash,
          ""
        )
      ).to.be.revertedWith("Deadline too short");
    });

    it("Should fail with deadline too long", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 100 * 24 * 60 * 60; // 100 days

      await expect(
        escrow.connect(buyer).createDeal(
          seller.address,
          0,
          deadline,
          ethers.ZeroHash,
          ""
        )
      ).to.be.revertedWith("Deadline too long");
    });
  });

  // ========================================================================
  // Deal Funding Tests (ETH)
  // ========================================================================

  describe("Deal Funding (ETH)", function () {
    it("Should create and fund deal with ETH", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;
      const fundAmount = ethers.parseEther("1");

      await expect(
        escrow.connect(buyer).createAndFundDeal(
          seller.address,
          2, // Service
          deadline,
          ethers.keccak256(ethers.toUtf8Bytes("service deal")),
          "QmService456",
          { value: fundAmount }
        )
      ).to.emit(escrow, "DealFunded");

      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(1); // Funded
      expect(deal.amount).to.equal(fundAmount);

      // Verify fee calculation (1% for 1 ETH)
      const expectedFee = await escrow.calculateFee(fundAmount);
      expect(deal.platformFee).to.equal(expectedFee);
    });

    it("Should fund deal separately", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create deal
      await escrow.connect(buyer).createDeal(
        seller.address,
        1, // PhysicalGoods
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("physical deal")),
        ""
      );

      // Fund deal
      const fundAmount = ethers.parseEther("0.5");
      await expect(
        escrow.connect(buyer).fundDeal(0, { value: fundAmount })
      ).to.emit(escrow, "DealFunded");

      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(1); // Funded
      expect(deal.amount).to.equal(fundAmount);
    });

    it("Should fail funding with zero ETH", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      await escrow.connect(buyer).createDeal(
        seller.address,
        0,
        deadline,
        ethers.ZeroHash,
        ""
      );

      await expect(
        escrow.connect(buyer).fundDeal(0, { value: 0 })
      ).to.be.revertedWith("Must send ETH");
    });
  });

  // ========================================================================
  // Fee Calculation Tests
  // ========================================================================

  describe("Fee Calculation", function () {
    it("Should calculate 2% fee for < 0.1 ETH", async function () {
      const { escrow } = await loadFixture(deployP2PEscrowFixture);

      const amount = ethers.parseEther("0.05");
      const fee = await escrow.calculateFee(amount);

      expect(fee).to.equal(ethers.parseEther("0.001")); // 2% of 0.05
    });

    it("Should calculate 1% fee for < 1 ETH", async function () {
      const { escrow } = await loadFixture(deployP2PEscrowFixture);

      const amount = ethers.parseEther("0.5");
      const fee = await escrow.calculateFee(amount);

      expect(fee).to.equal(ethers.parseEther("0.005")); // 1% of 0.5
    });

    it("Should calculate 0.5% fee for >= 1 ETH", async function () {
      const { escrow } = await loadFixture(deployP2PEscrowFixture);

      const amount = ethers.parseEther("2");
      const fee = await escrow.calculateFee(amount);

      expect(fee).to.equal(ethers.parseEther("0.01")); // 0.5% of 2
    });
  });

  // ========================================================================
  // Deal Completion Tests
  // ========================================================================

  describe("Deal Completion", function () {
    it("Should complete deal happy path", async function () {
      const { escrow, buyer, seller, platformTreasury } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;
      const fundAmount = ethers.parseEther("1");

      // Create and fund
      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        0, // DigitalGoods
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("digital deal")),
        "",
        { value: fundAmount }
      );

      // Confirm delivery
      await escrow.connect(buyer).confirmDelivery(0);

      // Get balances before
      const sellerBalanceBefore = await ethers.provider.getBalance(seller.address);
      const treasuryBalanceBefore = await ethers.provider.getBalance(platformTreasury.address);

      // Complete deal
      await escrow.connect(buyer).completeDeal(0);

      // Verify state
      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(3); // Completed

      // Verify payouts
      const fee = deal.platformFee;
      const sellerAmount = deal.amount - fee;

      const sellerBalanceAfter = await ethers.provider.getBalance(seller.address);
      const treasuryBalanceAfter = await ethers.provider.getBalance(platformTreasury.address);

      expect(sellerBalanceAfter - sellerBalanceBefore).to.equal(sellerAmount);
      expect(treasuryBalanceAfter - treasuryBalanceBefore).to.equal(fee);
    });

    it("Should complete deal after deadline", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 3600; // 1 hour

      // Create and fund
      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        2, // Service
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("timed deal")),
        "",
        { value: ethers.parseEther("1") }
      );

      // Confirm delivery
      await escrow.connect(buyer).confirmDelivery(0);

      // Warp past deadline
      await time.increase(7200); // 2 hours

      // Anyone can complete
      const randomSigner = (await ethers.getSigners())[9];
      await escrow.connect(randomSigner).completeAfterDeadline(0);

      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(3); // Completed
    });
  });

  // ========================================================================
  // Refund Tests
  // ========================================================================

  describe("Refund After Deadline", function () {
    it("Should refund buyer after deadline", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 3600; // 1 hour

      // Create and fund
      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        1, // PhysicalGoods
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("risky deal")),
        "",
        { value: ethers.parseEther("1") }
      );

      const buyerBalanceBefore = await ethers.provider.getBalance(buyer.address);

      // Warp past deadline
      await time.increase(7200); // 2 hours

      // Refund
      await expect(escrow.refundAfterDeadline(0))
        .to.emit(escrow, "DealRefunded");

      // Verify refund
      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(5); // Refunded

      const buyerBalanceAfter = await ethers.provider.getBalance(buyer.address);
      const tx = await ethers.provider.getTransaction(escrow.refundAfterDeadline(0).hash);
      const gasUsed = tx.gasPrice * (await ethers.provider.getTransactionReceipt(escrow.refundAfterDeadline(0).hash)).gasUsed;

      expect(buyerBalanceAfter - buyerBalanceBefore + gasUsed).to.equal(deal.amount);
    });
  });

  // ========================================================================
  // Dispute Resolution Tests
  // ========================================================================

  describe("Dispute Resolution", function () {
    it("Should open dispute", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create and fund
      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        4, // Freelance
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("freelance deal")),
        "",
        { value: ethers.parseEther("1") }
      );

      // Open dispute
      await expect(
        escrow.connect(buyer).openDispute(0, "Seller did not deliver")
      ).to.emit(escrow, "DealDisputed");

      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(4); // Disputed
    });

    it("Should resolve dispute with full refund", async function () {
      const { escrow, buyer, seller, arbiter1 } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create and fund
      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        0, // DigitalGoods
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("disputed deal")),
        "",
        { value: ethers.parseEther("1") }
      );

      // Open dispute
      await escrow.connect(buyer).openDispute(0, "Fraud");

      const buyerBalanceBefore = await ethers.provider.getBalance(buyer.address);

      // Arbiter resolves
      await escrow.connect(arbiter1).resolveDispute(0, true, 100); // Full refund

      // Verify refund
      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(5); // Refunded

      const buyerBalanceAfter = await ethers.provider.getBalance(buyer.address);
      expect(buyerBalanceAfter - buyerBalanceBefore).to.be.closeTo(
        deal.amount,
        ethers.parseEther("0.001") // Allow for gas cost variance
      );
    });

    it("Should resolve dispute with partial refund", async function () {
      const { escrow, buyer, seller, arbiter1, platformTreasury } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create and fund
      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        2, // Service
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("partial dispute")),
        "",
        { value: ethers.parseEther("1") }
      );

      // Open dispute
      await escrow.connect(seller).openDispute(0, "Buyer unhappy with quality");

      const buyerBalanceBefore = await ethers.provider.getBalance(buyer.address);
      const sellerBalanceBefore = await ethers.provider.getBalance(seller.address);

      // Arbiter resolves: 40% to buyer, 60% to seller
      await escrow.connect(arbiter1).resolveDispute(0, false, 40);

      // Verify
      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(3); // Completed

      // Buyer got 40%
      const buyerBalanceAfter = await ethers.provider.getBalance(buyer.address);
      expect(buyerBalanceAfter - buyerBalanceBefore).to.equal(ethers.parseEther("0.4"));

      // Seller got 60% minus fee
      const sellerBalanceAfter = await ethers.provider.getBalance(seller.address);
      const sellerShare = ethers.parseEther("0.6") - deal.platformFee;
      expect(sellerBalanceAfter - sellerBalanceBefore).to.equal(sellerShare);
    });

    it("Should resolve dispute with full payout to seller", async function () {
      const { escrow, buyer, seller, arbiter1 } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create and fund
      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        0, // DigitalGoods
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("valid deal")),
        "",
        { value: ethers.parseEther("1") }
      );

      // Open dispute
      await escrow.connect(buyer).openDispute(0, "False claim");

      const sellerBalanceBefore = await ethers.provider.getBalance(seller.address);

      // Arbiter resolves: 100% to seller
      await escrow.connect(arbiter1).resolveDispute(0, false, 0); // 0% to buyer

      // Verify
      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(3); // Completed

      const sellerBalanceAfter = await ethers.provider.getBalance(seller.address);
      const sellerReceived = sellerBalanceAfter - sellerBalanceBefore;
      expect(sellerReceived).to.equal(deal.amount - deal.platformFee);
    });
  });

  // ========================================================================
  // Cancellation Tests
  // ========================================================================

  describe("Cancellation", function () {
    it("Should cancel deal", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      await escrow.connect(buyer).createDeal(
        seller.address,
        2, // Service
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("cancel this")),
        ""
      );

      await expect(escrow.connect(buyer).cancelDeal(0))
        .to.emit(escrow, "DealCancelled");

      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(6); // Cancelled
    });
  });

  // ========================================================================
  // ERC-20 Token Tests
  // ========================================================================

  describe("ERC-20 Token Deals", function () {
    it("Should fund deal with tokens", async function () {
      const { escrow, token, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create deal
      await escrow.connect(buyer).createDeal(
        seller.address,
        0, // DigitalGoods
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("token deal")),
        ""
      );

      // Approve and fund with tokens
      await token.connect(buyer).approve(escrow.target, ethers.parseEther("1000"));
      await escrow.connect(buyer).fundDealWithToken(0, ethers.parseEther("1000"));

      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(1); // Funded
      expect(deal.amount).to.equal(ethers.parseEther("1000"));
      expect(deal.paymentToken).to.equal(token.target);
    });

    it("Should complete deal with tokens", async function () {
      const { escrow, token, buyer, seller, platformTreasury } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create and fund with tokens
      await escrow.connect(buyer).createDeal(
        seller.address,
        2, // Service
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("token service")),
        ""
      );

      await token.connect(buyer).approve(escrow.target, ethers.parseEther("500"));
      await escrow.connect(buyer).fundDealWithToken(0, ethers.parseEther("500"));

      // Confirm and complete
      await escrow.connect(buyer).confirmDelivery(0);

      const sellerBalanceBefore = await token.balanceOf(seller.address);
      const treasuryBalanceBefore = await token.balanceOf(platformTreasury.address);

      await escrow.connect(buyer).completeDeal(0);

      // Verify token payouts
      const deal = await escrow.getDeal(0);
      expect(deal.state).to.equal(3); // Completed

      const fee = deal.platformFee;
      const sellerAmount = deal.amount - fee;

      expect(await token.balanceOf(seller.address) - sellerBalanceBefore).to.equal(sellerAmount);
      expect(await token.balanceOf(platformTreasury.address) - treasuryBalanceBefore).to.equal(fee);
    });
  });

  // ========================================================================
  // Admin Functions Tests
  // ========================================================================

  describe("Admin Functions", function () {
    it("Should add arbiter", async function () {
      const { escrow, platformTreasury } = await loadFixture(deployP2PEscrowFixture);

      const newArbiter = (await ethers.getSigners())[9].address;

      await escrow.connect(platformTreasury).addArbiter(newArbiter);

      expect(await escrow.authorizedArbiters(newArbiter)).to.be.true;
    });

    it("Should remove arbiter", async function () {
      const { escrow, platformTreasury, arbiter1 } = await loadFixture(deployP2PEscrowFixture);

      await escrow.connect(platformTreasury).removeArbiter(arbiter1.address);

      expect(await escrow.authorizedArbiters(arbiter1.address)).to.be.false;
    });

    it("Should set treasury", async function () {
      const { escrow, platformTreasury } = await loadFixture(deployP2PEscrowFixture);

      const newTreasury = (await ethers.getSigners())[9].address;

      await escrow.connect(platformTreasury).setTreasury(newTreasury);

      expect(await escrow.platformTreasury()).to.equal(newTreasury);
    });

    it("Should fail adding arbiter from non-treasury", async function () {
      const { escrow, buyer } = await loadFixture(deployP2PEscrowFixture);

      const newArbiter = (await ethers.getSigners())[9].address;

      await expect(
        escrow.connect(buyer).addArbiter(newArbiter)
      ).to.be.revertedWith("Only treasury");
    });
  });

  // ========================================================================
  // Statistics Tests
  // ========================================================================

  describe("Statistics", function () {
    it("Should track platform statistics", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      // Create and complete 3 deals
      for (let i = 0; i < 3; i++) {
        await escrow.connect(buyer).createAndFundDeal(
          seller.address,
          0, // DigitalGoods
          deadline,
          ethers.keccak256(ethers.toUtf8Bytes(`deal ${i}`)),
          "",
          { value: ethers.parseEther("0.1") }
        );

        await escrow.connect(buyer).confirmDelivery(i);
        await escrow.connect(buyer).completeDeal(i);
      }

      const platformStats = await escrow.getPlatformStats();

      expect(platformStats.totalDeals).to.equal(3);
      expect(platformStats.completedDeals).to.equal(3);
      expect(platformStats.totalFeesCollected).to.be.gt(0);
      expect(platformStats.totalVolume).to.be.gt(0);
    });
  });

  // ========================================================================
  // Edge Cases Tests
  // ========================================================================

  describe("Edge Cases", function () {
    it("Should fail funding non-existent deal", async function () {
      const { escrow, buyer } = await loadFixture(deployP2PEscrowFixture);

      await expect(
        escrow.connect(buyer).fundDeal(999, { value: ethers.parseEther("1") })
      ).to.be.revertedWith("Deal does not exist");
    });

    it("Should fail funding already funded deal", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      await escrow.connect(buyer).createAndFundDeal(
        seller.address,
        0,
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("funded")),
        "",
        { value: ethers.parseEther("1") }
      );

      await expect(
        escrow.connect(buyer).fundDeal(0, { value: ethers.parseEther("1") })
      ).to.be.revertedWith("Invalid deal state");
    });

    it("Should fail completing unfunded deal", async function () {
      const { escrow, buyer, seller } = await loadFixture(deployP2PEscrowFixture);

      const deadline = (await time.latest()) + 7 * 24 * 60 * 60;

      await escrow.connect(buyer).createDeal(
        seller.address,
        0,
        deadline,
        ethers.keccak256(ethers.toUtf8Bytes("unfunded")),
        ""
      );

      await expect(
        escrow.connect(buyer).completeDeal(0)
      ).to.be.revertedWith("Invalid deal state");
    });
  });
});
