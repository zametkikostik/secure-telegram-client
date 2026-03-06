// messenger/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod crypto;
mod webrtc;
mod p2p;
mod web3;
mod ai;
mod telegram;
mod chat;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Инициализация компонентов
            let handle = app.handle();
            
            // Здесь можно инициализировать AI translator, P2P сеть, и т.д.
            println!("Liberty Reach запущен!");
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске Liberty Reach");
}
