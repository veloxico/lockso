import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Download, FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Alert } from "@/components/ui/alert";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { importExportApi } from "@/api/import-export";
import type { ExportFormat } from "@/types/import-export";

interface Props {
  open: boolean;
  onClose: () => void;
  vaultId: string;
  vaultName: string;
}

const FORMATS: { value: ExportFormat; label: string; mime: string; ext: string }[] = [
  { value: "csv", label: "CSV", mime: "text/csv", ext: ".csv" },
  { value: "json", label: "JSON", mime: "application/json", ext: ".json" },
];

export function ExportDialog({ open, onClose, vaultId, vaultName }: Props) {
  const { t } = useTranslation();
  const [format, setFormat] = useState<ExportFormat>("csv");
  const [exporting, setExporting] = useState(false);
  const [error, setError] = useState("");

  const handleExport = async () => {
    setExporting(true);
    setError("");

    try {
      const result = await importExportApi.export(vaultId, format);

      // Create downloadable file
      const fmt = FORMATS.find((f) => f.value === format)!;
      const blob = new Blob([result.data], { type: fmt.mime });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${vaultName.replace(/[^a-zA-Z0-9]/g, "_")}${fmt.ext}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      onClose();
    } catch {
      setError(t("importExport.errorExportFailed"));
    } finally {
      setExporting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose}>
      <DialogHeader>
        <DialogTitle>{t("importExport.exportTitle")}</DialogTitle>
        <DialogDescription>
          {t("importExport.exportDescription", { name: vaultName })}
        </DialogDescription>
      </DialogHeader>

      <DialogContent>
        <div className="space-y-4">
          <Alert variant="destructive">
            <p className="text-sm">{t("importExport.exportWarning")}</p>
          </Alert>

          <div className="space-y-2">
            <Label>{t("importExport.format")}</Label>
            <div className="flex gap-3">
              {FORMATS.map((f) => (
                <button
                  key={f.value}
                  onClick={() => setFormat(f.value)}
                  className={`flex items-center gap-2 rounded-md border p-3 text-sm transition-colors ${
                    format === f.value
                      ? "border-primary bg-primary/5 text-primary"
                      : "border-border hover:border-muted-foreground/50"
                  }`}
                >
                  <FileText className="h-4 w-4" />
                  <span className="font-medium">{f.label}</span>
                </button>
              ))}
            </div>
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>
      </DialogContent>

      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          {t("vault.cancel")}
        </Button>
        <Button onClick={handleExport} disabled={exporting}>
          {exporting ? <Spinner size="sm" /> : <Download className="h-4 w-4" />}
          {t("importExport.export")}
        </Button>
      </DialogFooter>
    </Dialog>
  );
}
