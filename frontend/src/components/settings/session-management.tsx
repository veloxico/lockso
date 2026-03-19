import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Monitor,
  Smartphone,
  Globe,
  Trash2,
  Shield,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { sessionApi } from "@/api/sessions";
import type { SessionView } from "@/types/session";

export function SessionManagement() {
  const { t } = useTranslation();
  const [sessions, setSessions] = useState<SessionView[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [revoking, setRevoking] = useState<string | null>(null);
  const [revokingAll, setRevokingAll] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError("");
    try {
      const result = await sessionApi.list();
      setSessions(result);
    } catch {
      setError(t("settings.errorLoadFailed"));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    load();
  }, [load]);

  const handleRevoke = async (id: string) => {
    setRevoking(id);
    try {
      await sessionApi.deleteSingle(id);
      setSessions((prev) => prev.filter((s) => s.id !== id));
    } catch {
      setError(t("settings.errorActionFailed"));
    } finally {
      setRevoking(null);
    }
  };

  const handleRevokeAll = async () => {
    setRevokingAll(true);
    try {
      await sessionApi.deleteAllOthers();
      setSessions((prev) => prev.filter((s) => s.isCurrent));
    } catch {
      setError(t("settings.errorActionFailed"));
    } finally {
      setRevokingAll(false);
    }
  };

  const otherCount = sessions.filter((s) => !s.isCurrent).length;

  return (
    <div className="max-w-4xl space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Shield className="h-5 w-5 text-muted-foreground" />
          <div>
            <h2 className="text-lg font-semibold">
              {t("sessions.title")}
            </h2>
            <p className="text-sm text-muted-foreground">
              {t("sessions.description")}
            </p>
          </div>
        </div>

        {otherCount > 0 && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleRevokeAll}
            disabled={revokingAll}
            className="text-destructive hover:text-destructive"
          >
            {revokingAll ? (
              <Spinner size="sm" />
            ) : (
              <>
                <X className="mr-1.5 h-3.5 w-3.5" />
                {t("sessions.revokeAll")}
              </>
            )}
          </Button>
        )}
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {loading ? (
        <div className="flex justify-center py-12">
          <Spinner size="md" />
        </div>
      ) : sessions.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">
          {t("sessions.noSessions")}
        </p>
      ) : (
        <div className="space-y-3">
          {sessions.map((session) => (
            <SessionCard
              key={session.id}
              session={session}
              onRevoke={handleRevoke}
              isRevoking={revoking === session.id}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function SessionCard({
  session,
  onRevoke,
  isRevoking,
}: {
  session: SessionView;
  onRevoke: (id: string) => void;
  isRevoking: boolean;
}) {
  const { t } = useTranslation();
  const deviceInfo = parseUserAgent(session.userAgent || "");

  return (
    <div
      className={`rounded-lg border p-4 ${
        session.isCurrent
          ? "border-primary/30 bg-primary/5"
          : "border-border bg-card"
      }`}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-start gap-3">
          <div className="mt-0.5">
            <DeviceIcon type={session.clientType} />
          </div>
          <div className="space-y-1">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">
                {deviceInfo.browser}
              </span>
              {session.isCurrent && (
                <span className="inline-flex items-center rounded-full bg-primary/15 px-2 py-0.5 text-[10px] font-medium text-primary">
                  {t("sessions.current")}
                </span>
              )}
            </div>
            <p className="text-xs text-muted-foreground">
              {deviceInfo.os}
            </p>
            <div className="flex items-center gap-3 text-xs text-muted-foreground">
              {session.clientIp && (
                <span className="flex items-center gap-1">
                  <Globe className="h-3 w-3" />
                  {session.clientIp}
                </span>
              )}
              <span>
                {t("sessions.lastActive")}{" "}
                {formatRelativeTime(session.lastActivityAt)}
              </span>
            </div>
            <p className="text-[11px] text-muted-foreground">
              {t("sessions.createdAt")}{" "}
              {new Date(session.createdAt).toLocaleString()}
            </p>
          </div>
        </div>

        {!session.isCurrent && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onRevoke(session.id)}
            disabled={isRevoking}
            className="text-destructive hover:text-destructive hover:bg-destructive/10"
          >
            {isRevoking ? (
              <Spinner size="sm" />
            ) : (
              <Trash2 className="h-4 w-4" />
            )}
          </Button>
        )}
      </div>
    </div>
  );
}

function DeviceIcon({ type }: { type: string }) {
  const lower = type.toLowerCase();
  if (lower.includes("mobile") || lower.includes("phone")) {
    return <Smartphone className="h-5 w-5 text-muted-foreground" />;
  }
  return <Monitor className="h-5 w-5 text-muted-foreground" />;
}

function parseUserAgent(ua: string): { browser: string; os: string } {
  if (!ua) return { browser: "Unknown browser", os: "Unknown OS" };

  let browser = "Unknown browser";
  if (ua.includes("Firefox/")) browser = "Firefox";
  else if (ua.includes("Edg/")) browser = "Microsoft Edge";
  else if (ua.includes("Chrome/") && !ua.includes("Edg/")) browser = "Chrome";
  else if (ua.includes("Safari/") && !ua.includes("Chrome/")) browser = "Safari";
  else if (ua.includes("Opera/") || ua.includes("OPR/")) browser = "Opera";

  let os = "Unknown OS";
  if (ua.includes("Windows")) os = "Windows";
  else if (ua.includes("Mac OS")) os = "macOS";
  else if (ua.includes("Linux")) os = "Linux";
  else if (ua.includes("Android")) os = "Android";
  else if (ua.includes("iPhone") || ua.includes("iPad")) os = "iOS";

  return { browser, os };
}

function formatRelativeTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime();
  const minutes = Math.floor(diff / 60_000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}
