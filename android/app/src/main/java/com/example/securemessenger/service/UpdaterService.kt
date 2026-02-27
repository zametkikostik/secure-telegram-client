package com.example.securemessenger.service

import android.app.*
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import com.example.securemessenger.MainActivity
import com.example.securemessenger.R
import com.example.securemessenger.rust.Core
import kotlinx.coroutines.*
import java.io.File

/**
 * Сервис для проверки и загрузки обновлений через IPFS
 */
class UpdaterService : Service() {
    
    companion object {
        private const val TAG = "UpdaterService"
        private const val CHANNEL_ID = "updater_channel"
        private const val NOTIFICATION_ID = 1001
        private const val UPDATE_CHECK_INTERVAL = 24 * 60 * 60 * 1000L // 24 часа
    }
    
    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_CHECK_UPDATE -> checkForUpdates()
            ACTION_INSTALL_UPDATE -> installUpdate(intent.getStringExtra("apk_path"))
            else -> startPeriodicCheck()
        }
        
        return START_STICKY
    }
    
    override fun onBind(intent: Intent?): IBinder? = null
    
    override fun onDestroy() {
        super.onDestroy()
        serviceScope.cancel()
    }
    
    /**
     * Создание notification канала
     */
    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Обновления",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Уведомления об обновлениях приложения"
            }
            
            val notificationManager = getSystemService(NotificationManager::class.java)
            notificationManager.createNotificationChannel(channel)
        }
    }
    
    /**
     * Периодическая проверка обновлений
     */
    private fun startPeriodicCheck() {
        serviceScope.launch {
            while (isActive) {
                checkForUpdates()
                delay(UPDATE_CHECK_INTERVAL)
            }
        }
    }
    
    /**
     * Проверка обновлений через IPFS
     */
    private fun checkForUpdates() {
        serviceScope.launch {
            try {
                Log.d(TAG, "Checking for updates via IPFS...")
                
                // Вызов Rust функции
                val updateInfo = Core.checkForUpdates()
                
                if (updateInfo.isNotEmpty()) {
                    // Обновление доступно
                    showUpdateNotification(updateInfo)
                } else {
                    Log.d(TAG, "No updates available")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error checking updates", e)
            }
        }
    }
    
    /**
     * Показ уведомления об обновлении
     */
    private fun showUpdateNotification(updateInfo: String) {
        val intent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
        }
        
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            intent,
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        
        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentTitle("Доступно обновление")
            .setContentText("Нажмите для загрузки новой версии")
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setContentIntent(pendingIntent)
            .setAutoCancel(true)
            .build()
        
        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.notify(NOTIFICATION_ID, notification)
    }
    
    /**
     * Установка обновления
     */
    private fun installUpdate(apkPath: String?) {
        serviceScope.launch {
            try {
                if (apkPath == null) {
                    Log.e(TAG, "APK path is null")
                    return@launch
                }
                
                val apkFile = File(apkPath)
                if (!apkFile.exists()) {
                    Log.e(TAG, "APK file does not exist")
                    return@launch
                }
                
                // Запрос разрешения на установку
                val installIntent = Intent(Intent.ACTION_INSTALL_PACKAGE).apply {
                    data = android.net.Uri.fromFile(apkFile)
                    flags = Intent.FLAG_ACTIVITY_NEW_TASK
                    putExtra(Intent.EXTRA_NOT_UNKNOWN_SOURCE, true)
                }
                
                startActivity(installIntent)
                
                Log.i(TAG, "Update installation started")
            } catch (e: Exception) {
                Log.e(TAG, "Error installing update", e)
            }
        }
    }
    
    companion object {
        const val ACTION_CHECK_UPDATE = "com.example.securemessenger.CHECK_UPDATE"
        const val ACTION_INSTALL_UPDATE = "com.example.securemessenger.INSTALL_UPDATE"
        
        /**
         * Запуск сервиса
         */
        fun start(context: Context) {
            val intent = Intent(context, UpdaterService::class.java)
            intent.action = ACTION_CHECK_UPDATE
            
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }
    }
}
