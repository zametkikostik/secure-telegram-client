package com.example.securemessenger.rust

/**
 * JNI биндинги для вызова Rust функций из Android
 */
object Core {
    init {
        System.loadLibrary("secure_messenger_core")
    }
    
    /**
     * Инициализация ядра
     */
    external fun init(): Boolean
    
    /**
     * Отправка сообщения
     */
    external fun sendMessage(chatId: Long, text: String): Boolean
    
    /**
     * Проверка обновлений
     * @return JSON с информацией об обновлении или пустая строка
     */
    external fun checkForUpdates(): String
    
    /**
     * Подключение через obfs4
     */
    external fun connectObfs4(bridgeAddr: String, publicKey: String): Boolean
    
    /**
     * Проверка блокировок
     * @return Тип блокировки: "ok", "dns_blocked", "tcp_rst", "ip_blocked"
     */
    external fun checkBlockage(target: String): String
    
    /**
     * Генерация пары ключей
     * @return Строка формата "public_key:secret_key"
     */
    external fun generateKeyPair(): String
    
    /**
     * Подпись сообщения
     */
    external fun signMessage(message: String, secretKey: String): String
    
    /**
     * Верификация подписи
     * @return true если подпись валидна
     */
    external fun verifySignature(message: String, signature: String, publicKey: String): Boolean
}
