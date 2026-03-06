// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title FeeSplitter
 * @dev Распределение комиссий между администраторами
 */
contract FeeSplitter {
    address public owner;
    
    struct Recipient {
        address wallet;
        uint256 percentage; // В базисных пунктах (10000 = 100%)
    }
    
    Recipient[] public recipients;
    uint256 public totalPercentage;
    
    event FeeReceived(address indexed token, uint256 amount);
    event FeeDistributed(address indexed recipient, uint256 amount);
    event RecipientAdded(address indexed wallet, uint256 percentage);
    event RecipientRemoved(address indexed wallet);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }
    
    constructor(address _owner) {
        owner = _owner;
    }
    
    function addRecipient(address wallet, uint256 percentage) external onlyOwner {
        require(totalPercentage + percentage <= 10000, "Percentage overflow");
        
        recipients.push(Recipient({
            wallet: wallet,
            percentage: percentage
        }));
        
        totalPercentage += percentage;
        
        emit RecipientAdded(wallet, percentage);
    }
    
    function removeRecipient(uint256 index) external onlyOwner {
        require(index < recipients.length, "Invalid index");
        
        totalPercentage -= recipients[index].percentage;
        
        recipients[index] = recipients[recipients.length - 1];
        recipients.pop();
        
        emit RecipientRemoved(recipients[index].wallet);
    }
    
    function distributeFee(address token, uint256 amount) internal {
        require(totalPercentage > 0, "No recipients");
        
        for (uint256 i = 0; i < recipients.length; i++) {
            Recipient storage recipient = recipients[i];
            uint256 share = (amount * recipient.percentage) / 10000;
            
            // Transfer tokens (ERC20)
            if (token != address(0)) {
                // IERC20(token).transfer(recipient.wallet, share);
            } else {
                // Transfer ETH
                (bool success, ) = recipient.wallet.call{value: share}("");
                require(success, "Transfer failed");
            }
            
            emit FeeDistributed(recipient.wallet, share);
        }
    }
    
    receive() external payable {
        emit FeeReceived(address(0), msg.value);
        distributeFee(address(0), msg.value);
    }
    
    function withdraw() external onlyOwner {
        (bool success, ) = owner.call{value: address(this).balance}("");
        require(success, "Withdraw failed");
    }
}
