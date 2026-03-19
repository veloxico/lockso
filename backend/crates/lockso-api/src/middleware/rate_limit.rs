/// Redis-backed rate limiter using atomic INCR+EXPIRE via Lua script.
#[derive(Clone)]
pub struct RateLimiter {
    redis: redis::aio::ConnectionManager,
}

/// Lua script for atomic increment with expiry.
/// Returns the current count after increment.
/// If key doesn't exist, sets it to 1 with TTL.
/// If key exists, increments and returns new count.
const RATE_LIMIT_SCRIPT: &str = r#"
local current = redis.call('INCR', KEYS[1])
if current == 1 then
    redis.call('EXPIRE', KEYS[1], ARGV[1])
end
return current
"#;

impl RateLimiter {
    pub fn new(redis: redis::aio::ConnectionManager) -> Self {
        Self { redis }
    }

    /// Check if a request is allowed under the rate limit.
    ///
    /// Returns Ok(remaining) if allowed, Err(()) if limit exceeded.
    /// Uses an atomic Lua script to prevent INCR/EXPIRE race conditions.
    pub async fn check(&self, key: &str, max_requests: u64, window_secs: u64) -> Result<u64, ()> {
        let redis_key = format!("rl:{key}");
        let mut conn = self.redis.clone();

        let count: u64 = redis::Script::new(RATE_LIMIT_SCRIPT)
            .key(&redis_key)
            .arg(window_secs)
            .invoke_async(&mut conn)
            .await
            .unwrap_or(max_requests + 1); // Fail closed: deny on Redis error

        if count > max_requests {
            Err(())
        } else {
            Ok(max_requests - count)
        }
    }

    /// Check login rate limit for an IP address using lockout settings from DB.
    /// If lockout is disabled, always allows the request.
    pub async fn check_login_with_settings(
        &self,
        ip: &str,
        enabled: bool,
        max_attempts: u32,
        window_seconds: u64,
    ) -> Result<u64, ()> {
        if !enabled {
            return Ok(u64::MAX);
        }
        self.check(&format!("login:{ip}"), max_attempts as u64, window_seconds).await
    }

    /// Check login rate limit for an IP address (fallback with env-based defaults).
    /// Used when DB settings are unavailable.
    pub async fn check_login(&self, ip: &str) -> Result<u64, ()> {
        let max: u64 = std::env::var("RATE_LIMIT_LOGIN_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        self.check(&format!("login:{ip}"), max, 60).await
    }

    /// Check general API rate limit for an IP address.
    /// Default: 120 requests per 60 seconds.
    pub async fn check_api(&self, ip: &str) -> Result<u64, ()> {
        let max: u64 = std::env::var("RATE_LIMIT_API_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120);
        self.check(&format!("api:{ip}"), max, 60).await
    }
}
