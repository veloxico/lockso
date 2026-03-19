import { useState, useCallback, useRef, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCw, Copy, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  generatePassword,
  passwordStrength,
  type GeneratorOptions,
  DEFAULT_OPTIONS,
} from "@/lib/password-generator";
import { BreachIndicator } from "./breach-indicator";
import { cn } from "@/lib/utils";

interface PasswordGeneratorProps {
  onUse: (password: string) => void;
}

const STRENGTH_COLORS = [
  "bg-red-500",
  "bg-orange-500",
  "bg-amber-500",
  "bg-emerald-400",
  "bg-emerald-500",
];

const STRENGTH_LABELS = [
  "item.strengthVeryWeak",
  "item.strengthWeak",
  "item.strengthFair",
  "item.strengthGood",
  "item.strengthStrong",
];

export function PasswordGenerator({ onUse }: PasswordGeneratorProps) {
  const { t } = useTranslation();
  const [options, setOptions] = useState<GeneratorOptions>(DEFAULT_OPTIONS);
  const [password, setPassword] = useState(() => generatePassword(DEFAULT_OPTIONS));
  const [copied, setCopied] = useState(false);
  const [applied, setApplied] = useState(false);
  const appliedTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(appliedTimerRef.current), []);

  const regenerate = useCallback(() => {
    setPassword(generatePassword(options));
    setCopied(false);
  }, [options]);

  const copyTimerRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(copyTimerRef.current), []);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(password);
      setCopied(true);
      clearTimeout(copyTimerRef.current);
      copyTimerRef.current = setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard API not available
    }
  };

  const updateOption = (key: keyof GeneratorOptions, value: boolean | number) => {
    const next = { ...options, [key]: value };
    setOptions(next);
    setPassword(generatePassword(next));
    setCopied(false);
  };

  const strength = passwordStrength(password);

  return (
    <div className="space-y-4 rounded-lg border border-border p-4 bg-muted/30">
      <div className="flex items-center justify-between">
        <Label className="text-sm font-medium">{t("item.generator")}</Label>
      </div>

      {/* Generated password */}
      <div className="flex items-center gap-2">
        <Input
          value={password}
          readOnly
          className="font-mono text-sm cursor-pointer hover:bg-muted/50 transition-colors"
          onClick={() => {
            onUse(password);
            setApplied(true);
            clearTimeout(appliedTimerRef.current);
            appliedTimerRef.current = setTimeout(() => setApplied(false), 1500);
          }}
          title={t("item.clickToApply")}
        />
        <Button variant="ghost" size="icon" onClick={regenerate} title={t("item.regenerate")}>
          <RefreshCw className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" onClick={handleCopy} title={t("item.copy")}>
          {copied ? (
            <Check className="h-4 w-4 text-emerald-500" />
          ) : (
            <Copy className="h-4 w-4" />
          )}
        </Button>
      </div>

      {/* Applied feedback */}
      {applied && (
        <p className="text-xs text-emerald-500 font-medium flex items-center gap-1">
          <Check className="h-3 w-3" />
          {t("item.passwordApplied")}
        </p>
      )}

      {/* Strength bar */}
      <div className="space-y-1">
        <div className="flex gap-1">
          {Array.from({ length: 4 }).map((_, i) => (
            <div
              key={i}
              className={cn(
                "h-1.5 flex-1 rounded-full transition-colors",
                i <= strength - 1 ? STRENGTH_COLORS[strength] : "bg-muted",
              )}
            />
          ))}
        </div>
        <div className="flex items-center justify-between">
          <p className="text-xs text-muted-foreground">{t(STRENGTH_LABELS[strength] ?? "item.strengthVeryWeak")}</p>
          <BreachIndicator password={password} />
        </div>
      </div>

      {/* Options */}
      <div className="grid grid-cols-2 gap-3">
        <div className="col-span-2 space-y-1">
          <Label className="text-xs">{t("item.genLength")}: {options.length}</Label>
          <input
            type="range"
            min={8}
            max={64}
            value={options.length}
            onChange={(e) => updateOption("length", Number(e.target.value))}
            className="w-full accent-primary"
          />
        </div>

        <ToggleOption
          label={t("item.genUppercase")}
          checked={options.uppercase}
          onChange={(v) => updateOption("uppercase", v)}
        />
        <ToggleOption
          label={t("item.genLowercase")}
          checked={options.lowercase}
          onChange={(v) => updateOption("lowercase", v)}
        />
        <ToggleOption
          label={t("item.genDigits")}
          checked={options.digits}
          onChange={(v) => updateOption("digits", v)}
        />
        <ToggleOption
          label={t("item.genSymbols")}
          checked={options.symbols}
          onChange={(v) => updateOption("symbols", v)}
        />
      </div>

      {/* Use button */}
      <Button
        className="w-full"
        size="sm"
        onClick={() => {
          onUse(password);
          setApplied(true);
          clearTimeout(appliedTimerRef.current);
          appliedTimerRef.current = setTimeout(() => setApplied(false), 1500);
        }}
      >
        {applied ? (
          <>
            <Check className="h-4 w-4" />
            {t("item.passwordApplied")}
          </>
        ) : (
          t("item.usePassword")
        )}
      </Button>
    </div>
  );
}

function ToggleOption({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center gap-2 cursor-pointer text-sm">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="h-4 w-4 rounded border-input accent-primary"
      />
      {label}
    </label>
  );
}
