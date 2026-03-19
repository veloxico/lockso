import { api } from "./client";
import type { SessionView } from "@/types/session";

export const sessionApi = {
  list: () => api.get<SessionView[]>("/sessions"),

  deleteSingle: (id: string) =>
    api.delete<{ message: string }>(`/sessions/${id}`),

  deleteAllOthers: () =>
    api.delete<{ message: string; count: number }>("/sessions"),
};
