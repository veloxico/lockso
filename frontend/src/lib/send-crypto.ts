/**
 * Client-side encryption for Secure Send.
 * Uses Web Crypto API — the encryption key never leaves the browser.
 */

/** Generate a random 32-byte key as base64url. */
export function generateSendKey(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(32));
  return base64UrlEncode(bytes);
}

/** Encrypt plaintext with AES-256-GCM. Returns base64 ciphertext (nonce + ct). */
export async function encryptSendPayload(
  plaintext: string,
  keyB64: string,
): Promise<string> {
  const keyBytes = base64UrlDecode(keyB64);
  const key = await crypto.subtle.importKey(
    "raw",
    keyBytes.buffer as ArrayBuffer,
    { name: "AES-GCM" },
    false,
    ["encrypt"],
  );
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const encoded = new TextEncoder().encode(plaintext);
  const ct = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv: nonce as Uint8Array<ArrayBuffer> },
    key,
    encoded,
  );

  // Prepend nonce to ciphertext
  const combined = new Uint8Array(nonce.length + ct.byteLength);
  combined.set(nonce, 0);
  combined.set(new Uint8Array(ct), nonce.length);

  return btoa(String.fromCharCode(...combined));
}

/** Decrypt ciphertext (nonce + ct) with AES-256-GCM. */
export async function decryptSendPayload(
  ciphertextB64: string,
  keyB64: string,
): Promise<string> {
  const combined = Uint8Array.from(atob(ciphertextB64), (c) =>
    c.charCodeAt(0),
  );
  const nonce = combined.slice(0, 12);
  const ct = combined.slice(12);

  const keyBytes = base64UrlDecode(keyB64);
  const key = await crypto.subtle.importKey(
    "raw",
    keyBytes.buffer as ArrayBuffer,
    { name: "AES-GCM" },
    false,
    ["decrypt"],
  );

  const plainBuf = await crypto.subtle.decrypt(
    { name: "AES-GCM", iv: nonce as Uint8Array<ArrayBuffer> },
    key,
    ct,
  );

  return new TextDecoder().decode(plainBuf);
}

// ─── Base64url helpers ───

function base64UrlEncode(bytes: Uint8Array): string {
  const str = btoa(String.fromCharCode(...bytes));
  return str.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function base64UrlDecode(str: string): Uint8Array {
  const padded = str.replace(/-/g, "+").replace(/_/g, "/");
  const decoded = atob(padded);
  return Uint8Array.from(decoded, (c) => c.charCodeAt(0));
}
