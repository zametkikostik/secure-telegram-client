// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract P2PEscrow {
    address public admin;
    uint256 public adminFeePercent = 30; // 3%
    
    struct Deal {
        address buyer;
        address seller;
        uint256 amount;
        bool paid;
        bool released;
    }
    
    mapping(uint256 => Deal) public deals;
    uint256 public dealCounter;
    
    constructor(address _admin) {
        admin = _admin;
    }
    
    function createDeal(address seller) external payable returns (uint256) {
        dealCounter++;
        deals[dealCounter] = Deal({
            buyer: msg.sender,
            seller: seller,
            amount: msg.value,
            paid: false,
            released: false
        });
        return dealCounter;
    }
    
    function confirmPayment(uint256 dealId) external {
        Deal storage deal = deals[dealId];
        require(msg.sender == deal.buyer);
        require(!deal.paid);
        deal.paid = true;
    }
    
    function releaseCrypto(uint256 dealId) external {
        Deal storage deal = deals[dealId];
        require(msg.sender == deal.seller);
        require(deal.paid);
        require(!deal.released);
        
        uint256 adminFee = (deal.amount * adminFeePercent) / 10000;
        uint256 sellerAmount = deal.amount - adminFee;
        
        (bool success,) = admin.call{value: adminFee}("");
        require(success, "Admin transfer failed");
        
        (success,) = deal.seller.call{value: sellerAmount}("");
        require(success, "Seller transfer failed");
        
        deal.released = true;
    }
}
