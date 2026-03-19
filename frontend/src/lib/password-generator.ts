/**
 * Client-side password generator.
 * Uses crypto.getRandomValues() for cryptographically secure randomness.
 */

export interface GeneratorOptions {
  length: number;
  uppercase: boolean;
  lowercase: boolean;
  digits: boolean;
  symbols: boolean;
}

const UPPER = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER = "abcdefghijklmnopqrstuvwxyz";
const DIGITS = "0123456789";
const SYMBOLS = "!@#$%^&*()_+-=[]{}|;:,.<>?";

export const DEFAULT_OPTIONS: GeneratorOptions = {
  length: 20,
  uppercase: true,
  lowercase: true,
  digits: true,
  symbols: true,
};

/**
 * Generate a random password.
 * Guarantees at least one character from each enabled charset
 * when length >= number of enabled charsets.
 */
export function generatePassword(opts: GeneratorOptions): string {
  let charset = "";
  const required: string[] = [];

  if (opts.uppercase) {
    charset += UPPER;
    required.push(randomChar(UPPER));
  }
  if (opts.lowercase) {
    charset += LOWER;
    required.push(randomChar(LOWER));
  }
  if (opts.digits) {
    charset += DIGITS;
    required.push(randomChar(DIGITS));
  }
  if (opts.symbols) {
    charset += SYMBOLS;
    required.push(randomChar(SYMBOLS));
  }

  if (charset.length === 0) {
    charset = LOWER + UPPER + DIGITS;
    required.push(randomChar(LOWER));
  }

  const remaining = Math.max(0, opts.length - required.length);
  const chars = [...required];

  for (let i = 0; i < remaining; i++) {
    chars.push(randomChar(charset));
  }

  // Fisher-Yates shuffle
  for (let i = chars.length - 1; i > 0; i--) {
    const j = secureRandom(i + 1);
    const tmp = chars[i]!;
    chars[i] = chars[j]!;
    chars[j] = tmp;
  }

  return chars.join("");
}

function randomChar(charset: string): string {
  return charset.charAt(secureRandom(charset.length));
}

/**
 * Returns a uniform random integer in [0, max).
 * Uses rejection sampling to eliminate modulo bias.
 */
function secureRandom(max: number): number {
  const array = new Uint32Array(1);
  // Rejection sampling: discard values that would cause modulo bias
  const limit = Math.floor(0x100000000 / max) * max;
  let value: number;
  do {
    crypto.getRandomValues(array);
    value = array[0]!;
  } while (value >= limit);
  return value % max;
}

/**
 * Estimate password strength (0-4).
 */
export function passwordStrength(password: string): number {
  if (!password) return 0;
  let score = 0;
  if (password.length >= 8) score++;
  if (password.length >= 16) score++;
  if (/[A-Z]/.test(password) && /[a-z]/.test(password)) score++;
  if (/\d/.test(password)) score++;
  if (/[^A-Za-z0-9]/.test(password)) score++;
  return Math.min(4, score);
}
