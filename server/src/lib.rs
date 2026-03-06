// server/src/lib.rs
//! Liberty Reach Server Library
//! 
//! Этот модуль предоставляет основные компоненты сервера Liberty Reach

pub mod api;
pub mod auth;
pub mod db;
pub mod middleware;
pub mod websocket;

// Ре-экспорт для тестов
pub use api;
pub use auth;
pub use db;
