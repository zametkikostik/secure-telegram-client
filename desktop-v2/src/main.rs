#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

mod commands;
mod p2p;
mod crypto;
mod db;

fn main() {
    // Создаём меню системного трея
    let show = CustomMenuItem::new("show".to_string(), "Показать");
    let hide = CustomMenuItem::new("hide".to_string(), "Скрыть");
    let quit = CustomMenuItem::new("quit".to_string(), "Выход");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("Попытка запуска второй копии: {:?}, {:?}", argv, cwd);
            app.get_window("main")
                .unwrap()
                .set_focus()
                .unwrap();
        }))
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                if let Some(window) = app.get_window("main") {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_window("main") {
                        window.hide().unwrap();
                    }
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_version,
            commands::get_platform_info,
            commands::send_message,
            commands::get_messages,
            commands::create_chat,
            commands::join_p2p_network,
            commands::encrypt_message,
            commands::decrypt_message,
        ])
        .setup(|app| {
            // Инициализация P2P сети
            let p2p_handle = p2p::P2PNode::new()?;
            app.manage(p2p_handle);

            // Инициализация базы данных
            let db = db::Database::new()?;
            app.manage(db);

            // Автозапуск для Linux Mint
            #[cfg(target_os = "linux")]
            {
                println!("Запуск на Linux Mint");
                // Проверяем, есть ли файл .desktop
                let desktop_file = "/usr/share/applications/io.secure-telegram.desktop";
                if std::path::Path::new(desktop_file).exists() {
                    println!("Файл .desktop найден");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске приложения");
}
