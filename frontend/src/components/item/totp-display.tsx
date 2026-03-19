import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Copy, Check, Eye, EyeOff, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  generateTotpCode,
  getTotpSecondsRemaining,
  TOTP_PERIOD_SECONDS,
} from "@/lib/totp";

interface TotpDisplayProps {
  /** The TOTP field label (field name). */
  name: string;
  /** The raw TOTP secret (base32 or otpauth:// URI). */
  secret: string;
}

/** Extract base32 secret from either a raw base32 string or an otpauth:// URI. */
function extractSecret(input: string): string {
  const trimmed = input.trim();

  // If it's an otpauth:// URI, extract the secret param
  if (trimmed.toLowerCase().startsWith("otpauth://")) {
    try {
      const url = new URL(trimmed);
      return url.searchParams.get("secret") || trimmed;
    } catch {
      // fallback to raw
    }
  }

  // Treat as raw base32
  return trimmed;
}

/**
 * Displays a live TOTP code with a countdown timer.
 *
 * Takes a base32 TOTP secret (or otpauth:// URI) and generates
 * the current 6-digit code, refreshing automatically every period.
 */
export function TotpDisplay({ name, secret }: TotpDisplayProps) {
  const { t } = useTranslation();
  const [code, setCode] = useState<string | null>(null);
  const [secondsLeft, setSecondsLeft] = useState(getTotpSecondsRemaining());
  const [error, setError] = useState(false);
  const [copied, setCopied] = useState(false);
  const [showSecret, setShowSecret] = useState(false);
  const copyTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const clipboardClearRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const base32Secret = extractSecret(secret);

  const generateCode = useCallback(async () => {
    try {
      const newCode = await generateTotpCode(base32Secret);
      setCode(newCode);
      setError(false);
    } catch {
      setCode(null);
      setError(true);
    }
  }, [base32Secret]);

  // Generate code on mount and refresh every second
  useEffect(() => {
    generateCode();

    const interval = setInterval(() => {
      const remaining = getTotpSecondsRemaining();
      setSecondsLeft(remaining);

      // Generate new code when period rolls over
      if (remaining === TOTP_PERIOD_SECONDS) {
        generateCode();
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [generateCode]);

  // Cleanup timers
  useEffect(() => {
    return () => {
      clearTimeout(copyTimerRef.current);
      clearTimeout(clipboardClearRef.current);
    };
  }, []);

  const handleCopy = async () => {
    if (!code) return;
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      clearTimeout(copyTimerRef.current);
      copyTimerRef.current = setTimeout(() => setCopied(false), 2000);

      // Auto-clear clipboard after 30s
      clearTimeout(clipboardClearRef.current);
      clipboardClearRef.current = setTimeout(() => {
        navigator.clipboard.writeText("").catch(() => {});
      }, 30_000);
    } catch {
      // Clipboard not available
    }
  };

  // Progress percentage (countdown)
  const progress = (secondsLeft / TOTP_PERIOD_SECONDS) * 100;
  const isUrgent = secondsLeft <= 5;

  return (
    <div className="rounded-md border border-border p-3">
      <div className="flex items-center justify-between mb-1.5">
        <p className="text-xs text-muted-foreground">{name}</p>
        <div className="flex items-center gap-1.5">
          {/* Countdown circle */}
          <div className="relative h-5 w-5" title={`${secondsLeft}s`}>
            <svg className="h-5 w-5 -rotate-90" viewBox="0 0 20 20">
              <circle
                cx="10"
                cy="10"
                r="8"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                className="text-muted/30"
              />
              <circle
                cx="10"
                cy="10"
                r="8"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeDasharray={`${(progress / 100) * 50.27} 50.27`}
                className={isUrgent ? "text-destructive" : "text-primary"}
              />
            </svg>
            <span
              className={`absolute inset-0 flex items-center justify-center text-[8px] font-bold ${
                isUrgent ? "text-destructive" : "text-muted-foreground"
              }`}
            >
              {secondsLeft}
            </span>
          </div>
        </div>
      </div>

      {/* OTP code */}
      {error ? (
        <div className="flex items-center gap-2">
          <span className="text-sm text-destructive">
            {t("item.totpError")}
          </span>
          <Button variant="ghost" size="icon" onClick={generateCode}>
            <RefreshCw className="h-3.5 w-3.5" />
          </Button>
        </div>
      ) : (
        <div className="flex items-center gap-2">
          <code
            className={`flex-1 text-2xl font-mono font-bold tracking-[0.25em] transition-colors ${
              isUrgent ? "text-destructive" : "text-foreground"
            }`}
          >
            {code ? `${code.slice(0, 3)} ${code.slice(3)}` : "--- ---"}
          </code>
          <Button
            variant="ghost"
            size="icon"
            onClick={handleCopy}
            title={t("item.copy")}
            className="shrink-0"
          >
            {copied ? (
              <Check className="h-4 w-4 text-emerald-500" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </Button>
        </div>
      )}

      {/* Toggle to show raw secret */}
      <button
        type="button"
        onClick={() => setShowSecret(!showSecret)}
        className="mt-2 flex items-center gap-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors"
      >
        {showSecret ? (
          <EyeOff className="h-3 w-3" />
        ) : (
          <Eye className="h-3 w-3" />
        )}
        {showSecret ? t("item.totpHideSecret") : t("item.totpShowSecret")}
      </button>
      {showSecret && (
        <code className="mt-1 block break-all text-[10px] text-muted-foreground font-mono bg-muted rounded px-2 py-1">
          {secret}
        </code>
      )}
    </div>
  );
}
