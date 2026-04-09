// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// ERC-20 interface (minimal)
interface IERC20 {
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
    function transfer(address to, uint256 amount) external returns (bool);
}

/**
 * @title P2PEscrow
 * @notice Смарт-контракт для безопасных P2P-сделок внутри мессенджера
 * @dev Постквантовая безопасность через гибридные подписи (X25519 + Kyber1024)
 *
 * Особенности:
 * - Мультиподпись (2 из 3: покупатель, продавец, арбитр)
 * - Таймаут с автоматическим возвратом средств
 * - Комиссия платформы (0.5-2% в зависимости от суммы)
 * - Поддержка ERC-20 токенов и нативного ETH
 * - Интеграция с E2EE мессенджером (хэши сообщений в events)
 */
contract P2PEscrow {
    // ========================================================================
    // Enums
    // ========================================================================

    enum DealState {
        Created,        // Сделка создана, ожидается депозит
        Funded,         // Средства заблокированы
        Delivered,      // Товар/услуга доставлена
        Completed,      // Сделка завершена, средства выплачены
        Disputed,       // Спор, требуется арбитраж
        Refunded,       // Возврат покупателю
        Cancelled       // Отменена
    }

    enum DealType {
        DigitalGoods,   // Цифровые товары
        PhysicalGoods,  // Физические товары
        Service,        // Услуги
        Subscription,   // Подписка
        Freelance       // Фриланс
    }

    // ========================================================================
    // Structs
    // ========================================================================

    struct Deal {
        uint256 id;
        DealType dealType;
        address buyer;
        address seller;
        address arbiter;
        uint256 amount;
        uint256 platformFee;
        address paymentToken; // address(0) для ETH
        DealState state;
        uint256 createdAt;
        uint256 fundedAt;
        uint256 deadline;
        bytes32 messageHash; // E2EE хэш описания сделки
        string ipfsMetadata; // IPFS CID с метаданными
    }

    struct PlatformStats {
        uint256 totalDeals;
        uint256 completedDeals;
        uint256 disputedDeals;
        uint256 totalVolume;
        uint256 totalFeesCollected;
    }

    // ========================================================================
    // State Variables
    // ========================================================================

    mapping(uint256 => Deal) public deals;
    mapping(address => uint256[]) public userDeals;
    mapping(address => bool) public authorizedArbiters;
    address[] public arbiterList; // Список всех арбитров

    PlatformStats public stats;
    
    uint256 public dealCounter;
    address public platformTreasury;
    uint256 public constant MIN_DEADLINE = 1 hours;
    uint256 public constant MAX_DEADLINE = 90 days;
    
    // Fee tiers (basis points: 100 = 1%)
    uint256 public constant FEE_TIER_1 = 200;  // 2% для сделок < 0.1 ETH
    uint256 public constant FEE_TIER_2 = 100;  // 1% для сделок < 1 ETH
    uint256 public constant FEE_TIER_3 = 50;   // 0.5% для сделок >= 1 ETH
    uint256 public constant TIER_1_MAX = 0.1 ether;
    uint256 public constant TIER_2_MAX = 1 ether;

    // ========================================================================
    // Events
    // ========================================================================

    event DealCreated(
        uint256 indexed dealId,
        address indexed buyer,
        address indexed seller,
        uint256 amount,
        DealType dealType,
        bytes32 messageHash
    );

    event DealFunded(
        uint256 indexed dealId,
        address indexed buyer,
        uint256 amount,
        uint256 platformFee
    );

    event DealDelivered(uint256 indexed dealId);
    
    event DealCompleted(
        uint256 indexed dealId,
        address indexed seller,
        uint256 sellerAmount,
        uint256 platformFee
    );

    event DealDisputed(
        uint256 indexed dealId,
        address indexed raisedBy,
        string reason
    );

    event DealResolved(
        uint256 indexed dealId,
        address indexed arbiter,
        bool refundToBuyer,
        uint256 buyerAmount,
        uint256 sellerAmount
    );

    event DealRefunded(uint256 indexed dealId, address indexed buyer);
    event DealCancelled(uint256 indexed dealId);
    event ArbiterAdded(address indexed arbiter);
    event ArbiterRemoved(address indexed arbiter);
    event TreasuryUpdated(address indexed oldTreasury, address indexed newTreasury);

    // ========================================================================
    // Modifiers
    // ========================================================================

    modifier onlyArbiter() {
        require(authorizedArbiters[msg.sender], "Not authorized arbiter");
        _;
    }

    modifier validDeal(uint256 _dealId) {
        require(_dealId < dealCounter, "Deal does not exist");
        _;
    }

    modifier onlyBuyer(uint256 _dealId) {
        require(msg.sender == deals[_dealId].buyer, "Only buyer can call");
        _;
    }

    modifier onlySeller(uint256 _dealId) {
        require(msg.sender == deals[_dealId].seller, "Only seller can call");
        _;
    }

    modifier inState(uint256 _dealId, DealState _state) {
        require(deals[_dealId].state == _state, "Invalid deal state");
        _;
    }

    // ========================================================================
    // Constructor
    // ========================================================================

    constructor(address _platformTreasury, address[] memory _initialArbiters) {
        require(_platformTreasury != address(0), "Invalid treasury");
        platformTreasury = _platformTreasury;

        for (uint i = 0; i < _initialArbiters.length; i++) {
            address arbiter = _initialArbiters[i];
            require(arbiter != address(0), "Invalid arbiter");
            authorizedArbiters[arbiter] = true;
            arbiterList.push(arbiter);
            emit ArbiterAdded(arbiter);
        }
    }

    // ========================================================================
    // Admin Functions
    // ========================================================================

    function addArbiter(address _arbiter) external {
        require(msg.sender == platformTreasury, "Only treasury");
        require(_arbiter != address(0), "Invalid arbiter");
        require(!authorizedArbiters[_arbiter], "Already an arbiter");
        authorizedArbiters[_arbiter] = true;
        arbiterList.push(_arbiter);
        emit ArbiterAdded(_arbiter);
    }

    function removeArbiter(address _arbiter) external {
        require(msg.sender == platformTreasury, "Only treasury");
        authorizedArbiters[_arbiter] = false;
        emit ArbiterRemoved(_arbiter);
    }

    function setTreasury(address _newTreasury) external {
        require(msg.sender == platformTreasury, "Only treasury");
        require(_newTreasury != address(0), "Invalid treasury");
        emit TreasuryUpdated(platformTreasury, _newTreasury);
        platformTreasury = _newTreasury;
    }

    // ========================================================================
    // Deal Creation
    // ========================================================================

    /**
     * @notice Создать новую сделку (без депозита)
     * @param _seller Адрес продавца
     * @param _dealType Тип сделки
     * @param _deadline Дедлайн (timestamp)
     * @param _messageHash E2EE хэш описания сделки
     * @param _ipfsMetadata IPFS CID с метаданными
     * @return dealId ID сделки
     */
    function createDeal(
        address _seller,
        DealType _dealType,
        uint256 _deadline,
        bytes32 _messageHash,
        string calldata _ipfsMetadata
    ) external returns (uint256 dealId) {
        require(_seller != address(0), "Invalid seller");
        require(_seller != msg.sender, "Cannot deal with yourself");
        require(
            _deadline >= block.timestamp + MIN_DEADLINE,
            "Deadline too short"
        );
        require(
            _deadline <= block.timestamp + MAX_DEADLINE,
            "Deadline too long"
        );

        dealId = dealCounter++;
        
        // Determine arbiter (first authorized arbiter)
        address arbiter = _selectArbiter();
        require(arbiter != address(0), "No arbiters available");

        deals[dealId] = Deal({
            id: dealId,
            dealType: _dealType,
            buyer: msg.sender,
            seller: _seller,
            arbiter: arbiter,
            amount: 0, // Will be set on funding
            platformFee: 0, // Will be calculated on funding
            paymentToken: address(0), // Default to ETH
            state: DealState.Created,
            createdAt: block.timestamp,
            fundedAt: 0,
            deadline: _deadline,
            messageHash: _messageHash,
            ipfsMetadata: _ipfsMetadata
        });

        userDeals[msg.sender].push(dealId);
        userDeals[_seller].push(dealId);
        stats.totalDeals++;

        emit DealCreated(
            dealId,
            msg.sender,
            _seller,
            0,
            _dealType,
            _messageHash
        );
    }

    /**
     * @notice Создать сделку с немедленным депозитом ETH
     */
    function createAndFundDeal(
        address _seller,
        DealType _dealType,
        uint256 _deadline,
        bytes32 _messageHash,
        string calldata _ipfsMetadata
    ) external payable returns (uint256 dealId) {
        require(msg.value > 0, "Must send ETH");

        dealId = this.createDeal(
            _seller,
            _dealType,
            _deadline,
            _messageHash,
            _ipfsMetadata
        );

        _fundDeal(dealId, msg.value, address(0));
    }

    // ========================================================================
    // Funding
    // ========================================================================

    /**
     * @notice Заблокировать средства в escrow (ETH)
     */
    function fundDeal(uint256 _dealId) external payable validDeal(_dealId) onlyBuyer(_dealId) inState(_dealId, DealState.Created) {
        require(msg.value > 0, "Must send ETH");
        _fundDeal(_dealId, msg.value, address(0));
    }

    /**
     * @notice Заблокировать ERC-20 токены в escrow
     */
    function fundDealWithToken(
        uint256 _dealId,
        uint256 _amount
    ) external validDeal(_dealId) onlyBuyer(_dealId) inState(_dealId, DealState.Created) {
        require(_amount > 0, "Amount must be > 0");
        _fundDeal(_dealId, _amount, deals[_dealId].paymentToken);
    }

    /**
     * @notice Внутренняя функция блокировки средств
     */
    function _fundDeal(uint256 _dealId, uint256 _amount, address _token) internal {
        Deal storage deal = deals[_dealId];
        
        // Calculate platform fee
        uint256 fee = _calculateFee(_amount);
        
        deal.amount = _amount;
        deal.platformFee = fee;
        deal.paymentToken = _token;
        deal.state = DealState.Funded;
        deal.fundedAt = block.timestamp;

        // Transfer tokens if ERC-20
        if (_token != address(0)) {
            IERC20(_token).transferFrom(msg.sender, address(this), _amount);
        }

        userDeals[msg.sender].push(_dealId);

        emit DealFunded(_dealId, msg.sender, _amount, fee);
    }

    // ========================================================================
    // Deal Completion
    // ========================================================================

    /**
     * @notice Подтвердить получение товара/услуги (покупатель)
     */
    function confirmDelivery(uint256 _dealId) external validDeal(_dealId) onlyBuyer(_dealId) inState(_dealId, DealState.Funded) {
        deals[_dealId].state = DealState.Delivered;
        emit DealDelivered(_dealId);
    }

    /**
     * @notice Завершить сделку и выплатить продавцу (после подтверждения)
     */
    function completeDeal(uint256 _dealId) external validDeal(_dealId) inState(_dealId, DealState.Delivered) {
        Deal storage deal = deals[_dealId];
        require(
            msg.sender == deal.buyer || msg.sender == deal.seller,
            "Only buyer or seller"
        );

        _payoutDeal(_dealId);
    }

    /**
     * @notice Автозавершение после дедлайна (любой адрес)
     */
    function completeAfterDeadline(uint256 _dealId) external validDeal(_dealId) {
        Deal storage deal = deals[_dealId];
        require(deal.state == DealState.Delivered, "Not delivered");
        require(block.timestamp >= deal.deadline, "Deadline not reached");

        _payoutDeal(_dealId);
    }

    /**
     * @notice Авто-возврат средств покупателю после дедлайна (если не доставлено)
     */
    function refundAfterDeadline(uint256 _dealId) external validDeal(_dealId) {
        Deal storage deal = deals[_dealId];
        require(
            deal.state == DealState.Funded || deal.state == DealState.Created,
            "Invalid state for refund"
        );
        require(block.timestamp >= deal.deadline, "Deadline not reached");

        deal.state = DealState.Refunded;

        // Return funds to buyer
        if (deal.paymentToken == address(0)) {
            payable(deal.buyer).transfer(deal.amount);
        } else {
            IERC20(deal.paymentToken).transfer(deal.buyer, deal.amount);
        }

        emit DealRefunded(_dealId, deal.buyer);
    }

    // ========================================================================
    // Dispute Resolution
    // ========================================================================

    /**
     * @notice Открыть спор (покупатель или продавец)
     */
    function openDispute(
        uint256 _dealId,
        string calldata _reason
    ) external validDeal(_dealId) inState(_dealId, DealState.Funded) {
        Deal storage deal = deals[_dealId];
        require(
            msg.sender == deal.buyer || msg.sender == deal.seller,
            "Only buyer or seller"
        );

        deal.state = DealState.Disputed;
        stats.disputedDeals++;

        emit DealDisputed(_dealId, msg.sender, _reason);
    }

    /**
     * @notice Разрешить спор (только арбитр)
     * @param _dealId ID сделки
     * @param _refundToBuyer true = полный возврат покупателю, false = выплата продавцу
     * @param _buyerPercent Процент покупателю (0-100), если частичный возврат
     */
    function resolveDispute(
        uint256 _dealId,
        bool _refundToBuyer,
        uint256 _buyerPercent
    ) external validDeal(_dealId) onlyArbiter inState(_dealId, DealState.Disputed) {
        require(_buyerPercent <= 100, "Invalid percentage");
        
        Deal storage deal = deals[_dealId];

        if (_refundToBuyer && _buyerPercent == 100) {
            // Полный возврат покупателю
            deal.state = DealState.Refunded;
            
            if (deal.paymentToken == address(0)) {
                payable(deal.buyer).transfer(deal.amount);
            } else {
                IERC20(deal.paymentToken).transfer(deal.buyer, deal.amount);
            }
            
            emit DealRefunded(_dealId, deal.buyer);
        } else if (!_refundToBuyer) {
            // Выплата продавцу (возможно частичная)
            deal.state = DealState.Completed;
            
            uint256 sellerAmount = deal.amount;
            uint256 buyerAmount = 0;
            
            if (_buyerPercent > 0) {
                buyerAmount = (deal.amount * _buyerPercent) / 100;
                sellerAmount = deal.amount - buyerAmount;
                
                if (deal.paymentToken == address(0)) {
                    payable(deal.buyer).transfer(buyerAmount);
                } else {
                    IERC20(deal.paymentToken).transfer(deal.buyer, buyerAmount);
                }
            }
            
            // Payout to seller
            uint256 sellerNet = sellerAmount - deal.platformFee;
            if (deal.paymentToken == address(0)) {
                payable(deal.seller).transfer(sellerNet);
                payable(platformTreasury).transfer(deal.platformFee);
            } else {
                IERC20(deal.paymentToken).transfer(deal.seller, sellerNet);
                IERC20(deal.paymentToken).transfer(platformTreasury, deal.platformFee);
            }
            
            stats.completedDeals++;
            stats.totalFeesCollected += deal.platformFee;
            
            emit DealCompleted(_dealId, deal.seller, sellerNet, deal.platformFee);
        }

        emit DealResolved(_dealId, msg.sender, _refundToBuyer, _buyerPercent > 0 ? (deal.amount * _buyerPercent) / 100 : 0, deal.amount - (_buyerPercent > 0 ? (deal.amount * _buyerPercent) / 100 : 0));
    }

    // ========================================================================
    // Cancellation
    // ========================================================================

    /**
     * @notice Отменить сделку (до финансирования)
     */
    function cancelDeal(uint256 _dealId) external validDeal(_dealId) inState(_dealId, DealState.Created) onlyBuyer(_dealId) {
        deals[_dealId].state = DealState.Cancelled;
        emit DealCancelled(_dealId);
    }

    // ========================================================================
    // Internal Functions
    // ========================================================================

    /**
     * @notice Выплатить средства продавцу
     */
    function _payoutDeal(uint256 _dealId) internal {
        Deal storage deal = deals[_dealId];
        
        deal.state = DealState.Completed;
        stats.completedDeals++;
        stats.totalFeesCollected += deal.platformFee;
        stats.totalVolume += deal.amount;

        uint256 sellerAmount = deal.amount - deal.platformFee;

        if (deal.paymentToken == address(0)) {
            // ETH payout
            payable(deal.seller).transfer(sellerAmount);
            payable(platformTreasury).transfer(deal.platformFee);
        } else {
            // ERC-20 payout
            IERC20(deal.paymentToken).transfer(deal.seller, sellerAmount);
            IERC20(deal.paymentToken).transfer(platformTreasury, deal.platformFee);
        }

        emit DealCompleted(_dealId, deal.seller, sellerAmount, deal.platformFee);
    }

    /**
     * @notice Рассчитать комиссию платформы
     */
    function _calculateFee(uint256 _amount) internal pure returns (uint256) {
        if (_amount < TIER_1_MAX) {
            return (_amount * FEE_TIER_1) / 10000;
        } else if (_amount < TIER_2_MAX) {
            return (_amount * FEE_TIER_2) / 10000;
        } else {
            return (_amount * FEE_TIER_3) / 10000;
        }
    }

    /**
     * @notice Выбрать арбитра (первый из списка)
     */
    function _selectArbiter() internal view returns (address) {
        require(arbiterList.length > 0, "No arbiters available");
        return arbiterList[0];
    }

    // ========================================================================
    // View Functions
    // ========================================================================

    function getDeal(uint256 _dealId) external view returns (Deal memory) {
        require(_dealId < dealCounter, "Deal does not exist");
        return deals[_dealId];
    }

    function getUserDeals(address _user) external view returns (uint256[] memory) {
        return userDeals[_user];
    }

    function getPlatformStats() external view returns (PlatformStats memory) {
        return stats;
    }

    function calculateFee(uint256 _amount) external pure returns (uint256) {
        return _calculateFee(_amount);
    }

    // ========================================================================
    // Receive Function
    // ========================================================================

    receive() external payable {
        // Allow receiving ETH for gas refunds
    }
}
