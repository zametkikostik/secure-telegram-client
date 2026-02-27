//! Shadowsocks транспорт
//! 
//! Прокси с шифрованием для обхода блокировок

use anyhow::{Result, Context};
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use tokio::net::TcpStream;

/// Shadowsocks транспорт
pub struct ShadowsocksTransport {
    /// Адрес сервера
    server_addr: SocketAddr,
    /// Метод шифрования
    method: String,
    /// Пароль
    password: String,
}

impl ShadowsocksTransport {
    /// Создание нового транспорта
    pub fn new(server_addr: SocketAddr, method: String, password: String) -> Self {
        Self {
            server_addr,
            method,
            password,
        }
    }

    /// Парсинг Shadowsocks URL
    /// Формат: ss://method:password@host:port
    pub fn from_url(url: &str) -> Result<Self> {
        if !url.starts_with("ss://") {
            anyhow::bail!("Неверный формат Shadowsocks URL");
        }

        let without_scheme = &url[5..];
        let parts: Vec<&str> = without_scheme.split('@').collect();

        if parts.len() != 2 {
            anyhow::bail!("Неверный формат: ожидается ss://method:password@host:port");
        }

        let method_pass = parts[0];
        let host_port = parts[1];

        let mp: Vec<&str> = method_pass.split(':').collect();
        if mp.len() != 2 {
            anyhow::bail!("Неверный формат метода и пароля");
        }

        let method = mp[0].to_string();
        let password = mp[1].to_string();
        let server_addr = host_port.parse::<SocketAddr>()
            .context("Неверный формат адреса сервера")?;

        Ok(Self::new(server_addr, method, password))
    }

    /// Подключение через Shadowsocks
    pub async fn connect(&self, target: &str) -> Result<ShadowsocksStream> {
        // 1. TCP подключение к серверу
        let stream = TcpStream::connect(self.server_addr)
            .await
            .context("Ошибка подключения к Shadowsocks серверу")?;

        // 2. Shadowsocks handshake (упрощённо)
        // В полной реализации здесь будет:
        // - SIP004 шифрование
        // - Отправка целевого адреса

        Ok(ShadowsocksStream {
            inner: stream,
            target: target.to_string(),
        })
    }
}

/// Shadowsocks стрим
pub struct ShadowsocksStream {
    inner: TcpStream,
    target: String,
}

impl AsyncRead for ShadowsocksStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for ShadowsocksStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadowsocks_url_parsing() {
        let url = "ss://chacha20-ietf-poly1305:password123@192.168.1.1:8388";
        let transport = ShadowsocksTransport::from_url(url).unwrap();

        assert_eq!(transport.method, "chacha20-ietf-poly1305");
        assert_eq!(transport.password, "password123");
    }

    #[test]
    fn test_invalid_url() {
        let url = "invalid://url";
        assert!(ShadowsocksTransport::from_url(url).is_err());
    }
}
