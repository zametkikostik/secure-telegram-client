package com.example.securemessenger

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.example.securemessenger.rust.Core
import android.util.Log

class MainActivity : AppCompatActivity() {
    
    companion object {
        private const val TAG = "MainActivity"
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Инициализация Rust ядра
        val initialized = Core.init()
        Log.i(TAG, "Rust Core initialized: $initialized")
        
        // Простой UI
        val textView = android.widget.TextView(this).apply {
            text = "Secure Messenger v0.2.0\n\nRust Core: ${if (initialized) "OK" else "FAILED"}"
            textSize = 18f
            setPadding(32, 32, 32, 32)
        }
        
        setContentView(textView)
        
        // Проверка обновлений
        checkForUpdates()
    }
    
    private fun checkForUpdates() {
        Thread {
            try {
                val updateInfo = Core.checkForUpdates()
                Log.i(TAG, "Update check result: $updateInfo")
            } catch (e: Exception) {
                Log.e(TAG, "Error checking updates", e)
            }
        }.start()
    }
}
