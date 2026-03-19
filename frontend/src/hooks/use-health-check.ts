import { useEffect, useState, useRef } from "react";
import type { HealthResponse } from "@/types/api";

interface HealthCheckResult {
  data: HealthResponse | null;
  error: string | null;
  isLoading: boolean;
}

// Module-level cache — shared across all hook instances, fetched once per page load
let cachedData: HealthResponse | null = null;
let cachedError: string | null = null;
let fetchPromise: Promise<void> | null = null;

/**
 * Fetches system health once per page load and caches the result.
 * Used by route guards and loading page to determine routing.
 */
export function useHealthCheck(): HealthCheckResult {
  const [data, setData] = useState<HealthResponse | null>(cachedData);
  const [error, setError] = useState<string | null>(cachedError);
  const [isLoading, setIsLoading] = useState(!cachedData && !cachedError);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;

    // Already have cached result
    if (cachedData || cachedError) {
      setData(cachedData);
      setError(cachedError);
      setIsLoading(false);
      return;
    }

    // Deduplicate concurrent fetches
    if (!fetchPromise) {
      fetchPromise = (async () => {
        try {
          const response = await fetch("/v1/app/health-check");
          if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
          }
          cachedData = await response.json();
          cachedError = null;
        } catch (err) {
          cachedError =
            err instanceof Error ? err.message : "Failed to connect to server";
          cachedData = null;
        }
      })();
    }

    fetchPromise.then(() => {
      if (mounted.current) {
        setData(cachedData);
        setError(cachedError);
        setIsLoading(false);
      }
    });

    return () => {
      mounted.current = false;
    };
  }, []);

  return { data, error, isLoading };
}

/** Force re-fetch on next call (used after wizard completes). */
export function invalidateHealthCheck() {
  cachedData = null;
  cachedError = null;
  fetchPromise = null;
}
