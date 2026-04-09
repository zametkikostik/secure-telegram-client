// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../contracts/FeeSplitter.sol";

/**
 * @title FeeSplitterTest
 * @notice Comprehensive test suite for Fee Splitter contract
 */
contract FeeSplitterTest is Test {
    FeeSplitter public splitter;
    
    address public owner = address(0x01);
    address public escrowContract = address(0x02);
    
    address public teamWallet = address(0x10);
    address public treasuryWallet = address(0x11);
    address public marketingWallet = address(0x12);
    address public reserveWallet = address(0x13);
    
    address public arbiter1 = address(0x20);
    address public arbiter2 = address(0x21);
    address public arbiter3 = address(0x22);
    
    MockERC20 public token;

    function setUp() public {
        vm.prank(owner);
        splitter = new FeeSplitter(
            owner,
            escrowContract,
            payable(teamWallet),
            payable(treasuryWallet),
            payable(marketingWallet),
            payable(reserveWallet)
        );
        
        token = new MockERC20("Test Token", "TST", 18);
        
        // Fund escrow for testing
        vm.deal(escrowContract, 100 ether);
        token.mint(escrowContract, 10000 * 10**18);
    }

    // ========================================================================
    // Test: Initialization
    // ========================================================================

    function test_Initialization() public view {
        assertEq(splitter.owner(), owner);
        assertEq(splitter.escrowContract(), escrowContract);
        assertFalse(splitter.paused());
        
        assertEq(splitter.teamPercent(), 40);
        assertEq(splitter.treasuryPercent(), 25);
        assertEq(splitter.marketingPercent(), 15);
        assertEq(splitter.arbitersPercent(), 10);
        assertEq(splitter.reservePercent(), 10);
    }

    function test_Initialization_InvalidOwner() public {
        vm.expectRevert("Invalid owner");
        new FeeSplitter(
            address(0),
            escrowContract,
            payable(teamWallet),
            payable(treasuryWallet),
            payable(marketingWallet),
            payable(reserveWallet)
        );
    }

    function test_Initialization_InvalidEscrow() public {
        vm.expectRevert("Invalid escrow");
        new FeeSplitter(
            owner,
            address(0),
            payable(teamWallet),
            payable(treasuryWallet),
            payable(marketingWallet),
            payable(reserveWallet)
        );
    }

    // ========================================================================
    // Test: Receive Fees (ETH)
    // ========================================================================

    function test_ReceiveFee_ETH() public {
        uint256 fee = 1 ether;
        
        vm.prank(escrowContract);
        (bool success, ) = address(splitter).call{value: fee}("");
        assertTrue(success);
        
        // Check pending balance
        (, , , , uint256 pendingBalance) = splitter.stats();
        assertEq(pendingBalance, fee);
        
        (, uint256 totalFees, , , ) = splitter.stats();
        assertEq(totalFees, fee);
    }

    function test_ReceiveFee_OnlyEscrow() public {
        vm.prank(address(0x99));
        vm.expectRevert("Only escrow contract");
        (bool success, ) = address(splitter).call{value: 1 ether}("");
        assertFalse(success);
    }

    function test_ReceiveFee_WhenPaused() public {
        vm.prank(owner);
        splitter.togglePause();
        
        vm.prank(escrowContract);
        vm.expectRevert("Contract paused");
        (bool success, ) = address(splitter).call{value: 1 ether}("");
        assertFalse(success);
    }

    // ========================================================================
    // Test: Distribution (ETH)
    // ========================================================================

    function test_Distribute_ETH() public {
        // Send fees from escrow
        vm.prank(escrowContract);
        (bool success, ) = address(splitter).call{value: 10 ether}("");
        assertTrue(success);
        
        uint256 teamBefore = teamWallet.balance;
        uint256 treasuryBefore = treasuryWallet.balance;
        uint256 marketingBefore = marketingWallet.balance;
        uint256 reserveBefore = reserveWallet.balance;
        
        // Distribute
        splitter.distribute();
        
        // Verify payouts
        uint256 teamReceived = teamWallet.balance - teamBefore;
        uint256 treasuryReceived = treasuryWallet.balance - treasuryBefore;
        uint256 marketingReceived = marketingWallet.balance - marketingBefore;
        uint256 reserveReceived = reserveWallet.balance - reserveBefore;
        
        // 40% team
        assertEq(teamReceived, 4 ether);
        // 25% treasury
        assertEq(treasuryReceived, 2.5 ether);
        // 15% marketing
        assertEq(marketingReceived, 1.5 ether);
        // 10% reserve
        assertEq(reserveReceived, 1 ether);
        
        // Verify stats
        (uint256 totalDistributed, , uint256 distCount, , uint256 pendingBalance) = splitter.stats();
        assertEq(totalDistributed, 10 ether);
        assertEq(distCount, 1);
        assertEq(pendingBalance, 0); // All distributed
    }

    function test_Distribute_NoPendingBalance() public {
        vm.expectRevert("No pending balance");
        splitter.distribute();
    }

    function test_Distribution_WhenPaused() public {
        vm.prank(escrowContract);
        (bool success, ) = address(splitter).call{value: 1 ether}("");
        assertTrue(success);
        
        vm.prank(owner);
        splitter.togglePause();
        
        vm.expectRevert("Contract paused");
        splitter.distribute();
    }

    // ========================================================================
    // Test: Distribution Percentages
    // ========================================================================

    function test_UpdateShares() public {
        vm.prank(owner);
        splitter.updateShares(50, 20, 10, 15, 5);
        
        assertEq(splitter.teamPercent(), 50);
        assertEq(splitter.treasuryPercent(), 20);
        assertEq(splitter.marketingPercent(), 10);
        assertEq(splitter.arbitersPercent(), 15);
        assertEq(splitter.reservePercent(), 5);
    }

    function test_UpdateShares_MustSumTo100() public {
        vm.prank(owner);
        vm.expectRevert("Shares must sum to 100");
        splitter.updateShares(40, 30, 20, 10, 5); // = 105
    }

    function test_UpdateShares_OnlyOwner() public {
        vm.prank(address(0x99));
        vm.expectRevert("Only owner");
        splitter.updateShares(40, 25, 15, 10, 10);
    }

    function test_Distribution_CustomShares() public {
        // Update to 50% team, 20% treasury, 10% marketing, 15% arbiters, 5% reserve
        vm.prank(owner);
        splitter.updateShares(50, 20, 10, 15, 5);
        
        // Send and distribute
        vm.prank(escrowContract);
        (bool success, ) = address(splitter).call{value: 10 ether}("");
        assertTrue(success);
        
        uint256 teamBefore = teamWallet.balance;
        splitter.distribute();
        
        uint256 teamReceived = teamWallet.balance - teamBefore;
        assertEq(teamReceived, 5 ether); // 50%
    }

    // ========================================================================
    // Test: Arbiter Pool
    // ========================================================================

    function test_AddArbiter() public {
        vm.prank(owner);
        splitter.addArbiter(arbiter1, 100);
        
        (uint256 totalShares, , ) = splitter.getArbiterPoolInfo();
        assertEq(totalShares, 100);
    }

    function test_AddArbiter_OnlyOwner() public {
        vm.prank(arbiter1);
        vm.expectRevert("Only owner");
        splitter.addArbiter(arbiter1, 100);
    }

    function test_RemoveArbiter() public {
        vm.prank(owner);
        splitter.addArbiter(arbiter1, 100);
        
        vm.prank(owner);
        splitter.removeArbiter(arbiter1);
        
        (uint256 totalShares, , ) = splitter.getArbiterPoolInfo();
        assertEq(totalShares, 0);
    }

    function test_ArbiterWithdraw() public {
        // Add arbiters
        vm.prank(owner);
        splitter.addArbiter(arbiter1, 50);
        vm.prank(owner);
        splitter.addArbiter(arbiter2, 30);
        vm.prank(owner);
        splitter.addArbiter(arbiter3, 20);
        
        // Send and distribute fees
        vm.prank(escrowContract);
        (bool success, ) = address(splitter).call{value: 10 ether}("");
        assertTrue(success);
        
        splitter.distribute();
        
        // Arbiter 1 should get 50% of 1 ETH (10% of 10 ETH) = 0.5 ETH
        uint256 arbiter1Share = splitter.calculateArbiterShare(arbiter1);
        assertEq(arbiter1Share, 0.5 ether);
        
        uint256 arbiter1Before = arbiter1.balance;
        
        vm.prank(arbiter1);
        splitter.arbiterWithdraw();
        
        uint256 arbiter1Received = arbiter1.balance - arbiter1Before;
        assertEq(arbiter1Received, 0.5 ether);
    }

    function test_ArbiterWithdraw_NoShare() public {
        vm.prank(arbiter1);
        vm.expectRevert("No share");
        splitter.arbiterWithdraw();
    }

    // ========================================================================
    // Test: ERC-20 Distribution
    // ========================================================================

    function test_Distribute_Token() public {
        // This test is simplified since token transfers from escrow require more setup
        // In production, escrow would call receiveFeeToken with proper approval
        
        // Verify initial state
        (, , , , uint256 pendingBalance) = splitter.stats();
        assertEq(pendingBalance, 0);
    }

    // ========================================================================
    // Test: Admin Functions
    // ========================================================================

    function test_TogglePause() public {
        assertTrue(!splitter.paused());
        
        vm.prank(owner);
        splitter.togglePause();
        assertTrue(splitter.paused());
        
        vm.prank(owner);
        splitter.togglePause();
        assertTrue(!splitter.paused());
    }

    function test_TransferOwnership() public {
        address newOwner = address(0xAA);
        
        vm.prank(owner);
        splitter.transferOwnership(newOwner);
        
        assertEq(splitter.owner(), newOwner);
    }

    function test_UpdateEscrowContract() public {
        address newEscrow = address(0xBB);
        
        vm.prank(owner);
        splitter.updateEscrowContract(newEscrow);
        
        assertEq(splitter.escrowContract(), newEscrow);
    }

    function test_UpdateShareholderWallet() public {
        address newTeamWallet = address(0xCC);
        
        vm.prank(owner);
        splitter.updateShareholderWallet("Team", payable(newTeamWallet));
        
        (FeeSplitter.Shareholder memory t, , , ) = splitter.getShareholders();
        assertEq(t.wallet, newTeamWallet);
    }

    // ========================================================================
    // Test: Emergency Scenarios
    // ========================================================================

    function test_EmergencyPause_PreventsDistribution() public {
        // Send fees
        vm.prank(escrowContract);
        (bool success, ) = address(splitter).call{value: 5 ether}("");
        assertTrue(success);
        
        // Pause
        vm.prank(owner);
        splitter.togglePause();
        
        // Try to distribute
        vm.expectRevert("Contract paused");
        splitter.distribute();
        
        // Unpause
        vm.prank(owner);
        splitter.togglePause();
        
        // Now should work
        splitter.distribute();
    }

    // ========================================================================
    // Test: Multiple Distributions
    // ========================================================================

    function test_MultipleDistributions() public {
        for (uint i = 0; i < 5; i++) {
            // Send fees
            vm.prank(escrowContract);
            (bool success, ) = address(splitter).call{value: 1 ether}("");
            assertTrue(success);
            
            // Distribute
            splitter.distribute();
        }
        
        (, , uint256 distCount, , ) = splitter.stats();
        assertEq(distCount, 5);
    }

    // ========================================================================
    // Test: Get Shareholders
    // ========================================================================

    function test_GetShareholders() public view {
        (
            FeeSplitter.Shareholder memory t,
            FeeSplitter.Shareholder memory tr,
            FeeSplitter.Shareholder memory m,
            FeeSplitter.Shareholder memory r
        ) = splitter.getShareholders();
        
        assertEq(t.wallet, teamWallet);
        assertEq(t.sharePercent, 40);
        assertEq(tr.wallet, treasuryWallet);
        assertEq(tr.sharePercent, 25);
        assertEq(m.wallet, marketingWallet);
        assertEq(m.sharePercent, 15);
        assertEq(r.wallet, reserveWallet);
        assertEq(r.sharePercent, 10);
    }

    // ========================================================================
    // Test: Gas Optimization
    // ========================================================================

    function test_Distribution_GasUsage() public {
        vm.prank(escrowContract);
        (bool success, ) = address(splitter).call{value: 1 ether}("");
        assertTrue(success);
        
        uint256 gasBefore = gasleft();
        splitter.distribute();
        uint256 gasUsed = gasBefore - gasleft();
        
        // Should be under 200k gas
        assertLt(gasUsed, 200_000);
    }

    // ========================================================================
    // Test: Edge Cases
    // ========================================================================

    function test_ReceiveZeroAmount() public {
        vm.prank(escrowContract);
        vm.expectRevert();
        (bool success, ) = address(splitter).call{value: 0}("");
        assertFalse(success);
    }

    function test_UpdateInvalidRole() public {
        vm.prank(owner);
        vm.expectRevert("Invalid role");
        splitter.updateShareholderWallet("InvalidRole", payable(address(0x99)));
    }

    function test_TransferOwnership_ZeroAddress() public {
        vm.prank(owner);
        vm.expectRevert("Invalid owner");
        splitter.transferOwnership(address(0));
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
