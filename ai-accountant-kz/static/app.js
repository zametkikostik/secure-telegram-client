// ===== Global State =====
let currentPage = 'dashboard';
let allTransactions = [];
let charts = {};

// ===== Init =====
document.addEventListener('DOMContentLoaded', () => {
    initNavigation();
    loadDashboard();
    document.querySelector('#add-tx-form input[name="date"]').valueAsDate = new Date();
    loadUserProfile();
});

// ===== User Profile =====
async function loadUserProfile() {
    const token = localStorage.getItem('access_token');
    
    if (!token) {
        // Нет токена - перенаправляем на вход
        window.location.href = '/auth';
        return;
    }
    
    try {
        const response = await fetch('/api/v1/auth/me', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        if (response.ok) {
            const user = await response.json();
            document.getElementById('user-name').textContent = user.full_name || 'Пользователь';
            document.getElementById('user-email').textContent = user.email;
        } else if (response.status === 401) {
            // Токен истёк
            localStorage.removeItem('access_token');
            window.location.href = '/auth';
        }
    } catch (error) {
        console.error('Error loading profile:', error);
    }
}

function logout() {
    if (confirm('Выйти из аккаунта?')) {
        localStorage.removeItem('access_token');
        localStorage.removeItem('token_type');
        window.location.href = '/auth';
    }
}

// ===== Navigation =====
function initNavigation() {
    document.querySelectorAll('.nav-item').forEach(item => {
        item.addEventListener('click', (e) => {
            e.preventDefault();
            const page = item.dataset.page;
            showPage(page);
        });
    });
}

function showPage(page) {
    document.querySelectorAll('.nav-item').forEach(item => {
        item.classList.toggle('active', item.dataset.page === page);
    });
    
    document.querySelectorAll('.page').forEach(p => {
        p.classList.toggle('active', p.id === `page-${page}`);
    });
    
    const titles = {
        dashboard: 'Главная',
        transactions: 'Транзакции',
        documents: 'Документы',
        employees: 'Сотрудники',
        banks: 'Банки',
        tax: 'Налоги',
        reports: 'Отчёты',
        settings: 'Настройки'
    };
    document.getElementById('page-title').textContent = titles[page] || page;
    
    currentPage = page;
    
    if (page === 'transactions') loadTransactions();
    if (page === 'employees') loadEmployees();
    if (page === 'tax') loadTaxData();
    if (page === 'reports') loadReports();
}

// ===== Dashboard =====
async function loadDashboard() {
    await loadSummary();
    await loadRecentTransactions();
    loadChart();
    loadKPI();
}

async function loadSummary() {
    const token = localStorage.getItem('access_token');
    
    try {
        const response = await fetch('/api/v1/summary', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        const summary = await response.json();
        
        document.getElementById('dash-income').textContent = formatMoney(summary.total_income) + ' ₸';
        document.getElementById('dash-expense').textContent = formatMoney(summary.total_expense) + ' ₸';
        document.getElementById('dash-profit').textContent = formatMoney(summary.net_profit) + ' ₸';
        document.getElementById('dash-tax').textContent = formatMoney(summary.tax_amount) + ' ₸';
        
        const taxResponse = await fetch('/api/v1/tax/calculate?period=2026');
        const taxData = await taxResponse.json();
        const limitPercent = (summary.total_income / taxData.limit_amount) * 100;
        document.getElementById('limit-progress').style.width = Math.min(limitPercent, 100) + '%';
        document.getElementById('limit-used').textContent = limitPercent.toFixed(1) + '%';
        
    } catch (error) {
        console.error('Error loading summary:', error);
    }
}

async function loadRecentTransactions() {
    const token = localStorage.getItem('access_token');
    
    try {
        const response = await fetch('/api/v1/transactions?limit=5', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        allTransactions = await response.json();
        
        const container = document.getElementById('recent-transactions');
        if (allTransactions.length === 0) {
            container.innerHTML = '<p class="empty-message">Нет транзакций</p>';
            return;
        }
        
        container.innerHTML = allTransactions.map(tx => `
            <div class="transaction-item ${tx.type}">
                <div>
                    <div class="tx-desc">${escapeHtml(tx.description)}</div>
                    <div class="tx-meta">${new Date(tx.date).toLocaleDateString('ru-RU')}</div>
                </div>
                <div class="tx-amount ${tx.type}">
                    ${tx.type === 'income' ? '+' : '-'}${formatMoney(tx.amount)} ₸
                </div>
            </div>
        `).join('');
        
    } catch (error) {
        console.error('Error loading transactions:', error);
    }
}

// ===== Charts =====
function loadChart() {
    const ctx = document.getElementById('chart-main');
    if (!ctx) return;
    
    const monthlyData = groupByMonth(allTransactions);
    const months = Object.keys(monthlyData);
    
    if (charts.main) charts.main.destroy();
    
    charts.main = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: months,
            datasets: [
                {
                    label: 'Доходы',
                    data: months.map(m => monthlyData[m].income),
                    backgroundColor: '#10b981',
                    borderRadius: 4
                },
                {
                    label: 'Расходы',
                    data: months.map(m => monthlyData[m].expense),
                    backgroundColor: '#ef4444',
                    borderRadius: 4
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: { legend: { position: 'top' } },
            scales: {
                y: { beginAtZero: true, ticks: { callback: v => formatMoney(v) + ' ₸' } }
            }
        }
    });
}

function groupByMonth(transactions) {
    const data = {};
    transactions.forEach(tx => {
        const month = new Date(tx.date).toLocaleDateString('ru-RU', {month: 'short', year: '2-digit'});
        if (!data[month]) data[month] = {income: 0, expense: 0};
        if (tx.type === 'income') data[month].income += tx.amount;
        else data[month].expense += tx.amount;
    });
    return data;
}

// ===== Transactions =====
async function loadTransactions() {
    const filterType = document.getElementById('tx-filter-type').value;
    const filterSource = document.getElementById('tx-filter-source').value;
    const search = document.getElementById('tx-search').value.toLowerCase();
    const token = localStorage.getItem('access_token');

    try {
        let url = '/api/v1/transactions?limit=100';
        if (filterType) url += '&tx_type=' + filterType;

        const response = await fetch(url, {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        let transactions = await response.json();
        
        if (filterSource) transactions = transactions.filter(tx => tx.source === filterSource);
        if (search) transactions = transactions.filter(tx => 
            tx.description.toLowerCase().includes(search) ||
            (tx.counterparty && tx.counterparty.toLowerCase().includes(search))
        );
        
        const tbody = document.getElementById('transactions-table');
        tbody.innerHTML = transactions.map(tx => `
            <tr>
                <td>${new Date(tx.date).toLocaleDateString('ru-RU')}</td>
                <td><span class="badge ${tx.type === 'income' ? 'success' : 'error'}">${tx.type === 'income' ? 'Доход' : 'Расход'}</span></td>
                <td>${escapeHtml(tx.description)}</td>
                <td>${tx.counterparty ? escapeHtml(tx.counterparty) : '-'}</td>
                <td class="${tx.type === 'income' ? 'text-green' : 'text-red'}">
                    ${tx.type === 'income' ? '+' : '-'}${formatMoney(tx.amount)} ₸
                </td>
                <td>${tx.source}</td>
                <td>${tx.ai_category || '-'}</td>
                <td><button class="btn btn-sm" onclick="deleteTransaction('${tx.id}')">🗑️</button></td>
            </tr>
        `).join('');
        
    } catch (error) {
        console.error('Error loading transactions:', error);
        showToast('Ошибка загрузки транзакций', true);
    }
}

// ===== Add Transaction =====
function showAddTransactionModal() {
    document.getElementById('modal-add-transaction').classList.add('active');
}

function closeModal(id) {
    document.getElementById(id).classList.remove('active');
}

async function addTransaction(e) {
    e.preventDefault();
    const form = e.target;
    const formData = new FormData(form);

    const token = localStorage.getItem('access_token');
    
    const params = new URLSearchParams();
    params.append('date', formData.get('date'));
    params.append('amount', formData.get('amount'));
    params.append('type', formData.get('type'));
    params.append('description', formData.get('description'));
    if (formData.get('counterparty')) params.append('counterparty', formData.get('counterparty'));
    params.append('source', 'manual');

    try {
        const response = await fetch('/api/v1/transactions?' + params.toString(), {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (response.ok) {
            showToast('✅ Транзакция добавлена');
            closeModal('modal-add-transaction');
            form.reset();
            form.querySelector('input[name="date"]').valueAsDate = new Date();
            loadDashboard();
            loadTransactions();
        } else {
            showToast('❌ Ошибка при добавлении', true);
        }
    } catch (error) {
        showToast('❌ Ошибка соединения', true);
    }
}

async function deleteTransaction(id) {
    if (!confirm('Удалить транзакцию?')) return;

    const token = localStorage.getItem('access_token');

    try {
        const response = await fetch(`/api/v1/transactions/${id}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        if (response.ok) {
            showToast('🗑️ Транзакция удалена');
            loadDashboard();
            loadTransactions();
        } else {
            showToast('❌ Ошибка', true);
        }
    } catch (error) {
        showToast('❌ Ошибка', true);
    }
}

// ===== Banks =====
function connectBank(bank) {
    showPage('banks');
    showToast(`Подключение ${bank === 'kaspi' ? 'Kaspi' : 'Halyk'}... Введите API ключи`);
}

async function saveBankSettings(bank) {
    const prefix = bank === 'kaspi' ? 'kaspi' : 'halyk';
    const apiKey = document.getElementById(`${prefix}-api-key`).value;
    const merchantId = document.getElementById(`${prefix}-merchant-id`).value || 
                       document.getElementById(`${prefix}-client-id`).value;
    
    localStorage.setItem(`${prefix}_api_key`, apiKey);
    localStorage.setItem(`${prefix}_merchant_id`, merchantId);
    
    document.getElementById(`${prefix}-status`).textContent = 'Подключен';
    document.getElementById(`${prefix}-detail-status`).textContent = 'Подключен';
    document.getElementById(`${prefix}-detail-status`).className = 'badge success';
    
    showToast(`✅ Настройки ${bank} сохранены`);
}

async function syncBank(bank) {
    const token = localStorage.getItem('access_token');
    showToast(`🔄 Синхронизация с ${bank === 'kaspi' ? 'Kaspi' : 'Halyk'}...`);

    try {
        const endpoint = bank === 'kaspi' ? '/api/v1/integrations/kaspi/sync' : '/api/v1/integrations/halyk/sync';
        const response = await fetch(endpoint + '?days=30', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (response.ok) {
            const result = await response.json();
            showToast(`✅ Синхронизировано: ${result.imported || 0} транзакций`);
            loadDashboard();
        } else if (response.status === 401) {
            showToast('❌ Требуется авторизация', true);
            window.location.href = '/auth';
        } else {
            showToast('⚠️ Банк не подключен', true);
        }
    } catch (error) {
        showToast('❌ Ошибка синхронизации', true);
    }
}

// ===== Tax =====
async function loadTaxData() {
    try {
        const summary = await fetch('/api/v1/summary').then(r => r.json());
        const taxCalc = await fetch('/api/v1/tax/calculate?period=2026').then(r => r.json());
        
        document.getElementById('tax-income').textContent = formatMoney(summary.total_income) + ' ₸';
        document.getElementById('tax-expense').textContent = formatMoney(summary.total_expense) + ' ₸';
        document.getElementById('tax-amount').textContent = formatMoney(taxCalc.tax_amount) + ' ₸';
        
        const declResponse = await fetch('/api/v1/integrations/tax/declarations');
        const declarations = await declResponse.json();
        
        const container = document.getElementById('declarations-list');
        if (declarations.length === 0) {
            container.innerHTML = '<p class="empty-message">Нет деклараций</p>';
        } else {
            container.innerHTML = declarations.map(d => `
                <div class="declaration-item">
                    <div>
                        <strong>${d.period}</strong>
                        <div>${formatMoney(d.total_income)} ₸ → ${formatMoney(d.tax_amount)} ₸</div>
                    </div>
                    <span class="badge ${d.status === 'paid' ? 'success' : 'warning'}">${d.status}</span>
                </div>
            `).join('');
        }
    } catch (error) {
        console.error('Error loading tax data:', error);
    }
}

async function submitDeclaration() {
    const period = document.getElementById('tax-period').value;
    const token = localStorage.getItem('access_token');
    
    if (!confirm(`Отправить декларацию за ${period}?`)) return;

    try {
        const response = await fetch(`/api/v1/integrations/tax/submit-declaration?period=${period}`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        if (response.ok) {
            const result = await response.json();
            showToast(`✅ Декларация отправлена!`);
            loadTaxData();
        } else if (response.status === 401) {
            showToast('❌ Требуется авторизация', true);
            window.location.href = '/auth';
        } else {
            const error = await response.json();
            showToast('⚠️ ' + (error.detail || 'Ошибка отправки'), true);
        }
    } catch (error) {
        showToast('❌ Ошибка соединения', true);
    }
}

// ===== Reports =====
async function loadReports() {
    await loadMonthlyReport();
    await loadTrendReport();
    await loadCategoryReport();
}

async function loadMonthlyReport() {
    try {
        const currentYear = new Date().getFullYear();
        const response = await fetch(`/api/v1/stats/monthly?year=${currentYear}`);
        const result = await response.json();
        const data = result.data || [];
        
        const months = ['Янв', 'Фев', 'Мар', 'Апр', 'Май', 'Июн', 'Июл', 'Авг', 'Сен', 'Окт', 'Ноя', 'Дек'];
        const labels = data.map(d => months[d.month - 1]);
        const incomeData = data.map(d => d.income);
        const expenseData = data.map(d => d.expense);
        const profitData = incomeData.map((inc, i) => inc - expenseData[i]);
        
        if (charts.monthly) charts.monthly.destroy();
        
        const ctx = document.getElementById('chart-monthly');
        if (!ctx) return;
        
        charts.monthly = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: labels,
                datasets: [
                    {
                        label: 'Доходы',
                        data: incomeData,
                        backgroundColor: '#10b981',
                        borderRadius: 4
                    },
                    {
                        label: 'Расходы',
                        data: expenseData,
                        backgroundColor: '#ef4444',
                        borderRadius: 4
                    },
                    {
                        label: 'Прибыль',
                        data: profitData,
                        type: 'line',
                        borderColor: '#3b82f6',
                        borderWidth: 2,
                        fill: false
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { position: 'top' },
                    title: { display: true, text: `${currentYear} год - Месячный отчёт` }
                },
                scales: {
                    y: { beginAtZero: true, ticks: { callback: v => formatMoney(v) + ' ₸' } }
                }
            }
        });
    } catch (error) {
        console.error('Error loading monthly report:', error);
    }
}

async function loadTrendReport() {
    try {
        const response = await fetch('/api/v1/transactions?limit=1000');
        const transactions = await response.json();
        
        // Group by month for trend
        const monthlyData = {};
        transactions.forEach(tx => {
            const date = new Date(tx.date);
            const key = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`;
            if (!monthlyData[key]) monthlyData[key] = {income: 0, expense: 0, count: 0};
            if (tx.type === 'income') monthlyData[key].income += tx.amount;
            else monthlyData[key].expense += tx.amount;
            monthlyData[key].count++;
        });
        
        const sortedKeys = Object.keys(monthlyData).sort();
        const labels = sortedKeys.map(k => {
            const [year, month] = k.split('-');
            return new Date(year, month - 1).toLocaleDateString('ru-RU', {month: 'short', year: '2-digit'});
        });
        
        // Calculate cumulative
        let cumulativeIncome = 0;
        let cumulativeExpense = 0;
        const incomeTrend = sortedKeys.map(k => {
            cumulativeIncome += monthlyData[k].income;
            return cumulativeIncome;
        });
        const expenseTrend = sortedKeys.map(k => {
            cumulativeExpense += monthlyData[k].expense;
            return cumulativeExpense;
        });
        
        if (charts.trend) charts.trend.destroy();
        
        const ctx = document.getElementById('chart-trend');
        if (!ctx) return;
        
        charts.trend = new Chart(ctx, {
            type: 'line',
            data: {
                labels: labels,
                datasets: [
                    {
                        label: 'Доходы (накопительно)',
                        data: incomeTrend,
                        borderColor: '#10b981',
                        backgroundColor: 'rgba(16, 185, 129, 0.1)',
                        fill: true,
                        tension: 0.4
                    },
                    {
                        label: 'Расходы (накопительно)',
                        data: expenseTrend,
                        borderColor: '#ef4444',
                        backgroundColor: 'rgba(239, 68, 68, 0.1)',
                        fill: true,
                        tension: 0.4
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { position: 'top' },
                    title: { display: true, text: 'Динамика доходов и расходов' }
                },
                scales: {
                    y: { beginAtZero: true, ticks: { callback: v => formatMoney(v) + ' ₸' } }
                }
            }
        });
    } catch (error) {
        console.error('Error loading trend report:', error);
    }
}

async function loadCategoryReport() {
    try {
        const response = await fetch('/api/v1/transactions?limit=1000');
        const transactions = await response.json();
        
        // Group by category
        const categories = {};
        transactions.forEach(tx => {
            const cat = tx.ai_category || tx.manual_category || 'Без категории';
            if (!categories[cat]) categories[cat] = {income: 0, expense: 0};
            if (tx.type === 'income') categories[cat].income += tx.amount;
            else categories[cat].expense += tx.amount;
        });
        
        // Income by category (doughnut)
        const incomeCats = Object.entries(categories)
            .filter(([_, v]) => v.income > 0)
            .sort((a, b) => b[1].income - a[1].income);
        
        if (charts.categoryIncome) charts.categoryIncome.destroy();
        
        const ctxIncome = document.getElementById('chart-categories-income');
        if (ctxIncome && incomeCats.length > 0) {
            charts.categoryIncome = new Chart(ctxIncome, {
                type: 'doughnut',
                data: {
                    labels: incomeCats.map(([k]) => k),
                    datasets: [{
                        data: incomeCats.map(([_, v]) => v.income),
                        backgroundColor: [
                            '#10b981', '#3b82f6', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899', '#6366f1'
                        ]
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { position: 'right' },
                        title: { display: true, text: 'Доходы по категориям' }
                    }
                }
            });
        }
        
        // Expense by category (doughnut)
        const expenseCats = Object.entries(categories)
            .filter(([_, v]) => v.expense > 0)
            .sort((a, b) => b[1].expense - a[1].expense);
        
        if (charts.categoryExpense) charts.categoryExpense.destroy();
        
        const ctxExpense = document.getElementById('chart-categories-expense');
        if (ctxExpense && expenseCats.length > 0) {
            charts.categoryExpense = new Chart(ctxExpense, {
                type: 'doughnut',
                data: {
                    labels: expenseCats.map(([k]) => k),
                    datasets: [{
                        data: expenseCats.map(([_, v]) => v.expense),
                        backgroundColor: [
                            '#ef4444', '#f59e0b', '#8b5cf6', '#ec4899', '#6366f1', '#10b981', '#3b82f6'
                        ]
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        legend: { position: 'right' },
                        title: { display: true, text: 'Расходы по категориям' }
                    }
                }
            });
        }
    } catch (error) {
        console.error('Error loading category report:', error);
    }
}

async function exportCSV() {
    try {
        const response = await fetch('/api/v1/export/csv');
        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `transactions_${new Date().toISOString().split('T')[0]}.csv`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        window.URL.revokeObjectURL(url);
        showToast('📥 Экспорт загружен');
    } catch (error) {
        showToast('❌ Ошибка экспорта', true);
    }
}

// ===== Settings =====
function saveProfile() {
    showToast('✅ Профиль сохранён');
}

function saveTelegramSettings() {
    showToast('✅ Telegram настройки сохранены');
}

function saveAISettings() {
    showToast('✅ AI настройки сохранены');
}

// ===== Utilities =====
function formatMoney(amount) {
    return amount.toLocaleString('ru-RU', { minimumFractionDigits: 0, maximumFractionDigits: 2 });
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showToast(message, isError = false) {
    const toast = document.createElement('div');
    toast.className = 'toast' + (isError ? ' error' : '');
    toast.textContent = message;
    document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

function refreshData() {
    loadDashboard();
    showToast('🔄 Обновлено');
}

function toggleNotifications() {
    showToast('🔔 Уведомления');
}

// ===== OCR Functions =====
let selectedFile = null;

function showOCRModal() {
    document.getElementById('modal-ocr').classList.add('active');
}

function previewReceipt() {
    const fileInput = document.getElementById('ocr-file');
    const file = fileInput.files[0];
    
    if (file) {
        selectedFile = file;
        const reader = new FileReader();
        reader.onload = function(e) {
            const img = document.getElementById('preview-img');
            img.src = e.target.result;
            img.style.display = 'block';
            document.getElementById('ocr-process-btn').disabled = false;
        };
        reader.readAsDataURL(file);
    }
}

async function processReceipt() {
    if (!selectedFile) {
        showToast('❌ Выберите файл', true);
        return;
    }

    showToast('🔍 Распознавание чека...');

    const token = localStorage.getItem('access_token');
    const formData = new FormData();
    formData.append('file', selectedFile);
    formData.append('create_transaction', 'false');

    try {
        const response = await fetch('/api/v1/ocr/upload', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
            body: formData
        });

        if (response.ok) {
            const result = await response.json();

            const resultDiv = document.getElementById('ocr-result');
            const textPre = document.getElementById('ocr-text');

            resultDiv.style.display = 'block';
            
            // Формируем понятный вывод
            let outputText = '';
            if (result.transaction_data) {
                outputText = '✅ Чек распознан:\n' + 
                    'Сумма: ' + result.transaction_data.amount + ' ₸\n' +
                    'Тип: ' + (result.transaction_data.type === 'income' ? 'Доход' : 'Расход') + '\n' +
                    'Магазин: ' + (result.transaction_data.counterparty || 'Неизвестно') + '\n' +
                    'Дата: ' + (result.transaction_data.date || 'Не указана') + '\n\n' +
                    'Сырой текст:\n' + result.raw_text;
            } else if (result.raw_text && result.raw_text.trim()) {
                outputText = '⚠️ Распознан текст (AI отключён):\n\n' + result.raw_text + 
                    '\n\n💡 Для авто-распознавания структуры чека нужен API ключ.\n' +
                    'Настройте OPENAI_API_KEY в .env файле.';
            } else {
                outputText = '❌ Не удалось распознать текст.\n\n' +
                    'Возможно:\n' +
                    '• Изображение слишком тёмное/светлое\n' +
                    '• Текст нечёткий\n' +
                    '• Это не чек\n\n' +
                    'Попробуйте загрузить более качественное фото.';
            }
            
            textPre.textContent = outputText;

            if (result.success && result.transaction_data) {
                showToast('✅ Чек распознан! Можно добавить транзакцию');
            } else if (result.raw_text && result.raw_text.trim()) {
                showToast('⚠️ Текст распознан частично', false);
            } else {
                showToast('⚠️ Распознавание не удалось', true);
            }
        } else if (response.status === 401) {
            showToast('❌ Требуется авторизация', true);
            window.location.href = '/auth';
        } else {
            const error = await response.json();
            showToast('❌ ' + (error.detail || 'Ошибка распознавания'), true);
        }
    } catch (error) {
        showToast('❌ Ошибка соединения: ' + error.message, true);
    }
}

// ===== Document Functions =====
async function generateInvoice(e) {
    e.preventDefault();
    const token = localStorage.getItem('access_token');

    const buyerName = document.getElementById('invoice-buyer-name').value;
    const buyerInn = document.getElementById('invoice-buyer-inn').value;
    const items = document.getElementById('invoice-items').value;
    const total = document.getElementById('invoice-total').value;
    const vat = document.getElementById('invoice-vat').value;

    const formData = new FormData();
    formData.append('buyer_name', buyerName);
    formData.append('buyer_inn', buyerInn);
    formData.append('items', items);
    formData.append('total', total);
    formData.append('vat', vat);

    try {
        const response = await fetch('/api/v1/documents/invoice', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
            body: formData
        });

        if (response.ok) {
            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `invoice_${new Date().getTime()}.pdf`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            window.URL.revokeObjectURL(url);
            showToast('✅ Счёт сгенерирован!');
        } else if (response.status === 401) {
            showToast('❌ Требуется авторизация', true);
            window.location.href = '/auth';
        } else {
            showToast('❌ Ошибка генерации', true);
        }
    } catch (error) {
        showToast('❌ Ошибка соединения', true);
    }
}

async function generateAct(e) {
    e.preventDefault();
    const token = localStorage.getItem('access_token');

    const buyerName = document.getElementById('act-buyer-name').value;
    const buyerInn = document.getElementById('act-buyer-inn').value;
    const services = document.getElementById('act-services').value;
    const total = document.getElementById('act-total').value;
    const period = document.getElementById('act-period').value || new Date().toLocaleDateString('ru-RU', {month: 'long', year: 'numeric'});

    const formData = new FormData();
    formData.append('buyer_name', buyerName);
    formData.append('buyer_inn', buyerInn);
    formData.append('services', services);
    formData.append('total', total);
    formData.append('period', period);

    try {
        const response = await fetch('/api/v1/documents/act', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            },
            body: formData
        });

        if (response.ok) {
            const blob = await response.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `act_${new Date().getTime()}.pdf`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            window.URL.revokeObjectURL(url);
            showToast('✅ Акт сгенерирован!');
        } else if (response.status === 401) {
            showToast('❌ Требуется авторизация', true);
            window.location.href = '/auth';
        } else {
            showToast('❌ Ошибка генерации', true);
        }
    } catch (error) {
        showToast('❌ Ошибка соединения', true);
    }
}

// ===== KPI Functions =====
async function loadKPI() {
    const period = document.getElementById('kpi-period').value;
    const token = localStorage.getItem('access_token');
    
    try {
        // KPI Summary
        const kpiResponse = await fetch(`/api/v1/kpi/summary?period=${period}`, {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        const kpi = await kpiResponse.json();
        
        document.getElementById('kpi-avg-check').textContent = formatMoney(kpi.avg_check) + ' ₸';
        document.getElementById('kpi-margin').textContent = kpi.profit_margin.toFixed(1) + '%';
        document.getElementById('kpi-employees').textContent = kpi.employees_count;
        document.getElementById('kpi-payroll').textContent = formatMoney(kpi.payroll_fund) + ' ₸';
        
        // Tax Limit
        const limitResponse = await fetch('/api/v1/kpi/limits', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        const limit = await limitResponse.json();
        
        const gauge = document.getElementById('limit-gauge');
        const circumference = 2 * Math.PI * 45; // 283
        const offset = circumference - (limit.used_percent / 100) * circumference;
        gauge.style.strokeDashoffset = offset;
        
        document.getElementById('limit-percent').textContent = limit.used_percent.toFixed(1) + '%';
        document.getElementById('limit-total').textContent = formatMoney(limit.limit) + ' ₸';
        document.getElementById('limit-remaining').textContent = formatMoney(limit.remaining) + ' ₸';
        
        const statusEl = document.getElementById('limit-status');
        statusEl.className = 'limit-status status-' + limit.status;
        statusEl.textContent = limit.status === 'normal' ? '✓ В пределах лимита' : 
                               limit.status === 'warning' ? '⚠️ Внимание: 75% лимита' : 
                               '🚨 Критично: 90% лимита!';
        
    } catch (error) {
        console.error('Error loading KPI:', error);
    }
}

// ===== Employee Functions =====
async function loadEmployees() {
    try {
        const response = await fetch('/api/v1/employees?is_active=true');
        const employees = await response.json();
        
        const list = document.getElementById('employees-list');
        const select = document.getElementById('payroll-employee');
        
        if (employees.length === 0) {
            list.innerHTML = '<p class="empty-message">Нет сотрудников</p>';
            select.innerHTML = '<option value="">Нет сотрудников</option>';
            return;
        }
        
        list.innerHTML = employees.map(emp => `
            <div class="employee-item">
                <div class="emp-info">
                    <strong>${emp.full_name}</strong>
                    <div class="emp-meta">${emp.position || '—'} • ИНН: ${emp.inn}</div>
                    <div class="emp-salary">Оклад: ${formatMoney(emp.salary)} ₸</div>
                </div>
                <button class="btn btn-sm btn-danger" onclick="fireEmployee('${emp.id}')">Уволить</button>
            </div>
        `).join('');
        
        select.innerHTML = '<option value="">Выберите сотрудника</option>' + 
            employees.map(emp => `<option value="${emp.id}" data-salary="${emp.salary}">${emp.full_name}</option>`).join('');
        
    } catch (error) {
        console.error('Error loading employees:', error);
    }
}

function showAddEmployeeModal() {
    document.getElementById('modal-add-employee').classList.add('active');
}

async function addEmployee(e) {
    e.preventDefault();
    const form = e.target;
    const formData = new FormData(form);
    
    const params = new URLSearchParams();
    for (let [key, value] of formData.entries()) {
        params.append(key, value);
    }
    
    try {
        const response = await fetch('/api/v1/employees?' + params.toString(), {
            method: 'POST'
        });
        
        if (response.ok) {
            showToast('✅ Сотрудник добавлен');
            closeModal('modal-add-employee');
            form.reset();
            loadEmployees();
        } else {
            const error = await response.json();
            showToast('❌ ' + error.detail, true);
        }
    } catch (error) {
        showToast('❌ Ошибка соединения', true);
    }
}

async function fireEmployee(id) {
    if (!confirm('Уволить сотрудника?')) return;
    
    try {
        const response = await fetch(`/api/v1/employees/${id}`, {
            method: 'DELETE'
        });
        
        if (response.ok) {
            showToast('🗑️ Сотрудник уволен');
            loadEmployees();
        } else {
            showToast('❌ Ошибка', true);
        }
    } catch (error) {
        showToast('❌ Ошибка', true);
    }
}

async function calculatePayrollPreview() {
    const employeeId = document.getElementById('payroll-employee').value;
    const period = document.getElementById('payroll-period').value;
    const bonus = parseFloat(document.getElementById('payroll-bonus').value) || 0;
    
    if (!employeeId || !period) {
        document.getElementById('payroll-preview').style.display = 'none';
        return;
    }
    
    try {
        const response = await fetch(`/api/v1/employees/${employeeId}/payroll/calculate?period=${period}&bonus=${bonus}`, {
            method: 'POST'
        });
        
        if (response.ok) {
            const data = await response.json();
            const preview = document.getElementById('payroll-preview');
            
            preview.innerHTML = `
                <div class="payroll-breakdown">
                    <div class="pb-row"><span>Начислено:</span><strong>${formatMoney(data.total_accrued)} ₸</strong></div>
                    <div class="pb-row"><span>ОПВ (10%):</span><span>${formatMoney(data.deductions.opv)} ₸</span></div>
                    <div class="pb-row"><span>ИПН:</span><span>${formatMoney(data.deductions.opv_ipn)} ₸</span></div>
                    <div class="pb-row"><span>ОСМС:</span><span>${formatMoney(data.deductions.osms)} ₸</span></div>
                    <div class="pb-row total"><span>К выплате:</span><strong>${formatMoney(data.net_salary)} ₸</strong></div>
                </div>
            `;
            preview.style.display = 'block';
        }
    } catch (error) {
        console.error('Error calculating payroll:', error);
    }
}

async function accruePayroll() {
    const token = localStorage.getItem('access_token');
    const employeeId = document.getElementById('payroll-employee').value;
    const period = document.getElementById('payroll-period').value;
    const bonus = parseFloat(document.getElementById('payroll-bonus').value) || 0;

    if (!employeeId || !period) {
        showToast('❌ Выберите сотрудника и период', true);
        return;
    }

    if (!confirm(`Начислить зарплату за ${period}?`)) return;

    try {
        const response = await fetch(`/api/v1/employees/${employeeId}/payroll/accrue?period=${period}&bonus=${bonus}`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (response.ok) {
            showToast('✅ Зарплата начислена');
            document.getElementById('payroll-preview').style.display = 'none';
            document.getElementById('payroll-bonus').value = 0;
        } else if (response.status === 401) {
            showToast('❌ Требуется авторизация', true);
            window.location.href = '/auth';
        } else {
            showToast('❌ Ошибка', true);
        }
    } catch (error) {
        showToast('❌ Ошибка', true);
    }
}

// ===== Load Chart.js =====
if (typeof Chart === 'undefined') {
    const script = document.createElement('script');
    script.src = 'https://cdn.jsdelivr.net/npm/chart.js';
    document.head.appendChild(script);
}

// ===== PWA Service Worker =====
if ('serviceWorker' in navigator) {
    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/static/sw.js')
            .then(registration => {
                console.log('SW registered:', registration.scope);
            })
            .catch(error => {
                console.log('SW registration failed:', error);
            });
    });
}

// ===== Request Notification Permission =====
async function requestNotificationPermission() {
    if ('Notification' in window && Notification.permission === 'default') {
        await Notification.requestPermission();
    }
}

// ===== Show Install Prompt =====
let deferredPrompt;
window.addEventListener('beforeinstallprompt', e => {
    e.preventDefault();
    deferredPrompt = e;
    showInstallPrompt();
});

function showInstallPrompt() {
    const toast = document.createElement('div');
    toast.className = 'toast';
    toast.innerHTML = '📱 Установить приложение? <button onclick="installApp()" style="margin-left:10px;padding:5px 10px;background:white;color:#667eea;border:none;border-radius:4px;cursor:pointer;">Установить</button>';
    document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 5000);
}

async function installApp() {
    if (deferredPrompt) {
        deferredPrompt.prompt();
        const { outcome } = await deferredPrompt.userChoice;
        if (outcome === 'accepted') {
            showToast('🎉 Приложение установлено!');
        }
        deferredPrompt = null;
    }
}
