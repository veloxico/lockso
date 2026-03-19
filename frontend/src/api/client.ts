import { useAuthStore } from "../stores/auth";

const BASE_URL = "/v1";

/** HTTP methods that require CSRF protection. */
const CSRF_METHODS = new Set(["POST", "PUT", "PATCH", "DELETE"]);

/** Paths that are exempt from CSRF (unauthenticated endpoints). */
const CSRF_EXEMPT_PATHS = new Set([
  "/users/login",
  "/users/register",
  "/sessions/refresh",
]);

interface ApiError {
  status: number;
  message: string;
  code?: string;
}

class ApiClient {
  private baseUrl: string;
  private refreshPromise: Promise<boolean> | null = null;
  private csrfToken: string | null = null;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    isRetry = false,
  ): Promise<T> {
    // Auto-refresh token if about to expire (within 60 seconds)
    if (!isRetry) {
      await this.ensureValidToken();
    }

    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    // Always include auth header if we have a token (even on retry)
    const token = useAuthStore.getState().accessToken;
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    // Include CSRF token on state-changing requests (POST/PUT/PATCH/DELETE)
    if (CSRF_METHODS.has(method) && !CSRF_EXEMPT_PATHS.has(path) && token) {
      const csrf = await this.ensureCsrfToken();
      if (csrf) {
        headers["X-CSRF-Token"] = csrf;
      }
    }

    const response = await fetch(`${this.baseUrl}${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
    });

    // Handle 401 — try refresh once then retry (but don't recurse indefinitely)
    if (response.status === 401 && !isRetry) {
      // Invalidate CSRF token since session may have changed
      this.csrfToken = null;
      const refreshed = await this.tryRefresh();
      if (refreshed) {
        return this.request(method, path, body, true);
      }
      useAuthStore.getState().logout();
      throw { status: 401, message: "Session expired", code: "SESSION_EXPIRED" };
    }

    if (!response.ok) {
      const error: ApiError = {
        status: response.status,
        message: response.statusText,
      };
      try {
        const data = await response.json();
        error.message = data.message || error.message;
        error.code = data.code;
      } catch {
        // Response body is not JSON
      }

      // If CSRF token was invalid, clear it and retry once with a fresh token
      if (error.code === "CSRF_TOKEN_INVALID" && !isRetry) {
        this.csrfToken = null;
        return this.request(method, path, body, true);
      }

      // 2FA required — redirect to verification page
      if (error.code === "TWO_FACTOR_REQUIRED") {
        useAuthStore.getState().requireTwoFactor();
        throw error;
      }

      throw error;
    }

    // CSRF tokens are single-use — clear after successful state-changing request
    if (CSRF_METHODS.has(method) && !CSRF_EXEMPT_PATHS.has(path)) {
      this.csrfToken = null;
    }

    if (response.status === 204) {
      return undefined as T;
    }

    // Handle empty body (e.g. 200 with no content from Axum `Result<()>`)
    const text = await response.text();
    if (!text) {
      return undefined as T;
    }

    return JSON.parse(text);
  }

  /**
   * Ensure we have a valid CSRF token.
   * Fetches one from the backend if needed.
   */
  private async ensureCsrfToken(): Promise<string | null> {
    if (this.csrfToken) return this.csrfToken;

    const token = useAuthStore.getState().accessToken;
    if (!token) return null;

    try {
      const response = await fetch(`${this.baseUrl}/csrf-tokens/generate`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) return null;

      const data = await response.json();
      this.csrfToken = data.token;
      return this.csrfToken;
    } catch {
      return null;
    }
  }

  private async ensureValidToken(): Promise<void> {
    const state = useAuthStore.getState();
    if (!state.accessToken || !state.accessTokenExpiresAt) return;

    const expiresIn = state.accessTokenExpiresAt - Date.now();
    if (expiresIn < 60_000) {
      await this.tryRefresh();
    }
  }

  private async tryRefresh(): Promise<boolean> {
    // Deduplicate concurrent refresh attempts
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    const state = useAuthStore.getState();
    if (!state.refreshToken) return false;

    this.refreshPromise = (async (): Promise<boolean> => {
      try {
        const response = await fetch(`${this.baseUrl}/sessions/refresh`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ refreshToken: state.refreshToken }),
        });

        if (!response.ok) {
          useAuthStore.getState().logout();
          return false;
        }

        const data = await response.json();
        useAuthStore.getState().updateTokens(
          data.accessToken,
          data.refreshToken,
          data.accessTokenExpiredAt,
          data.refreshTokenExpiredAt,
        );
        // New session = new CSRF token needed
        this.csrfToken = null;
        return true;
      } catch {
        useAuthStore.getState().logout();
        return false;
      }
    })();

    const result = await this.refreshPromise;
    this.refreshPromise = null;
    return result;
  }

  get<T>(path: string): Promise<T> {
    return this.request<T>("GET", path);
  }

  post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("POST", path, body);
  }

  put<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("PUT", path, body);
  }

  patch<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>("PATCH", path, body);
  }

  delete<T>(path: string): Promise<T> {
    return this.request<T>("DELETE", path);
  }
}

export const api = new ApiClient(BASE_URL);
