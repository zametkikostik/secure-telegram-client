package com.example.securemessenger.service

import android.app.*
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import com.example.securemessenger.MainActivity
import com.example.securemessenger.rust.Core
import kotlinx.coroutines.*

/**
 * Сервис для проверки обновлений
 */
class UpdaterService : Service() {
    
    private val serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        intent?.action?.let {
            when (it) {
                ACTION_CHECK_UPDATE -> checkForUpdates()
            }
        }
        
        return START_STICKY
    }
    
    override fun onBind(intent: Intent?): IBinder? = null
    
    override fun onDestroy() {
        super.onDestroy()
        serviceScope.cancel()
    }
    
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
    
    private fun checkForUpdates() {
        serviceScope.launch {
            try {
                Log.d(TAG, "Checking for updates...")
                val updateInfo = Core.checkForUpdates()
                
                if (updateInfo.isNotEmpty()) {
                    showUpdateNotification(updateInfo)
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error checking updates", e)
            }
        }
    }
    
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
            .setSmallIcon(android.R.drawable.ic_dialog_alert)
            .setContentTitle("Доступно обновление")
            .setContentText(updateInfo)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .setContentIntent(pendingIntent)
            .setAutoCancel(true)
            .build()
        
        val notificationManager = getSystemService(NotificationManager::class.java)
        notificationManager.notify(NOTIFICATION_ID, notification)
    }
    
    companion object {
        private const val TAG = "UpdaterService"
        private const val CHANNEL_ID = "updater_channel"
        private const val NOTIFICATION_ID = 1001
        const val ACTION_CHECK_UPDATE = "com.example.securemessenger.CHECK_UPDATE"
        
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
