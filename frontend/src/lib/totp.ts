/**
 * Client-side TOTP (RFC 6238) code generation using Web Crypto API.
 *
 * Generates 6-digit codes with 30-second time steps, compatible with
 * Google Authenticator, Authy, 1Password, etc.
 */

const TOTP_DIGITS = 6;
const TOTP_PERIOD = 30;

/** Decode a Base32-encoded string (RFC 4648, no padding). */
function decodeBase32(input: string): ArrayBuffer {
  const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
  const cleaned = input.toUpperCase().replace(/[^A-Z2-7]/g, "");

  const out: number[] = [];
  let bits = 0;
  let value = 0;

  for (const ch of cleaned) {
    const idx = alphabet.indexOf(ch);
    if (idx === -1) continue;
    value = (value << 5) | idx;
    bits += 5;
    if (bits >= 8) {
      out.push((value >>> (bits - 8)) & 0xff);
      bits -= 8;
    }
  }

  const buf = new ArrayBuffer(out.length);
  const view = new Uint8Array(buf);
  for (let i = 0; i < out.length; i++) view[i] = out[i]!;
  return buf;
}

/** Generate an HMAC-SHA1 signature using Web Crypto. */
async function hmacSha1(key: ArrayBuffer, data: ArrayBuffer): Promise<ArrayBuffer> {
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    key,
    { name: "HMAC", hash: "SHA-1" },
    false,
    ["sign"],
  );
  return crypto.subtle.sign("HMAC", cryptoKey, data);
}

/** HOTP (RFC 4226) — HMAC-based One-Time Password. */
async function hotp(secret: ArrayBuffer, counter: bigint): Promise<string> {
  // Counter as big-endian 8 bytes
  const counterBuf = new ArrayBuffer(8);
  const counterView = new DataView(counterBuf);
  counterView.setBigUint64(0, counter, false);

  const hashBuf = await hmacSha1(secret, counterBuf);
  const hash = new Uint8Array(hashBuf);

  // Dynamic truncation
  const offset = hash[19]! & 0x0f;
  const code =
    ((hash[offset]! & 0x7f) << 24) |
    ((hash[offset + 1]! & 0xff) << 16) |
    ((hash[offset + 2]! & 0xff) << 8) |
    (hash[offset + 3]! & 0xff);

  const otp = code % 10 ** TOTP_DIGITS;
  return otp.toString().padStart(TOTP_DIGITS, "0");
}

/**
 * Generate the current TOTP code for the given base32-encoded secret.
 *
 * Returns the 6-digit code string.
 */
export async function generateTotpCode(secretBase32: string): Promise<string> {
  const secret = decodeBase32(secretBase32);
  const now = Math.floor(Date.now() / 1000);
  const counter = BigInt(Math.floor(now / TOTP_PERIOD));
  return hotp(secret, counter);
}

/**
 * Get the number of seconds remaining in the current TOTP period.
 */
export function getTotpSecondsRemaining(): number {
  const now = Math.floor(Date.now() / 1000);
  return TOTP_PERIOD - (now % TOTP_PERIOD);
}

/**
 * Get the TOTP period length in seconds.
 */
export const TOTP_PERIOD_SECONDS = TOTP_PERIOD;
