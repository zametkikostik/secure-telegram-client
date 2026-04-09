// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

// ERC-20 interface
interface IERC20External {
    function transfer(address _to, uint256 _amount) external returns (bool);
    function transferFrom(address _from, address _to, uint256 _amount) external returns (bool);
    function balanceOf(address _account) external view returns (uint256);
}

/**
 * @title FeeSplitter
 * @notice Автоматическое распределение комиссий между участниками платформы
 * @dev Интегрируется с P2PEscrow для распределения platform fees
 *
 * Распределение по умолчанию:
 * - 40% → Команда разработки (Team)
 * - 25% → Treasury платформы (Platform Treasury)
 * - 15% → Маркетинг и рост (Marketing)
 * - 10% → Арбитры (Arbiters Pool)
 * - 10% → Резервный фонд (Reserve Fund)
 *
 * Особенности:
 * - Настраиваемые проценты (сумма = 100%)
 * - Мгновенное распределение при вызове distribute()
 * - Поддержка ETH и ERC-20 токенов
 * - Автоматическое начисление арбитрам по активности
 * - Emergency pause (только owner)
 * - Прозрачная статистика
 */
contract FeeSplitter {
    // ========================================================================
    // Structs
    // ========================================================================

    /// Информация о получателе (shareholder)
    struct Shareholder {
        address payable wallet;
        uint256 sharePercent;    // Процент (0-100)
        string role;             // Роль (team, marketing, etc)
        bool isActive;
        uint256 totalReceived;   // Всего получено (wei)
        uint256 lastClaimedAt;   // Время последнего вывода
    }

    /// Статистика распределения
    struct DistributionStats {
        uint256 totalDistributed;
        uint256 totalFeesReceived;
        uint256 distributionCount;
        uint256 lastDistributionAt;
        uint256 pendingBalance;
    }

    /// Информация о пуле арбитров
    struct ArbiterPool {
        mapping(address => uint256) arbiterShares;  // Доля арбитра (репутация)
        uint256 totalShares;
        uint256 totalBalance;
        mapping(address => uint256) pendingWithdrawals;
    }

    // ========================================================================
    // State Variables
    // ========================================================================

    /// Владелец контракта (Platform Admin)
    address public owner;
    
    /// P2PEscrow контракт (авторизованный источник комиссий)
    address public escrowContract;
    
    /// Пауза (emergency stop)
    bool public paused;
    
    /// Минимальная сумма для вывода (gas optimization)
    uint256 public constant MIN_WITHDRAWAL = 0.001 ether;

    // Доли получателей (в процентах, сумма = 100)
    uint256 public teamPercent;
    uint256 public treasuryPercent;
    uint256 public marketingPercent;
    uint256 public arbitersPercent;
    uint256 public reservePercent;

    // Получатели
    Shareholder public team;
    Shareholder public treasury;
    Shareholder public marketing;
    Shareholder public reserve;
    
    /// Пул арбитров
    ArbiterPool public arbiterPool;
    
    /// Статистика
    DistributionStats public stats;
    
    /// Балансы токенов (token address => balance)
    mapping(address => uint256) public tokenBalances;
    
    /// Лог распределений
    struct DistributionLog {
        uint256 timestamp;
        uint256 amount;
        address token;
        address[] recipients;
        uint256[] amounts;
    }
    
    DistributionLog[] public distributionHistory;
    
    // ========================================================================
    // Events
    // ========================================================================

    event FeeReceived(
        uint256 amount,
        address token,
        address from
    );

    event Distributed(
        uint256 indexed timestamp,
        uint256 totalAmount,
        address token,
        uint256 teamAmount,
        uint256 treasuryAmount,
        uint256 marketingAmount,
        uint256 arbitersAmount,
        uint256 reserveAmount
    );

    event ShareholderUpdated(
        string role,
        address wallet,
        uint256 sharePercent
    );

    event ArbiterAdded(address indexed arbiter, uint256 share);
    event ArbiterRemoved(address indexed arbiter, uint256 share);
    event ArbiterWithdrawal(address indexed arbiter, uint256 amount);
    
    event EmergencyPause(bool paused, address triggeredBy, uint256 timestamp);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event EscrowContractUpdated(address indexed oldEscrow, address indexed newEscrow);

    // ========================================================================
    // Modifiers
    // ========================================================================

    modifier onlyOwner() {
        require(msg.sender == owner, "Only owner");
        _;
    }

    modifier onlyEscrow() {
        require(msg.sender == escrowContract, "Only escrow contract");
        _;
    }

    modifier notPaused() {
        require(!paused, "Contract paused");
        _;
    }

    modifier validShares(uint256 _team, uint256 _treasury, uint256 _marketing, uint256 _arbiters, uint256 _reserve) {
        require(
            _team + _treasury + _marketing + _arbiters + _reserve == 100,
            "Shares must sum to 100"
        );
        _;
    }

    // ========================================================================
    // Constructor
    // ========================================================================

    /**
     * @notice Инициализировать FeeSplitter
     * @param _owner Адрес владельца платформы
     * @param _escrowContract Адрес P2PEscrow контракта
     * @param _teamWallet Адрес кошелька команды разработки
     * @param _treasuryWallet Адрес казны платформы
     * @param _marketingWallet Адрес маркетингового кошелька
     * @param _reserveWallet Адрес резервного фонда
     */
    constructor(
        address _owner,
        address _escrowContract,
        address payable _teamWallet,
        address payable _treasuryWallet,
        address payable _marketingWallet,
        address payable _reserveWallet
    ) {
        require(_owner != address(0), "Invalid owner");
        require(_escrowContract != address(0), "Invalid escrow");
        
        owner = _owner;
        escrowContract = _escrowContract;
        
        // По умолчанию: 40/25/15/10/10
        teamPercent = 40;
        treasuryPercent = 25;
        marketingPercent = 15;
        arbitersPercent = 10;
        reservePercent = 10;
        
        team = Shareholder(_teamWallet, 40, "Team", true, 0, 0);
        treasury = Shareholder(_treasuryWallet, 25, "Treasury", true, 0, 0);
        marketing = Shareholder(_marketingWallet, 15, "Marketing", true, 0, 0);
        reserve = Shareholder(_reserveWallet, 10, "Reserve", true, 0, 0);
        
        emit ShareholderUpdated("Team", _teamWallet, 40);
        emit ShareholderUpdated("Treasury", _treasuryWallet, 25);
        emit ShareholderUpdated("Marketing", _marketingWallet, 15);
        emit ShareholderUpdated("Reserve", _reserveWallet, 10);
    }

    // ========================================================================
    // Core Functions
    // ========================================================================

    /**
     * @notice Получить комиссию от P2PEscrow (ETH)
     */
    receive() external payable onlyEscrow notPaused {
        _receiveFee(msg.value, address(0));
    }

    /**
     * @notice Получить комиссию от P2PEscrow (вызов из escrow)
     */
    function receiveFee() external payable onlyEscrow notPaused {
        require(msg.value > 0, "Amount must be > 0");
        _receiveFee(msg.value, address(0));
    }

    /**
     * @notice Получить комиссию в ERC-20 токенах
     */
    function receiveFeeToken(address _token, uint256 _amount) external onlyEscrow notPaused {
        require(_amount > 0, "Amount must be > 0");
        require(_token != address(0), "Invalid token");
        
        // Transfer tokens from escrow to this contract
        IERC20External(_token).transferFrom(_token == address(0) ? msg.sender : msg.sender, address(this), _amount);
        
        _receiveFee(_amount, _token);
    }

    /**
     * @notice Внутренняя функция получения комиссии
     */
    function _receiveFee(uint256 _amount, address _token) internal {
        stats.totalFeesReceived += _amount;
        stats.pendingBalance += _amount;
        
        if (_token != address(0)) {
            tokenBalances[_token] += _amount;
        }
        
        emit FeeReceived(_amount, _token, msg.sender);
    }

    /**
     * @notice Распределить накопленные комиссии (ETH)
     * @dev Вызывается любым адресом (gas оплачивает вызывающий)
     */
    function distribute() external notPaused {
        require(stats.pendingBalance > 0, "No pending balance");
        
        uint256 totalAmount = address(this).balance;
        require(totalAmount >= stats.pendingBalance, "Insufficient balance");
        
        // Calculate shares
        uint256 teamAmount = (totalAmount * teamPercent) / 100;
        uint256 treasuryAmount = (totalAmount * treasuryPercent) / 100;
        uint256 marketingAmount = (totalAmount * marketingPercent) / 100;
        uint256 arbitersAmount = (totalAmount * arbitersPercent) / 100;
        uint256 reserveAmount = (totalAmount * reservePercent) / 100;
        
        // Update shareholder stats
        team.totalReceived += teamAmount;
        team.lastClaimedAt = block.timestamp;
        
        treasury.totalReceived += treasuryAmount;
        treasury.lastClaimedAt = block.timestamp;
        
        marketing.totalReceived += marketingAmount;
        marketing.lastClaimedAt = block.timestamp;
        
        reserve.totalReceived += reserveAmount;
        reserve.lastClaimedAt = block.timestamp;
        
        // Update arbiter pool
        arbiterPool.totalBalance += arbitersAmount;
        
        // Transfer funds
        if (team.sharePercent > 0 && team.isActive) {
            _safeTransfer(team.wallet, teamAmount);
        }
        
        if (treasury.sharePercent > 0 && treasury.isActive) {
            _safeTransfer(treasury.wallet, treasuryAmount);
        }
        
        if (marketing.sharePercent > 0 && marketing.isActive) {
            _safeTransfer(marketing.wallet, marketingAmount);
        }
        
        if (reserve.sharePercent > 0 && reserve.isActive) {
            _safeTransfer(reserve.wallet, reserveAmount);
        }
        
        // Update stats
        stats.totalDistributed += totalAmount;
        stats.distributionCount++;
        stats.lastDistributionAt = block.timestamp;
        stats.pendingBalance = 0;
        
        // Log distribution
        address[] memory recipients = new address[](4);
        recipients[0] = team.wallet;
        recipients[1] = treasury.wallet;
        recipients[2] = marketing.wallet;
        recipients[3] = reserve.wallet;
        
        uint256[] memory amounts = new uint256[](4);
        amounts[0] = teamAmount;
        amounts[1] = treasuryAmount;
        amounts[2] = marketingAmount;
        amounts[3] = reserveAmount;
        
        distributionHistory.push(DistributionLog({
            timestamp: block.timestamp,
            amount: totalAmount,
            token: address(0),
            recipients: recipients,
            amounts: amounts
        }));
        
        emit Distributed(
            block.timestamp,
            totalAmount,
            address(0),
            teamAmount,
            treasuryAmount,
            marketingAmount,
            arbitersAmount,
            reserveAmount
        );
    }

    /**
     * @notice Распределить ERC-20 токены
     */
    function distributeToken(address _token) external notPaused {
        require(_token != address(0), "Invalid token");
        uint256 balance = tokenBalances[_token];
        require(balance > 0, "No pending balance");
        
        // Calculate shares
        uint256 teamAmount = (balance * teamPercent) / 100;
        uint256 treasuryAmount = (balance * treasuryPercent) / 100;
        uint256 marketingAmount = (balance * marketingPercent) / 100;
        uint256 arbitersAmount = (balance * arbitersPercent) / 100;
        uint256 reserveAmount = (balance * reservePercent) / 100;
        
        // Update stats
        team.totalReceived += teamAmount;
        treasury.totalReceived += treasuryAmount;
        marketing.totalReceived += marketingAmount;
        reserve.totalReceived += reserveAmount;
        arbiterPool.totalBalance += arbitersAmount;
        
        // Transfer tokens
        if (team.isActive) IERC20External(_token).transfer(team.wallet, teamAmount);
        if (treasury.isActive) IERC20External(_token).transfer(treasury.wallet, treasuryAmount);
        if (marketing.isActive) IERC20External(_token).transfer(marketing.wallet, marketingAmount);
        if (reserve.isActive) IERC20External(_token).transfer(reserve.wallet, reserveAmount);
        
        // Update balances
        tokenBalances[_token] = 0;
        stats.totalDistributed += balance;
        stats.distributionCount++;
        stats.lastDistributionAt = block.timestamp;
        
        emit Distributed(
            block.timestamp,
            balance,
            _token,
            teamAmount,
            treasuryAmount,
            marketingAmount,
            arbitersAmount,
            reserveAmount
        );
    }

    // ========================================================================
    // Arbiter Pool Functions
    // ========================================================================

    /**
     * @notice Добавить арбитра в пул
     */
    function addArbiter(address _arbiter, uint256 _share) external onlyOwner {
        require(_arbiter != address(0), "Invalid arbiter");
        require(_share > 0, "Share must be > 0");
        
        if (arbiterPool.arbiterShares[_arbiter] == 0) {
            arbiterPool.totalShares += _share;
        } else {
            arbiterPool.totalShares = arbiterPool.totalShares - arbiterPool.arbiterShares[_arbiter] + _share;
        }
        
        arbiterPool.arbiterShares[_arbiter] = _share;
        
        emit ArbiterAdded(_arbiter, _share);
    }

    /**
     * @notice Удалить арбитра из пула
     */
    function removeArbiter(address _arbiter) external onlyOwner {
        require(_arbiter != address(0), "Invalid arbiter");
        
        uint256 share = arbiterPool.arbiterShares[_arbiter];
        require(share > 0, "Arbiter not in pool");
        
        // Allow arbiter to withdraw pending before removal
        if (arbiterPool.pendingWithdrawals[_arbiter] > 0) {
            uint256 amount = arbiterPool.pendingWithdrawals[_arbiter];
            arbiterPool.pendingWithdrawals[_arbiter] = 0;
            arbiterPool.totalBalance -= amount;
            payable(_arbiter).transfer(amount);
            emit ArbiterWithdrawal(_arbiter, amount);
        }
        
        arbiterPool.totalShares -= share;
        arbiterPool.arbiterShares[_arbiter] = 0;
        
        emit ArbiterRemoved(_arbiter, share);
    }

    /**
     * @notice Рассчитать долю арбитра
     */
    function calculateArbiterShare(address _arbiter) public view returns (uint256) {
        if (arbiterPool.totalShares == 0 || arbiterPool.arbiterShares[_arbiter] == 0) {
            return 0;
        }
        
        return (arbiterPool.totalBalance * arbiterPool.arbiterShares[_arbiter]) / arbiterPool.totalShares;
    }

    /**
     * @notice Вывести средства из пула арбитров
     */
    function arbiterWithdraw() external {
        uint256 share = calculateArbiterShare(msg.sender);
        require(share > 0, "No share");
        require(share >= MIN_WITHDRAWAL, "Below minimum");
        
        // Update pending
        arbiterPool.pendingWithdrawals[msg.sender] = 0;
        arbiterPool.totalBalance -= share;
        
        payable(msg.sender).transfer(share);
        
        emit ArbiterWithdrawal(msg.sender, share);
    }

    // ========================================================================
    // Admin Functions
    // ========================================================================

    /**
     * @notice Обновить проценты распределения
     */
    function updateShares(
        uint256 _team,
        uint256 _treasury,
        uint256 _marketing,
        uint256 _arbiters,
        uint256 _reserve
    ) external onlyOwner validShares(_team, _treasury, _marketing, _arbiters, _reserve) {
        teamPercent = _team;
        treasuryPercent = _treasury;
        marketingPercent = _marketing;
        arbitersPercent = _arbiters;
        reservePercent = _reserve;
        
        team.sharePercent = _team;
        treasury.sharePercent = _treasury;
        marketing.sharePercent = _marketing;
        reserve.sharePercent = _reserve;
    }

    /**
     * @notice Обновить кошелёк получателя
     */
    function updateShareholderWallet(string calldata _role, address payable _newWallet) external onlyOwner {
        require(_newWallet != address(0), "Invalid wallet");
        
        if (keccak256(bytes(_role)) == keccak256(bytes("Team"))) {
            team.wallet = _newWallet;
        } else if (keccak256(bytes(_role)) == keccak256(bytes("Treasury"))) {
            treasury.wallet = _newWallet;
        } else if (keccak256(bytes(_role)) == keccak256(bytes("Marketing"))) {
            marketing.wallet = _newWallet;
        } else if (keccak256(bytes(_role)) == keccak256(bytes("Reserve"))) {
            reserve.wallet = _newWallet;
        } else {
            revert("Invalid role");
        }
        
        emit ShareholderUpdated(_role, _newWallet, _roleToPercent(_role));
    }

    /**
     * @notice Обновить Escrow контракт
     */
    function updateEscrowContract(address _newEscrow) external onlyOwner {
        require(_newEscrow != address(0), "Invalid escrow");
        emit EscrowContractUpdated(escrowContract, _newEscrow);
        escrowContract = _newEscrow;
    }

    /**
     * @notice Экстренная пауза
     */
    function togglePause() external onlyOwner {
        paused = !paused;
        emit EmergencyPause(paused, msg.sender, block.timestamp);
    }

    /**
     * @notice Передать владение
     */
    function transferOwnership(address _newOwner) external onlyOwner {
        require(_newOwner != address(0), "Invalid owner");
        emit OwnershipTransferred(owner, _newOwner);
        owner = _newOwner;
    }

    // ========================================================================
    // View Functions
    // ========================================================================

    /// Получить всех получателей
    function getShareholders() external view returns (
        Shareholder memory _team,
        Shareholder memory _treasury,
        Shareholder memory _marketing,
        Shareholder memory _reserve
    ) {
        return (team, treasury, marketing, reserve);
    }

    /// Получить баланс пула арбитров
    function getArbiterPoolInfo() external view returns (
        uint256 totalShares,
        uint256 totalBalance,
        uint256 arbiterShare
    ) {
        return (
            arbiterPool.totalShares,
            arbiterPool.totalBalance,
            calculateArbiterShare(msg.sender)
        );
    }

    /// Получить историю распределений
    function getDistributionHistory(uint256 _offset, uint256 _limit) external view returns (DistributionLog[] memory) {
        require(_limit > 0 && _limit <= 100, "Invalid limit");
        
        uint256 start = _offset < distributionHistory.length ? _offset : distributionHistory.length;
        uint256 end = start + _limit < distributionHistory.length ? start + _limit : distributionHistory.length;
        uint256 count = end - start;
        
        DistributionLog[] memory result = new DistributionLog[](count);
        for (uint256 i = 0; i < count; i++) {
            result[i] = distributionHistory[start + i];
        }
        
        return result;
    }

    /// Проверить сумму процентов для роли
    function _roleToPercent(string memory _role) internal view returns (uint256) {
        if (keccak256(bytes(_role)) == keccak256(bytes("Team"))) return teamPercent;
        if (keccak256(bytes(_role)) == keccak256(bytes("Treasury"))) return treasuryPercent;
        if (keccak256(bytes(_role)) == keccak256(bytes("Marketing"))) return marketingPercent;
        if (keccak256(bytes(_role)) == keccak256(bytes("Reserve"))) return reservePercent;
        return 0;
    }

    // ========================================================================
    // Internal Functions
    // ========================================================================

    /// Безопасный перевод ETH с защитой от reentrancy
    function _safeTransfer(address payable _to, uint256 _amount) internal {
        if (_amount == 0) {
            return;
        }
        
        bool isContract;
        assembly {
            isContract := gt(extcodesize(_to), 0)
        }
        
        if (!isContract) {
            (bool success, ) = _to.call{value: _amount}("");
            require(success, "Transfer failed");
        } else {
            // For contracts, use low-level call with limited gas
            (bool success, ) = _to.call{value: _amount, gas: 2300}("");
            require(success, "Contract transfer failed");
        }
    }
}
