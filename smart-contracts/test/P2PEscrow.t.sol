// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../contracts/P2PEscrow.sol";

/**
 * @title P2PEscrowTest
 * @notice Comprehensive test suite for P2P Escrow smart contract
 */
contract P2PEscrowTest is Test {
    P2PEscrow public escrow;
    
    address public platformTreasury = address(0x01);
    address public arbiter1 = address(0x02);
    address public arbiter2 = address(0x03);
    address public buyer = address(0x04);
    address public seller = address(0x05);
    address public buyer2 = address(0x06);
    
    // Mock ERC-20 token for testing
    MockERC20 public token;
    
    function setUp() public {
        address[] memory arbiters = new address[](2);
        arbiters[0] = arbiter1;
        arbiters[1] = arbiter2;
        
        escrow = new P2PEscrow(platformTreasury, arbiters);
        token = new MockERC20("Test Token", "TST", 18);
        
        // Fund buyers with ETH
        vm.deal(buyer, 100 ether);
        vm.deal(buyer2, 100 ether);
        
        // Fund buyers with tokens
        token.mint(buyer, 10000 * 10**18);
        token.mint(buyer2, 10000 * 10**18);
    }

    // ========================================================================
    // Test: Deal Creation
    // ========================================================================

    function test_CreateDeal() public {
        vm.prank(buyer);
        uint256 deadline = block.timestamp + 7 days;
        uint256 dealId = escrow.createDeal(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            deadline,
            keccak256("deal metadata"),
            "QmTest123"
        );

        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(deal.buyer, buyer);
        assertEq(deal.seller, seller);
        assertEq(uint(deal.dealType), uint(P2PEscrow.DealType.DigitalGoods));
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Created));
        assertEq(deal.messageHash, keccak256("deal metadata"));
        assertEq(deal.ipfsMetadata, "QmTest123");
    }

    function test_CreateDeal_InvalidSeller() public {
        vm.prank(buyer);
        vm.expectRevert("Invalid seller");
        escrow.createDeal(
            address(0),
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("metadata"),
            ""
        );
    }

    function test_CreateDeal_SelfDeal() public {
        vm.prank(buyer);
        vm.expectRevert("Cannot deal with yourself");
        escrow.createDeal(
            buyer,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("metadata"),
            ""
        );
    }

    function test_CreateDeal_DeadlineTooShort() public {
        vm.prank(buyer);
        vm.expectRevert("Deadline too short");
        escrow.createDeal(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 30 minutes,
            keccak256("metadata"),
            ""
        );
    }

    function test_CreateDeal_DeadlineTooLong() public {
        vm.prank(buyer);
        vm.expectRevert("Deadline too long");
        escrow.createDeal(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 100 days,
            keccak256("metadata"),
            ""
        );
    }

    // ========================================================================
    // Test: Deal Funding (ETH)
    // ========================================================================

    function test_CreateAndFundDeal_ETH() public {
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.Service,
            block.timestamp + 7 days,
            keccak256("service deal"),
            "QmService456"
        );

        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Funded));
        assertEq(deal.amount, 1 ether);
        
        // Calculate expected fee (1% for 1 ETH)
        uint256 expectedFee = escrow.calculateFee(1 ether);
        assertEq(deal.platformFee, expectedFee);
    }

    function test_FundDeal_Separate() public {
        vm.prank(buyer);
        uint256 dealId = escrow.createDeal(
            seller,
            P2PEscrow.DealType.PhysicalGoods,
            block.timestamp + 7 days,
            keccak256("physical deal"),
            ""
        );

        vm.prank(buyer);
        escrow.fundDeal{value: 0.5 ether}(dealId);

        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Funded));
        assertEq(deal.amount, 0.5 ether);
    }

    // ========================================================================
    // Test: Fee Calculation
    // ========================================================================

    function test_FeeTier1() public view {
        // 2% for < 0.1 ETH
        uint256 fee = escrow.calculateFee(0.05 ether);
        assertEq(fee, 0.001 ether); // 2% of 0.05 = 0.001
    }

    function test_FeeTier2() public view {
        // 1% for < 1 ETH
        uint256 fee = escrow.calculateFee(0.5 ether);
        assertEq(fee, 0.005 ether); // 1% of 0.5 = 0.005
    }

    function test_FeeTier3() public view {
        // 0.5% for >= 1 ETH
        uint256 fee = escrow.calculateFee(2 ether);
        assertEq(fee, 0.01 ether); // 0.5% of 2 = 0.01
    }

    // ========================================================================
    // Test: Deal Completion
    // ========================================================================

    function test_CompleteDeal_HappyPath() public {
        // Create and fund
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("digital deal"),
            ""
        );

        // Confirm delivery
        vm.prank(buyer);
        escrow.confirmDelivery(dealId);

        // Get balances before
        uint256 sellerBalanceBefore = seller.balance;
        uint256 treasuryBalanceBefore = platformTreasury.balance;

        // Complete deal (buyer or seller can call)
        vm.prank(buyer);
        escrow.completeDeal(dealId);

        // Verify state
        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Completed));

        // Verify payouts
        uint256 fee = deal.platformFee;
        uint256 sellerAmount = deal.amount - fee;
        
        assertEq(seller.balance - sellerBalanceBefore, sellerAmount);
        assertEq(platformTreasury.balance - treasuryBalanceBefore, fee);
    }

    function test_CompleteAfterDeadline() public {
        // Create and fund
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.Service,
            block.timestamp + 1 hours,
            keccak256("timed deal"),
            ""
        );

        // Confirm delivery
        vm.prank(buyer);
        escrow.confirmDelivery(dealId);

        // Warp past deadline
        vm.warp(block.timestamp + 2 hours);

        // Anyone can complete
        vm.prank(address(0x99));
        escrow.completeAfterDeadline(dealId);

        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Completed));
    }

    // ========================================================================
    // Test: Refund After Deadline
    // ========================================================================

    function test_RefundAfterDeadline() public {
        // Create and fund
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.PhysicalGoods,
            block.timestamp + 1 hours,
            keccak256("risky deal"),
            ""
        );

        uint256 buyerBalanceBefore = buyer.balance;

        // Warp past deadline
        vm.warp(block.timestamp + 2 hours);

        // Refund
        escrow.refundAfterDeadline(dealId);

        // Verify refund
        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Refunded));
        assertEq(buyer.balance - buyerBalanceBefore, deal.amount);
    }

    // ========================================================================
    // Test: Dispute Resolution
    // ========================================================================

    function test_OpenDispute() public {
        // Create and fund
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.Freelance,
            block.timestamp + 7 days,
            keccak256("freelance deal"),
            ""
        );

        // Open dispute (buyer)
        vm.prank(buyer);
        escrow.openDispute(dealId, "Seller did not deliver");

        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Disputed));
    }

    function test_ResolveDispute_FullRefund() public {
        // Create and fund
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("disputed deal"),
            ""
        );

        // Open dispute
        vm.prank(buyer);
        escrow.openDispute(dealId, "Fraud");

        uint256 buyerBalanceBefore = buyer.balance;

        // Arbiter resolves
        vm.prank(arbiter1);
        escrow.resolveDispute(dealId, true, 100); // Full refund

        // Verify refund
        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Refunded));
        assertEq(buyer.balance - buyerBalanceBefore, deal.amount);
    }

    function test_ResolveDispute_PartialRefund() public {
        // Create and fund
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.Service,
            block.timestamp + 7 days,
            keccak256("partial dispute"),
            ""
        );

        // Open dispute
        vm.prank(seller);
        escrow.openDispute(dealId, "Buyer unhappy with quality");

        uint256 buyerBalanceBefore = buyer.balance;
        uint256 sellerBalanceBefore = seller.balance;
        uint256 treasuryBalanceBefore = platformTreasury.balance;

        // Arbiter resolves: 40% to buyer, 60% to seller
        vm.prank(arbiter1);
        escrow.resolveDispute(dealId, false, 40);

        // Verify
        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Completed));
        
        // Buyer got 40%
        assertEq(buyer.balance - buyerBalanceBefore, 0.4 ether);
        // Seller got 60% minus fee
        uint256 sellerShare = 0.6 ether - deal.platformFee;
        assertEq(seller.balance - sellerBalanceBefore, sellerShare);
    }

    function test_ResolveDispute_FullPayout() public {
        // Create and fund
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("valid deal"),
            ""
        );

        // Open dispute
        vm.prank(buyer);
        escrow.openDispute(dealId, "False claim");

        uint256 sellerBalanceBefore = seller.balance;

        // Arbiter resolves: 100% to seller
        vm.prank(arbiter1);
        escrow.resolveDispute(dealId, false, 0); // 0% to buyer

        // Verify
        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Completed));
        
        uint256 sellerReceived = seller.balance - sellerBalanceBefore;
        assertEq(sellerReceived, deal.amount - deal.platformFee);
    }

    // ========================================================================
    // Test: Cancellation
    // ========================================================================

    function test_CancelDeal() public {
        vm.prank(buyer);
        uint256 dealId = escrow.createDeal(
            seller,
            P2PEscrow.DealType.Service,
            block.timestamp + 7 days,
            keccak256("cancel this"),
            ""
        );

        vm.prank(buyer);
        escrow.cancelDeal(dealId);

        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Cancelled));
    }

    // ========================================================================
    // Test: ERC-20 Token Deals
    // ========================================================================

    function test_FundDealWithToken() public {
        vm.prank(buyer);
        uint256 dealId = escrow.createDeal(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("token deal"),
            ""
        );

        // Approve and fund with tokens
        vm.prank(buyer);
        token.approve(address(escrow), 1000 * 10**18);
        
        vm.prank(buyer);
        escrow.fundDealWithToken(dealId, 1000 * 10**18);

        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Funded));
        assertEq(deal.amount, 1000 * 10**18);
        assertEq(deal.paymentToken, address(token));
    }

    function test_CompleteDeal_Token() public {
        // Create and fund with tokens
        vm.prank(buyer);
        uint256 dealId = escrow.createDeal(
            seller,
            P2PEscrow.DealType.Service,
            block.timestamp + 7 days,
            keccak256("token service"),
            ""
        );

        vm.prank(buyer);
        token.approve(address(escrow), 500 * 10**18);
        
        vm.prank(buyer);
        escrow.fundDealWithToken(dealId, 500 * 10**18);

        // Confirm and complete
        vm.prank(buyer);
        escrow.confirmDelivery(dealId);

        uint256 sellerBalanceBefore = token.balanceOf(seller);
        uint256 treasuryBalanceBefore = token.balanceOf(platformTreasury);

        vm.prank(buyer);
        escrow.completeDeal(dealId);

        // Verify token payouts
        P2PEscrow.Deal memory deal = escrow.getDeal(dealId);
        assertEq(uint(deal.state), uint(P2PEscrow.DealState.Completed));
        
        uint256 fee = deal.platformFee;
        uint256 sellerAmount = deal.amount - fee;
        
        assertEq(token.balanceOf(seller) - sellerBalanceBefore, sellerAmount);
        assertEq(token.balanceOf(platformTreasury) - treasuryBalanceBefore, fee);
    }

    // ========================================================================
    // Test: Admin Functions
    // ========================================================================

    function test_AddArbiter() public {
        address newArbiter = address(0xAA);
        
        vm.prank(platformTreasury);
        escrow.addArbiter(newArbiter);

        assertTrue(escrow.authorizedArbiters(newArbiter));
    }

    function test_RemoveArbiter() public {
        vm.prank(platformTreasury);
        escrow.removeArbiter(arbiter1);

        assertFalse(escrow.authorizedArbiters(arbiter1));
    }

    function test_SetTreasury() public {
        address newTreasury = address(0xBB);
        
        vm.prank(platformTreasury);
        escrow.setTreasury(newTreasury);

        assertEq(escrow.platformTreasury(), newTreasury);
    }

    function test_AddArbiter_OnlyTreasury() public {
        vm.prank(buyer);
        vm.expectRevert("Only treasury");
        escrow.addArbiter(address(0xCC));
    }

    // ========================================================================
    // Test: Statistics
    // ========================================================================

    function test_Statistics() public {
        // Create and complete multiple deals
        for (uint i = 0; i < 3; i++) {
            vm.prank(buyer);
            uint256 dealId = escrow.createAndFundDeal{value: 0.1 ether}(
                seller,
                P2PEscrow.DealType.DigitalGoods,
                block.timestamp + 7 days,
                keccak256(abi.encodePacked("deal ", i)),
                ""
            );

            vm.prank(buyer);
            escrow.confirmDelivery(dealId);

            vm.prank(buyer);
            escrow.completeDeal(dealId);
        }

        P2PEscrow.PlatformStats memory platformStats = escrow.getPlatformStats();
        
        assertEq(platformStats.totalDeals, 3);
        assertEq(platformStats.completedDeals, 3);
        assertGt(platformStats.totalFeesCollected, 0);
        assertGt(platformStats.totalVolume, 0);
    }

    // ========================================================================
    // Test: GetUserDeals
    // ========================================================================

    function test_GetUserDeals() public {
        vm.prank(buyer);
        uint256 deal1 = escrow.createDeal(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("deal1"),
            ""
        );

        vm.prank(buyer);
        uint256 deal2 = escrow.createDeal(
            seller,
            P2PEscrow.DealType.Service,
            block.timestamp + 7 days,
            keccak256("deal2"),
            ""
        );

        uint256[] memory buyerDeals = escrow.getUserDeals(buyer);
        assertGe(buyerDeals.length, 2);
        assertEq(buyerDeals[0], deal1);
        assertEq(buyerDeals[1], deal2);
    }

    // ========================================================================
    // Test: Edge Cases
    // ========================================================================

    function test_CannotFundNonExistentDeal() public {
        vm.expectRevert("Deal does not exist");
        vm.prank(buyer);
        escrow.fundDeal{value: 1 ether}(999);
    }

    function test_CannotFundAlreadyFundedDeal() public {
        vm.prank(buyer);
        uint256 dealId = escrow.createAndFundDeal{value: 1 ether}(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("funded"),
            ""
        );

        vm.prank(buyer);
        vm.expectRevert("Invalid deal state");
        escrow.fundDeal{value: 1 ether}(dealId);
    }

    function test_CannotCompleteUnfundedDeal() public {
        vm.prank(buyer);
        uint256 dealId = escrow.createDeal(
            seller,
            P2PEscrow.DealType.DigitalGoods,
            block.timestamp + 7 days,
            keccak256("unfunded"),
            ""
        );

        vm.prank(buyer);
        vm.expectRevert("Invalid deal state");
        escrow.completeDeal(dealId);
    }
}

// ========================================================================
// Mock ERC-20 Token
// ========================================================================

contract MockERC20 {
    string public name;
    string public symbol;
    uint8 public decimals;
    uint256 public totalSupply;
    
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;
    
    constructor(string memory _name, string memory _symbol, uint8 _decimals) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
    }
    
    function mint(address _to, uint256 _amount) external {
        balanceOf[_to] += _amount;
        totalSupply += _amount;
    }
    
    function transfer(address _to, uint256 _amount) external returns (bool) {
        balanceOf[msg.sender] -= _amount;
        balanceOf[_to] += _amount;
        return true;
    }
    
    function approve(address _spender, uint256 _amount) external returns (bool) {
        allowance[msg.sender][_spender] = _amount;
        return true;
    }
    
    function transferFrom(address _from, address _to, uint256 _amount) external returns (bool) {
        allowance[_from][msg.sender] -= _amount;
        balanceOf[_from] -= _amount;
        balanceOf[_to] += _amount;
        return true;
    }
}
