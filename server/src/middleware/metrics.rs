//! Prometheus Metrics Middleware
//!
//! Exports metrics for monitoring and alerting:
//! - HTTP request duration histogram
//! - Active connections gauge
//! - Authentication success/failure counter
//! - Database query duration
//! - Cache hit/miss ratio
//! - Message throughput
//! - WebSocket connections

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use prometheus::{
    HistogramVec, IntCounterVec, IntGaugeVec, Registry,
    HistogramOpts, Opts,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

// ============================================================================
// Metrics Collector
// ============================================================================

#[derive(Clone)]
pub struct MetricsCollector {
    pub registry: Registry,

    // HTTP metrics
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_requests_in_flight: IntGaugeVec,

    // Authentication metrics
    pub auth_attempts_total: IntCounterVec,
    pub auth_failures_total: IntCounterVec,

    // Database metrics
    pub db_query_duration_seconds: HistogramVec,
    pub db_connections_active: IntGaugeVec,

    // Cache metrics
    pub cache_hits_total: IntCounterVec,
    pub cache_misses_total: IntCounterVec,

    // Message metrics
    pub messages_sent_total: IntCounterVec,
    pub messages_received_total: IntCounterVec,
    pub message_size_bytes: HistogramVec,

    // WebSocket metrics
    pub ws_connections_active: IntGaugeVec,
    pub ws_messages_in: IntCounterVec,
    pub ws_messages_out: IntCounterVec,

    // Business metrics
    pub active_users: IntGaugeVec,
    pub p2p_messages_routed: IntCounterVec,
    pub cloudflare_fallbacks: IntCounterVec,

    // Enterprise metrics
    pub sso_logins_total: IntCounterVec,
    pub admin_actions_total: IntCounterVec,
    pub compliance_events_total: IntCounterVec,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let registry = Registry::new();

        // HTTP metrics
        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "path", "status"]
        ).unwrap();

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "path"]
        ).unwrap();

        let http_requests_in_flight = IntGaugeVec::new(
            Opts::new("http_requests_in_flight", "HTTP requests currently being processed"),
            &["method", "path"]
        ).unwrap();

        // Authentication metrics
        let auth_attempts_total = IntCounterVec::new(
            Opts::new("auth_attempts_total", "Total authentication attempts"),
            &["provider", "success"]
        ).unwrap();

        let auth_failures_total = IntCounterVec::new(
            Opts::new("auth_failures_total", "Total authentication failures"),
            &["provider", "reason"]
        ).unwrap();

        // Database metrics
        let db_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "db_query_duration_seconds",
                "Database query duration in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["operation"]
        ).unwrap();

        let db_connections_active = IntGaugeVec::new(
            Opts::new("db_connections_active", "Active database connections"),
            &["pool"]
        ).unwrap();

        // Cache metrics
        let cache_hits_total = IntCounterVec::new(
            Opts::new("cache_hits_total", "Total cache hits"),
            &["type"]
        ).unwrap();

        let cache_misses_total = IntCounterVec::new(
            Opts::new("cache_misses_total", "Total cache misses"),
            &["type"]
        ).unwrap();

        // Message metrics
        let messages_sent_total = IntCounterVec::new(
            Opts::new("messages_sent_total", "Total messages sent"),
            &["chat_type"]
        ).unwrap();

        let messages_received_total = IntCounterVec::new(
            Opts::new("messages_received_total", "Total messages received"),
            &["chat_type"]
        ).unwrap();

        let message_size_bytes = HistogramVec::new(
            HistogramOpts::new(
                "message_size_bytes",
                "Message size in bytes"
            ).buckets(vec![100.0, 500.0, 1000.0, 5000.0, 10000.0, 50000.0, 100000.0]),
            &["type"]
        ).unwrap();

        // WebSocket metrics
        let ws_connections_active = IntGaugeVec::new(
            Opts::new("ws_connections_active", "Active WebSocket connections"),
            &[]
        ).unwrap();

        let ws_messages_in = IntCounterVec::new(
            Opts::new("ws_messages_in_total", "Total WebSocket messages received"),
            &[]
        ).unwrap();

        let ws_messages_out = IntCounterVec::new(
            Opts::new("ws_messages_out_total", "Total WebSocket messages sent"),
            &[]
        ).unwrap();

        // Business metrics
        let active_users = IntGaugeVec::new(
            Opts::new("active_users", "Active users in the last 5 minutes"),
            &[]
        ).unwrap();

        let p2p_messages_routed = IntCounterVec::new(
            Opts::new("p2p_messages_routed_total", "P2P messages successfully routed"),
            &[]
        ).unwrap();

        let cloudflare_fallbacks = IntCounterVec::new(
            Opts::new("cloudflare_fallbacks_total", "Messages routed via Cloudflare fallback"),
            &["status"]
        ).unwrap();

        // Enterprise metrics
        let sso_logins_total = IntCounterVec::new(
            Opts::new("sso_logins_total", "Total SSO login attempts"),
            &["provider", "success"]
        ).unwrap();

        let admin_actions_total = IntCounterVec::new(
            Opts::new("admin_actions_total", "Total admin panel actions"),
            &["action", "success"]
        ).unwrap();

        let compliance_events_total = IntCounterVec::new(
            Opts::new("compliance_events_total", "Total compliance events"),
            &["type", "regulation"]
        ).unwrap();

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone())).unwrap();
        registry.register(Box::new(http_request_duration_seconds.clone())).unwrap();
        registry.register(Box::new(http_requests_in_flight.clone())).unwrap();
        registry.register(Box::new(auth_attempts_total.clone())).unwrap();
        registry.register(Box::new(auth_failures_total.clone())).unwrap();
        registry.register(Box::new(db_query_duration_seconds.clone())).unwrap();
        registry.register(Box::new(db_connections_active.clone())).unwrap();
        registry.register(Box::new(cache_hits_total.clone())).unwrap();
        registry.register(Box::new(cache_misses_total.clone())).unwrap();
        registry.register(Box::new(messages_sent_total.clone())).unwrap();
        registry.register(Box::new(messages_received_total.clone())).unwrap();
        registry.register(Box::new(message_size_bytes.clone())).unwrap();
        registry.register(Box::new(ws_connections_active.clone())).unwrap();
        registry.register(Box::new(ws_messages_in.clone())).unwrap();
        registry.register(Box::new(ws_messages_out.clone())).unwrap();
        registry.register(Box::new(active_users.clone())).unwrap();
        registry.register(Box::new(p2p_messages_routed.clone())).unwrap();
        registry.register(Box::new(cloudflare_fallbacks.clone())).unwrap();
        registry.register(Box::new(sso_logins_total.clone())).unwrap();
        registry.register(Box::new(admin_actions_total.clone())).unwrap();
        registry.register(Box::new(compliance_events_total.clone())).unwrap();

        Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
            auth_attempts_total,
            auth_failures_total,
            db_query_duration_seconds,
            db_connections_active,
            cache_hits_total,
            cache_misses_total,
            messages_sent_total,
            messages_received_total,
            message_size_bytes,
            ws_connections_active,
            ws_messages_in,
            ws_messages_out,
            active_users,
            p2p_messages_routed,
            cloudflare_fallbacks,
            sso_logins_total,
            admin_actions_total,
            compliance_events_total,
        }
    }

    // ========================================================================
    // Convenience methods
    // ========================================================================

    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        self.http_requests_total
            .with_label_values(&[method, path, &status.to_string()])
            .inc();

        self.http_request_duration_seconds
            .with_label_values(&[method, path])
            .observe(duration_secs);
    }

    pub fn increment_requests_in_flight(&self, method: &str, path: &str) {
        self.http_requests_in_flight
            .with_label_values(&[method, path])
            .inc();
    }

    pub fn decrement_requests_in_flight(&self, method: &str, path: &str) {
        self.http_requests_in_flight
            .with_label_values(&[method, path])
            .dec();
    }

    pub fn record_auth_attempt(&self, provider: &str, success: bool) {
        self.auth_attempts_total
            .with_label_values(&[provider, &success.to_string()])
            .inc();

        if !success {
            self.auth_failures_total
                .with_label_values(&[provider, "invalid_credentials"])
                .inc();
        }
    }

    pub fn record_db_query(&self, operation: &str, duration_secs: f64) {
        self.db_query_duration_seconds
            .with_label_values(&[operation])
            .observe(duration_secs);
    }

    pub fn record_cache_hit(&self, cache_type: &str) {
        self.cache_hits_total
            .with_label_values(&[cache_type])
            .inc();
    }

    pub fn record_cache_miss(&self, cache_type: &str) {
        self.cache_misses_total
            .with_label_values(&[cache_type])
            .inc();
    }

    pub fn record_message_sent(&self, chat_type: &str, size_bytes: usize) {
        self.messages_sent_total
            .with_label_values(&[chat_type])
            .inc();

        self.message_size_bytes
            .with_label_values(&[chat_type])
            .observe(size_bytes as f64);
    }

    pub fn set_ws_connections(&self, count: i64) {
        self.ws_connections_active
            .with_label_values(&[])
            .set(count);
    }

    pub fn inc_ws_messages_in(&self) {
        self.ws_messages_in
            .with_label_values(&[])
            .inc();
    }

    pub fn inc_ws_messages_out(&self) {
        self.ws_messages_out
            .with_label_values(&[])
            .inc();
    }

    pub fn set_active_users(&self, count: i64) {
        self.active_users
            .with_label_values(&[])
            .set(count);
    }

    pub fn inc_p2p_messages_routed(&self) {
        self.p2p_messages_routed
            .with_label_values(&[])
            .inc();
    }

    pub fn record_cloudflare_fallback(&self, status: &str) {
        self.cloudflare_fallbacks
            .with_label_values(&[status])
            .inc();
    }

    pub fn record_sso_login(&self, provider: &str, success: bool) {
        self.sso_logins_total
            .with_label_values(&[provider, &success.to_string()])
            .inc();
    }

    pub fn record_admin_action(&self, action: &str, success: bool) {
        self.admin_actions_total
            .with_label_values(&[action, &success.to_string()])
            .inc();
    }

    pub fn record_compliance_event(&self, event_type: &str, regulation: &str) {
        self.compliance_events_total
            .with_label_values(&[event_type, regulation])
            .inc();
    }
}

