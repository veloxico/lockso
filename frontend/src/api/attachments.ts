import { useAuthStore } from "@/stores/auth";
import type { AttachmentView } from "@/types/vault";

const BASE_URL = "/v1";

/**
 * Attachment API — uses raw fetch for multipart upload and binary download.
 * Cannot use the standard ApiClient for these because of non-JSON bodies.
 */
export const attachmentApi = {
  /** List attachments for an item */
  list: async (itemId: string): Promise<AttachmentView[]> => {
    const token = useAuthStore.getState().accessToken;
    const res = await fetch(`${BASE_URL}/items/${itemId}/attachments`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw await parseError(res);
    return res.json();
  },

  /** Upload a file attachment */
  upload: async (itemId: string, file: File): Promise<AttachmentView> => {
    const token = useAuthStore.getState().accessToken;

    // Get CSRF token — fail if unavailable (never send without CSRF)
    const csrfToken = await fetchCsrfToken(token);

    const formData = new FormData();
    formData.append("file", file);

    const res = await fetch(`${BASE_URL}/items/${itemId}/attachments`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "X-CSRF-Token": csrfToken,
      },
      body: formData,
    });
    if (!res.ok) throw await parseError(res);
    return res.json();
  },

  /** Download an attachment as a Blob */
  download: async (attachmentId: string): Promise<{ blob: Blob; filename: string }> => {
    const token = useAuthStore.getState().accessToken;
    const res = await fetch(`${BASE_URL}/attachments/${attachmentId}`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    if (!res.ok) throw await parseError(res);

    const blob = await res.blob();

    // Extract filename from Content-Disposition header
    const disposition = res.headers.get("content-disposition") || "";
    const match = disposition.match(/filename="?([^"]+)"?/);
    const filename = match?.[1] ?? "download";

    return { blob, filename };
  },

  /** Delete an attachment */
  delete: async (attachmentId: string): Promise<void> => {
    const token = useAuthStore.getState().accessToken;

    // Get CSRF token — fail if unavailable
    const csrfToken = await fetchCsrfToken(token);

    const res = await fetch(`${BASE_URL}/attachments/${attachmentId}`, {
      method: "DELETE",
      headers: {
        Authorization: `Bearer ${token}`,
        "X-CSRF-Token": csrfToken,
      },
    });
    if (!res.ok) throw await parseError(res);
  },
};

/** Fetch a CSRF token. Throws if the request fails — never proceed without CSRF. */
async function fetchCsrfToken(accessToken: string | null): Promise<string> {
  const res = await fetch(`${BASE_URL}/csrf-tokens/generate`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${accessToken}`,
    },
  });
  if (!res.ok) {
    throw { status: res.status, message: "Failed to obtain CSRF token", code: "CSRF_FETCH_FAILED" };
  }
  const data = await res.json();
  if (!data.token) {
    throw { status: 500, message: "CSRF token missing from response", code: "CSRF_FETCH_FAILED" };
  }
  return data.token;
}

async function parseError(res: Response) {
  try {
    const data = await res.json();
    return { status: res.status, message: data.message || res.statusText, code: data.code };
  } catch {
    return { status: res.status, message: res.statusText };
  }
}
