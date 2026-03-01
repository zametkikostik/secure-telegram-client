// ===== Auth Page JavaScript =====

// Переключение между вкладками
function showTab(tab) {
    // Убираем активный класс у кнопок
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.toggle('active', btn.textContent.toLowerCase().includes(tab === 'login' ? 'вход' : 'рег'));
    });
    
    // Переключаем формы
    document.querySelectorAll('.auth-form').forEach(form => {
        form.classList.toggle('active', form.id === `${tab}-form`);
    });
}

// Вход
async function handleLogin(event) {
    event.preventDefault();
    
    const form = event.target;
    const email = document.getElementById('login-email').value;
    const password = document.getElementById('login-password').value;
    const submitBtn = form.querySelector('button[type="submit"]');
    
    console.log('Login attempt:', email);
    
    // Показываем загрузку
    submitBtn.classList.add('btn-loading');
    submitBtn.disabled = true;
    
    try {
        const formData = new FormData();
        formData.append('username', email);
        formData.append('password', password);
        
        console.log('Sending request to /api/v1/auth/login');
        
        const response = await fetch('/api/v1/auth/login', {
            method: 'POST',
            body: formData
        });
        
        console.log('Response status:', response.status);
        
        if (response.ok) {
            const data = await response.json();
            console.log('Token received:', data.access_token.substring(0, 20) + '...');
            
            // Сохраняем токен
            localStorage.setItem('access_token', data.access_token);
            localStorage.setItem('token_type', data.token_type);
            
            showToast('✅ Вход успешен! Перенаправление...');
            
            // Перенаправление в кабинет
            setTimeout(() => {
                window.location.href = '/';
            }, 1000);
        } else {
            const error = await response.json();
            console.error('Login error:', error);
            showToast('❌ ' + (error.detail || 'Ошибка входа'), true);
            submitBtn.classList.remove('btn-loading');
            submitBtn.disabled = false;
        }
    } catch (error) {
        console.error('Connection error:', error);
        showToast('❌ Ошибка соединения: ' + error.message, true);
        submitBtn.classList.remove('btn-loading');
        submitBtn.disabled = false;
    }
}

// Регистрация
async function handleRegister(event) {
    event.preventDefault();
    
    const form = event.target;
    const email = document.getElementById('register-email').value;
    const password = document.getElementById('register-password').value;
    const confirmPassword = document.getElementById('register-confirm').value;
    const fullName = document.getElementById('register-name').value;
    const inn = document.getElementById('register-inn').value;
    const phone = document.getElementById('register-phone').value;
    const submitBtn = form.querySelector('button[type="submit"]');
    
    // Проверка паролей
    if (password !== confirmPassword) {
        showToast('❌ Пароли не совпадают', true);
        return;
    }
    
    // Проверка длины пароля
    if (password.length < 8) {
        showToast('❌ Пароль должен быть минимум 8 символов', true);
        return;
    }
    
    // Показываем загрузку
    submitBtn.classList.add('btn-loading');
    submitBtn.disabled = true;
    
    try {
        const response = await fetch('/api/v1/auth/register', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                email: email,
                password: password,
                full_name: fullName,
                inn: inn,
                phone: phone
            })
        });
        
        if (response.ok) {
            showToast('✅ Регистрация успешна! Теперь войдите...');
            
            // Переключаем на вкладку входа
            setTimeout(() => {
                showTab('login');
                document.getElementById('login-email').value = email;
                submitBtn.classList.remove('btn-loading');
                submitBtn.disabled = false;
            }, 1500);
        } else {
            const error = await response.json();
            showToast('❌ ' + (error.detail || 'Ошибка регистрации'), true);
            submitBtn.classList.remove('btn-loading');
            submitBtn.disabled = false;
        }
    } catch (error) {
        showToast('❌ Ошибка соединения', true);
        submitBtn.classList.remove('btn-loading');
        submitBtn.disabled = false;
    }
}

// Уведомления
function showToast(message, isError = false) {
    const toast = document.createElement('div');
    toast.className = 'toast' + (isError ? ' error' : '');
    toast.textContent = message;
    document.body.appendChild(toast);
    
    setTimeout(() => {
        toast.remove();
    }, 3000);
}

// Проверка токена при загрузке
window.addEventListener('load', () => {
    const token = localStorage.getItem('access_token');
    
    if (token) {
        // Проверяем валидность токена
        fetch('/api/v1/auth/me', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        })
        .then(response => {
            if (response.ok) {
                // Токен валиден, перенаправляем в кабинет
                window.location.href = '/';
            } else {
                // Токен невалиден, удаляем
                localStorage.removeItem('access_token');
            }
        })
        .catch(() => {
            localStorage.removeItem('access_token');
        });
    }
});

// Инициализация
document.addEventListener('DOMContentLoaded', () => {
    // Показываем вкладку входа по умолчанию
    showTab('login');
});
