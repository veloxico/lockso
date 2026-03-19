import { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ShieldAlert, ShieldCheck, Loader2 } from "lucide-react";
import { checkPasswordBreach } from "@/lib/breach-check";

interface BreachIndicatorProps {
  password: string;
}

/**
 * Shows whether a password has been found in known data breaches.
 * Debounces the check to avoid excessive API calls.
 */
export function BreachIndicator({ password }: BreachIndicatorProps) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<
    "idle" | "checking" | "safe" | "breached" | "error"
  >("idle");
  const [breachCount, setBreachCount] = useState(0);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    if (!password || password.length < 4) {
      setStatus("idle");
      setBreachCount(0);
      return;
    }

    setStatus("checking");
    let cancelled = false;

    // Debounce 800ms
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(async () => {
      const count = await checkPasswordBreach(password);
      if (cancelled) return;
      if (count === -1) {
        setStatus("error");
      } else if (count > 0) {
        setStatus("breached");
        setBreachCount(count);
      } else {
        setStatus("safe");
      }
    }, 800);

    return () => {
      cancelled = true;
      clearTimeout(timerRef.current);
    };
  }, [password]);

  if (status === "idle") return null;

  if (status === "checking") {
    return (
      <span className="flex items-center gap-1 text-xs text-muted-foreground">
        <Loader2 className="h-3 w-3 animate-spin" />
        {t("breach.checking")}
      </span>
    );
  }

  if (status === "breached") {
    return (
      <span className="flex items-center gap-1 text-xs text-destructive">
        <ShieldAlert className="h-3.5 w-3.5" />
        {t("breach.found", { count: breachCount })}
      </span>
    );
  }

  if (status === "safe") {
    return (
      <span className="flex items-center gap-1 text-xs text-success">
        <ShieldCheck className="h-3.5 w-3.5" />
        {t("breach.safe")}
      </span>
    );
  }

  // error — silently hide
  return null;
}
