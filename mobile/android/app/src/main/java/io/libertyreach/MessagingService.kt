package io.libertyreach

import android.util.Log
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import android.app.NotificationManager
import android.app.NotificationChannel
import android.app.PendingIntent
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat

class MessagingService : FirebaseMessagingService() {

    companion object {
        private const val TAG = "LibertyReachFCM"
        private const val CHANNEL_ID = "liberty_reach_channel"
    }

    override fun onMessageReceived(message: RemoteMessage) {
        Log.d(TAG, "Получено сообщение: ${message.data}")

        message.notification?.let {
            showNotification(
                title = it.title ?: "Liberty Reach",
                body = it.body ?: "",
                data = message.data
            )
        }
    }

    override fun onNewToken(token: String) {
        Log.d(TAG, "Новый FCM токен: $token")
        // Отправить токен на сервер
        sendRegistrationToServer(token)
    }

    private fun showNotification(
        title: String,
        body: String,
        data: Map<String, String>
    ) {
        val notificationManager = getSystemService(NOTIFICATION_SERVICE) as NotificationManager

        // Создание канала уведомлений
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Сообщения Liberty Reach",
                NotificationManager.IMPORTANCE_HIGH
            ).apply {
                description = "Уведомления о новых сообщениях"
            }
            notificationManager.createNotificationChannel(channel)
        }

        // Intent для открытия приложения
        val intent = Intent(this, MainActivity::class.java).apply {
            putExtra("chat_id", data["chat_id"])
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
        }

        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        // Создание уведомления
        val notification = NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notification)
            .setContentTitle(title)
            .setContentText(body)
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setAutoCancel(true)
            .setContentIntent(pendingIntent)
            .build()

        notificationManager.notify(System.currentTimeMillis().toInt(), notification)
    }

    private fun sendRegistrationToServer(token: String) {
        // Отправка токена на backend сервер
        // Для сохранения и отправки push-уведомлений
        Log.d(TAG, "Отправка токена на сервер: $token")
    }
}
