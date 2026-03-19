import type { ApiErrorResponse } from "@/types/api";

/**
 * Safely extract API error from a thrown value.
 * Returns a normalized ApiErrorResponse even if the error is
 * a network failure or unexpected shape.
 */
export function toApiError(err: unknown): ApiErrorResponse {
  // Our ApiClient throws objects with { status, message, code }
  if (
    typeof err === "object" &&
    err !== null &&
    "message" in err &&
    typeof (err as Record<string, unknown>).message === "string"
  ) {
    const obj = err as Record<string, unknown>;
    return {
      code: typeof obj.code === "string" ? obj.code : "UNKNOWN_ERROR",
      message: obj.message as string,
    };
  }

  // Native Error (e.g., fetch network failure)
  if (err instanceof Error) {
    return {
      code: "NETWORK_ERROR",
      message: err.message,
    };
  }

  return {
    code: "UNKNOWN_ERROR",
    message: "An unexpected error occurred",
  };
}
