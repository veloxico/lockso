/**
 * HaveIBeenPwned k-anonymity breach check.
 *
 * Sends only the first 5 characters of the SHA-1 hash to the HIBP API,
 * so the full password is never transmitted over the network.
 *
 * @see https://haveibeenpwned.com/API/v3#SearchingPwnedPasswordsByRange
 */

/**
 * Check if a password has been found in known data breaches.
 * Returns the number of times the password appeared in breaches, or 0 if clean.
 * Returns -1 if the check failed (network error, etc.).
 */
export async function checkPasswordBreach(
  password: string,
): Promise<number> {
  if (!password) return 0;

  try {
    // Compute SHA-1 hash of the password
    const encoder = new TextEncoder();
    const data = encoder.encode(password);
    const hashBuffer = await crypto.subtle.digest("SHA-1", data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("")
      .toUpperCase();

    // k-anonymity: send only first 5 chars
    const prefix = hashHex.slice(0, 5);
    const suffix = hashHex.slice(5);

    const response = await fetch(
      `https://api.pwnedpasswords.com/range/${prefix}`,
      {
        headers: {
          "Add-Padding": "true", // Prevent response length analysis
        },
      },
    );

    if (!response.ok) return -1;

    const text = await response.text();
    const lines = text.split("\n");

    for (const line of lines) {
      const [hashSuffix, count] = line.split(":");
      if (hashSuffix?.trim() === suffix) {
        return parseInt(count?.trim() ?? "0", 10);
      }
    }

    return 0;
  } catch {
    return -1;
  }
}