// ============================================================================
// Axum Middleware
// ============================================================================

/// HTTP request metrics middleware
pub async fn metrics_middleware(
    State(metrics): State<Arc<MetricsCollector>>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();

    metrics.increment_requests_in_flight(&method, &path);

    let start = Instant::now();

    let response = next.run(request).await;

    let duration_secs = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();

    metrics.decrement_requests_in_flight(&method, &path);
    metrics.record_http_request(&method, &path, status, duration_secs);

    response
}

/// Metrics endpoint handler
pub async fn metrics_handler(
    State(metrics): State<Arc<MetricsCollector>>,
) -> impl IntoResponse {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    encoder.encode(&metrics.registry.gather(), &mut buffer)
        .unwrap_or_default();

    let body = String::from_utf8(buffer).unwrap_or_default();

    (
        axum::http::StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(!collector.registry.gather().is_empty());
    }

    #[test]
    fn test_http_request_recording() {
        let collector = MetricsCollector::new();

        collector.record_http_request("GET", "/api/v1/users", 200, 0.05);
        collector.record_http_request("POST", "/api/v1/messages", 201, 0.12);

        let metrics = collector.registry.gather();
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_auth_attempt_recording() {
        let collector = MetricsCollector::new();

        collector.record_auth_attempt("jwt", true);
        collector.record_auth_attempt("ldap", false);

        // Verify counters incremented
        let metrics = collector.registry.gather();
        let auth_total = metrics.iter()
            .find(|m| m.get_name() == "auth_attempts_total")
            .unwrap();
        assert_eq!(auth_total.get_metric().len(), 2); // jwt+true, ldap+false
    }

    #[test]
    fn test_cache_recording() {
        let collector = MetricsCollector::new();

        collector.record_cache_hit("redis");
        collector.record_cache_miss("redis");

        assert_eq!(collector.cache_hits_total.with_label_values(&["redis"]).get(), 1);
        assert_eq!(collector.cache_misses_total.with_label_values(&["redis"]).get(), 1);
    }
}
