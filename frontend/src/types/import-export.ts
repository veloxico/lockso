export type ImportFormat = "csv" | "json" | "passwork" | "keepass" | "bitwarden";
export type ExportFormat = "csv" | "json";

export interface ImportResult {
  importedCount: number;
  skippedCount: number;
  errors: string[];
}

export interface ExportResult {
  format: string;
  data: string;
  itemCount: number;
}
