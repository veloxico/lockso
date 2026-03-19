import { useState, useRef } from "react";
import { useTranslation } from "react-i18next";
import { Upload, FileText, CheckCircle2, AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { importExportApi } from "@/api/import-export";
import type { ImportFormat, ImportResult } from "@/types/import-export";

interface Props {
  open: boolean;
  onClose: () => void;
  vaultId: string;
  vaultName: string;
  onSuccess: () => void;
}

const FORMATS: { value: ImportFormat; label: string; ext: string }[] = [
  { value: "csv", label: "CSV", ext: ".csv" },
  { value: "json", label: "Lockso JSON", ext: ".json" },
  { value: "bitwarden", label: "Bitwarden JSON", ext: ".json" },
  { value: "passwork", label: "Passwork JSON", ext: ".json" },
  { value: "keepass", label: "KeePass XML", ext: ".xml" },
];

export function ImportDialog({ open, onClose, vaultId, vaultName, onSuccess }: Props) {
  const { t } = useTranslation();
  const fileRef = useRef<HTMLInputElement>(null);

  const [format, setFormat] = useState<ImportFormat>("csv");
  const [fileName, setFileName] = useState("");
  const [fileData, setFileData] = useState("");
  const [createFolders, setCreateFolders] = useState(true);
  const [importing, setImporting] = useState(false);
  const [result, setResult] = useState<ImportResult | null>(null);
  const [error, setError] = useState("");

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setFileName(file.name);
    setResult(null);
    setError("");

    const reader = new FileReader();
    reader.onload = () => {
      setFileData(reader.result as string);
    };
    reader.onerror = () => {
      setError(t("importExport.errorReadFile"));
    };
    reader.readAsText(file);
  };

  const handleImport = async () => {
    if (!fileData) return;
    setImporting(true);
    setError("");
    setResult(null);

    try {
      const res = await importExportApi.import(vaultId, format, fileData, createFolders);
      setResult(res);
      if (res.importedCount > 0) {
        onSuccess();
      }
    } catch {
      setError(t("importExport.errorImportFailed"));
    } finally {
      setImporting(false);
    }
  };

  const handleClose = () => {
    setFileName("");
    setFileData("");
    setResult(null);
    setError("");
    onClose();
  };

  const selectedFormat = FORMATS.find((f) => f.value === format);

  return (
    <Dialog open={open} onClose={handleClose}>
      <DialogHeader>
        <DialogTitle>{t("importExport.importTitle")}</DialogTitle>
        <DialogDescription>
          {t("importExport.importDescription", { name: vaultName })}
        </DialogDescription>
      </DialogHeader>

      <DialogContent>
        <div className="space-y-4">
          {/* Format selection */}
          <div className="space-y-2">
            <Label>{t("importExport.format")}</Label>
            <div className="grid grid-cols-2 gap-2">
              {FORMATS.map((f) => (
                <button
                  key={f.value}
                  onClick={() => {
                    setFormat(f.value);
                    setResult(null);
                  }}
                  className={`rounded-md border p-2.5 text-sm text-left transition-colors ${
                    format === f.value
                      ? "border-primary bg-primary/5 text-primary"
                      : "border-border hover:border-muted-foreground/50"
                  }`}
                >
                  <span className="font-medium">{f.label}</span>
                  <span className="ml-1 text-xs text-muted-foreground">({f.ext})</span>
                </button>
              ))}
            </div>
          </div>

          {/* File upload */}
          <div className="space-y-2">
            <Label>{t("importExport.file")}</Label>
            <div
              onClick={() => fileRef.current?.click()}
              className="flex cursor-pointer items-center justify-center gap-2 rounded-md border-2 border-dashed border-border p-6 hover:border-muted-foreground/50 transition-colors"
            >
              <input
                ref={fileRef}
                type="file"
                accept={selectedFormat?.ext || "*"}
                onChange={handleFileChange}
                className="hidden"
              />
              {fileName ? (
                <div className="flex items-center gap-2 text-sm">
                  <FileText className="h-4 w-4 text-muted-foreground" />
                  <span className="font-medium">{fileName}</span>
                </div>
              ) : (
                <div className="text-center">
                  <Upload className="mx-auto h-6 w-6 text-muted-foreground" />
                  <p className="mt-1 text-sm text-muted-foreground">
                    {t("importExport.dropOrClick")}
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Create folders option */}
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={createFolders}
              onChange={(e) => setCreateFolders(e.target.checked)}
              className="h-4 w-4 rounded border-input accent-primary"
            />
            <span className="text-sm">{t("importExport.createFolders")}</span>
          </label>

          {/* Result */}
          {result && (
            <div className="rounded-md border border-border p-4 space-y-2">
              <div className="flex items-center gap-2">
                <CheckCircle2 className="h-4 w-4 text-green-500" />
                <span className="text-sm font-medium">
                  {t("importExport.imported", { count: result.importedCount })}
                </span>
              </div>
              {result.skippedCount > 0 && (
                <div className="flex items-center gap-2">
                  <AlertCircle className="h-4 w-4 text-yellow-500" />
                  <span className="text-sm text-muted-foreground">
                    {t("importExport.skipped", { count: result.skippedCount })}
                  </span>
                </div>
              )}
              {result.errors.length > 0 && (
                <details className="text-xs text-muted-foreground">
                  <summary className="cursor-pointer">
                    {t("importExport.showErrors", { count: result.errors.length })}
                  </summary>
                  <ul className="mt-1 space-y-0.5 pl-4">
                    {result.errors.slice(0, 20).map((err, i) => (
                      <li key={i}>{err}</li>
                    ))}
                  </ul>
                </details>
              )}
            </div>
          )}

          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>
      </DialogContent>

      <DialogFooter>
        <Button variant="outline" onClick={handleClose}>
          {result ? t("importExport.done") : t("vault.cancel")}
        </Button>
        {!result && (
          <Button onClick={handleImport} disabled={!fileData || importing}>
            {importing ? <Spinner size="sm" /> : <Upload className="h-4 w-4" />}
            {t("importExport.import")}
          </Button>
        )}
      </DialogFooter>
    </Dialog>
  );
}
