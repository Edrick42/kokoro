pub const JWT_SECRET: &str = "kokoro-dev-secret-change-in-production";

pub const TOKEN_EXPIRY_SECS: i64 = 86400;  // Tokens expire after 24 hours

pub mod routes {
    pub const LOGIN: &str = "/auth/login";
    #[allow(dead_code)]
    pub const LOGOUT: &str = "/auth/logout";
    pub const REGISTER: &str = "/auth/register";
    #[allow(dead_code)]
    pub const REFRESH: &str = "/auth/refresh";
    pub const PROFILE: &str = "/auth/profile";
}