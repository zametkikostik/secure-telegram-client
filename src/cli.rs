//! CLI интерфейс для интерактивного взаимодействия
//!
//! Позволяет пользователю взаимодействовать с клиентом через консоль.

use anyhow::Result;
use crate::tdlib_wrapper::client::{TdClient, TdEvent};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use log::{info, error};

/// CLI команда
#[derive(Debug, Clone)]
pub enum Command {
    /// Помощь
    Help,
    /// Выход
    Quit,
    /// Авторизация по номеру
    Auth { phone: String },
    /// Ввод кода
    Code { code: String },
    /// Ввод пароля 2FA
    Password { password: String },
    /// Отправка сообщения
    Send { chat_id: i64, text: String },
    /// Список чатов
    Chats { limit: i32 },
    /// История сообщений
    History { chat_id: i64, limit: i32 },
    /// Статус соединения
    Status,
    /// Неизвестная команда
    Unknown(String),
}

/// Парсинг команды
pub fn parse_command(input: &str) -> Command {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    
    if parts.is_empty() {
        return Command::Unknown(String::new());
    }
    
    match parts[0].to_lowercase().as_str() {
        "help" | "h" | "?" => Command::Help,
        "quit" | "exit" | "q" => Command::Quit,
        "auth" | "login" => {
            if parts.len() > 1 {
                Command::Auth { phone: parts[1].to_string() }
            } else {
                Command::Unknown("auth <phone>".to_string())
            }
        }
        "code" | "verify" => {
            if parts.len() > 1 {
                Command::Code { code: parts[1].to_string() }
            } else {
                Command::Unknown("code <code>".to_string())
            }
        }
        "password" | "pass" | "2fa" => {
            if parts.len() > 1 {
                Command::Password { password: parts[1].to_string() }
            } else {
                Command::Unknown("password <password>".to_string())
            }
        }
        "send" | "msg" => {
            if parts.len() > 2 {
                if let Ok(chat_id) = parts[1].parse::<i64>() {
                    let text = parts[2..].join(" ");
                    Command::Send { chat_id, text }
                } else {
                    Command::Unknown("send <chat_id> <text>".to_string())
                }
            } else {
                Command::Unknown("send <chat_id> <text>".to_string())
            }
        }
        "chats" | "list" => {
            let limit = parts.get(1).and_then(|s| s.parse::<i32>().ok()).unwrap_or(20);
            Command::Chats { limit }
        }
        "history" | "msgs" => {
            if parts.len() > 1 {
                if let Ok(chat_id) = parts[1].parse::<i64>() {
                    let limit = parts.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(20);
                    Command::History { chat_id, limit }
                } else {
                    Command::Unknown("history <chat_id> [limit]".to_string())
                }
            } else {
                Command::Unknown("history <chat_id> [limit]".to_string())
            }
        }
        "status" => Command::Status,
        _ => Command::Unknown(input.to_string()),
    }
}

