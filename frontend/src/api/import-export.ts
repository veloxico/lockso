import { api } from "./client";
import type {
  ImportFormat,
  ExportFormat,
  ImportResult,
  ExportResult,
} from "@/types/import-export";

export const importExportApi = {
  import: (
    vaultId: string,
    format: ImportFormat,
    data: string,
    createFolders = true,
  ) =>
    api.post<ImportResult>(`/vaults/${vaultId}/import`, {
      format,
      data,
      createFolders,
    }),

  export: (vaultId: string, format: ExportFormat) =>
    api.post<ExportResult>(`/vaults/${vaultId}/export`, { format }),
};
