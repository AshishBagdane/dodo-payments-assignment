use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc};

/// Type alias for the rate limiter.
/// We use a DirectRateLimiter which stores state in memory.
type RateLimiterType = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Middleware structure that holds the rate limiting state.
#[derive(Clone)]
pub struct RateLimitLayer {
    /// Map of IP addresses to their individual rate limiters.
    limiters: Arc<DashMap<std::net::IpAddr, Arc<RateLimiterType>>>,
    /// Requests allowed per hour.
    requests_per_hour: u32,
}

impl RateLimitLayer {
    /// Create a new RateLimitLayer with the specified requests per hour.
    pub fn new(requests_per_hour: u32) -> Self {
        Self {
            limiters: Arc::new(DashMap::new()),
            requests_per_hour,
        }
    }

    /// Middleware handler function.
    pub async fn handle(
        axum::extract::State(state): axum::extract::State<RateLimitLayer>,
        req: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract IP address from connection info
        // Note: In production with a reverse proxy, you'd check X-Forwarded-For
        let ip = match req.extensions().get::<ConnectInfo<SocketAddr>>() {
            Some(ConnectInfo(addr)) => addr.ip(),
            None => {
                // If we can't determine IP, we might choose to block or allow.
                // For safety, let's log and allow, or block.
                // Here we block securely.
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        // Get or create rate limiter for this IP
        let limiter = state.limiters.entry(ip).or_insert_with(|| {
            let quota = Quota::per_hour(NonZeroU32::new(state.requests_per_hour).expect("Rate limit must be > 0"));
            Arc::new(RateLimiter::direct(quota))
        }).clone();

        // Check if request is allowed
        if let Err(_) = limiter.check() {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        Ok(next.run(req).await)
    }
}