/// Запуск CLI
pub async fn run_cli(client: &mut TdClient) -> Result<()> {
    let mut stdout = tokio::io::stdout();
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);

    // Получаем канал событий
    let mut event_receiver = client.get_event_receiver().await;

    // Отправитель событий
    let event_sender = client.get_event_sender();

    // Запускаем обработчик событий в фоне
    let event_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Вывод приветствия
    use tokio::io::AsyncWriteExt;
    stdout.write_all(format!("\n🔐 Secure Telegram Client v{}\n", env!("CARGO_PKG_VERSION")).as_bytes()).await?;
    stdout.write_all(b"==========================================\n").await?;
    stdout.write_all(b"\xd0\x92\xd0\xb2\xd0\xb5\xd0\xb4\xd0\xb8\xd1\x82\xd0\xb5 'help' \xd0\xb4\xd0\xbb\xd1\x8f \xd1\x81\xd0\xbf\xd0\xb8\xd1\x81\xd0\xba\xd0\xb0 \xd0\xba\xd0\xbe\xd0\xbc\xd0\xb0\xd0\xbd\xd0\xb4\n\n").await?;
    stdout.flush().await?;

    let mut line = String::new();
    loop {
        // Чтение команды
        stdout.write_all(b"> ").await?;
        stdout.flush().await?;

        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        
        if bytes_read == 0 {
            break; // EOF
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        // Парсинг команды
        let command = parse_command(input);
        
        // Выполнение
        match command {
            Command::Help => {
                print_help(&mut stdout).await?;
            }
            Command::Quit => {
                stdout.write_all(b"\xd0\x92\xd1\x8b\xd1\x85\xd0\xbe\xd0\xb4...\n").await?;
                break;
            }
            Command::Auth { phone } => {
                match client.start_auth(&phone).await {
                    Ok(msg) => {
                        stdout.write_all(format!("✅ {}\n", msg).as_bytes()).await?;
                    }
                    Err(e) => {
                        stdout.write_all(format!("❌ Ошибка: {}\n", e).as_bytes()).await?;
                    }
                }
            }
            Command::Code { code } => {
                match client.check_code(&code).await {
                    Ok(msg) => {
                        stdout.write_all(format!("✅ {}\n", msg).as_bytes()).await?;
                    }
                    Err(e) => {
                        stdout.write_all(format!("❌ Ошибка: {}\n", e).as_bytes()).await?;
                    }
                }
            }
            Command::Password { password } => {
                match client.check_password(&password).await {
                    Ok(msg) => {
                        stdout.write_all(format!("✅ {}\n", msg).as_bytes()).await?;
                    }
                    Err(e) => {
                        stdout.write_all(format!("❌ Ошибка: {}\n", e).as_bytes()).await?;
                    }
                }
            }
            Command::Send { chat_id, text } => {
                match client.send_message(chat_id, &text).await {
                    Ok(_) => {
                        stdout.write_all(b"\xe2\x9c\x85 \xd0\xa1\xd0\xbe\xd0\xbe\xd0\xb1\xd1\x89\xd0\xb5\xd0\xbd\xd0\xb8\xd0\xb5 \xd0\xbe\xd1\x82\xd0\xbf\xd1\x80\xd0\xb0\xd0\xb2\xd0\xbb\xd0\xb5\xd0\xbd\xd0\xbe\n").await?;
                    }
                    Err(e) => {
                        stdout.write_all(format!("❌ Ошибка: {}\n", e).as_bytes()).await?;
                    }
                }
            }
            Command::Chats { limit } => {
                match client.get_chats(limit).await {
                    Ok(chats) => {
                        if chats.is_empty() {
                            stdout.write_all(b"\xf0\x9f\x93\xad \xd0\xa7\xd0\xb0\xd1\x82\xd0\xbe\xd0\xb2 \xd0\xbd\xd0\xb5\xd1\x82\n").await?;
                        } else {
                            stdout.write_all(b"\xf0\x9f\x93\x8b \xd0\xa7\xd0\xb0\xd1\x82\xd1\x8b:\n").await?;
                            for chat in chats {
                                let line = format!("  {}: {} - {:?}\n", chat.id, chat.title, chat.last_message);
                                stdout.write_all(line.as_bytes()).await?;
                            }
                        }
                    }
                    Err(e) => {
                        stdout.write_all(format!("❌ Ошибка: {}\n", e).as_bytes()).await?;
                    }
                }
            }
            Command::History { chat_id, limit } => {
                match client.get_messages(chat_id, limit).await {
                    Ok(messages) => {
                        if messages.is_empty() {
                            stdout.write_all(b"\xf0\x9f\x93\xad \xd0\xa1\xd0\xbe\xd0\xbe\xd0\xb1\xd1\x89\xd0\xb5\xd0\xbd\xd0\xb8\xd0\xb9 \xd0\xbd\xd0\xb5\xd1\x82\n").await?;
                        } else {
                            let line = format!("📋 История чата {}:\n", chat_id);
                            stdout.write_all(line.as_bytes()).await?;
                            for msg in messages {
                                let line = format!("  [{}] {}: {}\n", msg.id, msg.from_user_id, msg.text);
                                stdout.write_all(line.as_bytes()).await?;
                            }
                        }
                    }
                    Err(e) => {
                        stdout.write_all(format!("❌ Ошибка: {}\n", e).as_bytes()).await?;
                    }
                }
            }
            Command::Status => {
                let status = if client.is_authorized().await { "✅ Авторизован" } else { "❌ Не авторизован" };
                let line = format!("📊 Статус: {}\n", status);
                stdout.write_all(line.as_bytes()).await?;
            }
            Command::Unknown(cmd) => {
                let line = format!("❓ Неизвестная команда: '{}'\n", cmd);
                stdout.write_all(line.as_bytes()).await?;
                stdout.write_all(b"\xd0\x92\xd0\xb2\xd0\xb5\xd0\xb4\xd0\xb8\xd1\x82\xd0\xb5 'help' \xd0\xb4\xd0\xbb\xd1\x8f \xd1\x81\xd0\xbf\xd0\xb8\xd1\x81\xd0\xba\xd0\xb0 \xd0\xba\xd0\xbe\xd0\xbc\xd0\xb0\xd0\xbd\xd0\xb4\n").await?;
            }
        }
        stdout.flush().await?;
    }
    
    // Закрытие клиента
    client.close().await?;
    event_handle.abort();
    
    Ok(())
}

/// Вывод справки
async fn print_help<W: AsyncWriteExt + Unpin>(writer: &mut W) -> Result<()> {
    let help_text = r#"
📋 Доступные команды:

  🔐 Авторизация:
    auth <phone>       - Начать авторизацию по номеру телефона
    code <code>        - Ввести код подтверждения
    password <pass>    - Ввести пароль 2FA

  💬 Сообщения:
    send <id> <text>   - Отправить сообщение в чат
    chats [limit]      - Показать список чатов
    history <id> [n]   - Показать историю чата (n сообщений)

  ℹ️  Другое:
    status             - Показать статус подключения
    help               - Показать эту справку
    quit               - Выйти из программы

Примеры:
  auth +79991234567    - Авторизация по номеру
  code 12345           - Ввод кода из SMS
  send 12345678 Привет - Отправить сообщение в чат 12345678
  chats 10             - Показать 10 последних чатов
"#;
    writer.write_all(help_text.as_bytes()).await?;
    Ok(())
}
