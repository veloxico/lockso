import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  Paperclip,
  Upload,
  Download,
  Trash2,
  File,
  FileText,
  FileImage,
  FileArchive,
  FileAudio,
  FileVideo,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { attachmentApi } from "@/api/attachments";
import type { AttachmentView } from "@/types/vault";

interface AttachmentSectionProps {
  itemId: string;
}

const MAX_FILE_SIZE = 50 * 1024 * 1024; // 50 MB

export function AttachmentSection({ itemId }: AttachmentSectionProps) {
  const { t } = useTranslation();
  const [attachments, setAttachments] = useState<AttachmentView[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [error, setError] = useState("");
  const fileInputRef = useRef<HTMLInputElement>(null);

  const loadAttachments = useCallback(async () => {
    try {
      const data = await attachmentApi.list(itemId);
      setAttachments(data);
    } catch {
      // Ignore — section is optional
    } finally {
      setLoading(false);
    }
  }, [itemId]);

  useEffect(() => {
    setLoading(true);
    setError("");
    loadAttachments();
  }, [loadAttachments]);

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Reset input so same file can be selected again
    if (fileInputRef.current) fileInputRef.current.value = "";

    if (file.size > MAX_FILE_SIZE) {
      setError(t("attachments.fileTooLarge", { size: "50 MB" }));
      return;
    }

    setUploading(true);
    setError("");

    try {
      const view = await attachmentApi.upload(itemId, file);
      setAttachments((prev) => [...prev, view]);
    } catch (err: unknown) {
      const msg = (err as { message?: string })?.message || t("attachments.uploadFailed");
      setError(msg);
    } finally {
      setUploading(false);
    }
  };

  const handleDownload = async (attachment: AttachmentView) => {
    try {
      const { blob, filename } = await attachmentApi.download(attachment.id);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch {
      setError(t("attachments.downloadFailed"));
    }
  };

  const handleDelete = async (attachment: AttachmentView) => {
    if (!confirm(t("attachments.deleteConfirm", { name: attachment.name }))) {
      return;
    }

    try {
      await attachmentApi.delete(attachment.id);
      setAttachments((prev) => prev.filter((a) => a.id !== attachment.id));
    } catch {
      setError(t("attachments.deleteFailed"));
    }
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1.5">
          <Paperclip className="h-3.5 w-3.5" />
          {t("attachments.title")}
          {attachments.length > 0 && (
            <span className="ml-1 text-[10px] bg-muted rounded-full px-1.5 py-0.5">
              {attachments.length}
            </span>
          )}
        </p>

        <div>
          <input
            ref={fileInputRef}
            type="file"
            className="hidden"
            onChange={handleUpload}
          />
          <Button
            variant="ghost"
            size="sm"
            onClick={() => fileInputRef.current?.click()}
            disabled={uploading}
          >
            {uploading ? (
              <Spinner size="sm" />
            ) : (
              <Upload className="h-3.5 w-3.5" />
            )}
            {t("attachments.upload")}
          </Button>
        </div>
      </div>

      {error && <p className="text-xs text-destructive">{error}</p>}

      {loading ? (
        <div className="flex justify-center py-3">
          <Spinner size="sm" />
        </div>
      ) : attachments.length === 0 ? (
        <p className="text-xs text-muted-foreground py-2">
          {t("attachments.empty")}
        </p>
      ) : (
        <div className="space-y-1.5">
          {attachments.map((a) => (
            <AttachmentRow
              key={a.id}
              attachment={a}
              onDownload={() => handleDownload(a)}
              onDelete={() => handleDelete(a)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function AttachmentRow({
  attachment,
  onDownload,
  onDelete,
}: {
  attachment: AttachmentView;
  onDownload: () => void;
  onDelete: () => void;
}) {
  const Icon = getFileIcon(attachment.mimeType);

  return (
    <div className="flex items-center gap-2 rounded-md border border-border px-3 py-2 group hover:bg-muted/30 transition-colors">
      <Icon className="h-4 w-4 shrink-0 text-muted-foreground" />
      <div className="min-w-0 flex-1">
        <p className="text-sm truncate">{attachment.name}</p>
        <p className="text-[10px] text-muted-foreground">
          {formatFileSize(attachment.sizeBytes)}
        </p>
      </div>
      <div className="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
        <Button variant="ghost" size="icon" onClick={onDownload} className="h-7 w-7">
          <Download className="h-3.5 w-3.5" />
        </Button>
        <Button variant="ghost" size="icon" onClick={onDelete} className="h-7 w-7">
          <Trash2 className="h-3.5 w-3.5 text-destructive" />
        </Button>
      </div>
    </div>
  );
}

function getFileIcon(mimeType: string) {
  if (mimeType.startsWith("image/")) return FileImage;
  if (mimeType.startsWith("audio/")) return FileAudio;
  if (mimeType.startsWith("video/")) return FileVideo;
  if (mimeType.startsWith("text/")) return FileText;
  if (
    mimeType.includes("zip") ||
    mimeType.includes("tar") ||
    mimeType.includes("rar") ||
    mimeType.includes("7z") ||
    mimeType.includes("gzip")
  ) {
    return FileArchive;
  }
  if (mimeType.includes("pdf")) return FileText;
  return File;
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
