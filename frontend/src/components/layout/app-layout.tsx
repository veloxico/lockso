import { type ReactNode, createContext, useContext, useState, useEffect } from "react";
import { Sidebar } from "./sidebar";

interface SidebarContextValue {
  collapsed: boolean;
  setCollapsed: (v: boolean) => void;
  toggle: () => void;
}

const SidebarContext = createContext<SidebarContextValue>({
  collapsed: false,
  setCollapsed: () => {},
  toggle: () => {},
});

export function useSidebar() {
  return useContext(SidebarContext);
}

const STORAGE_KEY = "lockso:sidebar-collapsed";

interface AppLayoutProps {
  children: ReactNode;
  /** When true, children fill the entire main area (no padding/scroll wrapper). Used by vault-detail. */
  fullHeight?: boolean;
}

export function AppLayout({ children, fullHeight }: AppLayoutProps) {
  const [collapsed, setCollapsed] = useState(() => {
    try {
      return localStorage.getItem(STORAGE_KEY) === "true";
    } catch {
      return false;
    }
  });

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, String(collapsed));
    } catch {
      // ignore
    }
  }, [collapsed]);

  const toggle = () => setCollapsed((v) => !v);

  return (
    <SidebarContext.Provider value={{ collapsed, setCollapsed, toggle }}>
      <div className="flex h-screen overflow-hidden bg-background">
        <Sidebar />
        {fullHeight ? (
          <main className="flex-1 flex flex-col overflow-hidden">
            {children}
          </main>
        ) : (
          <main className="flex-1 overflow-y-auto">
            <div className="mx-auto max-w-7xl p-6">{children}</div>
          </main>
        )}
      </div>
    </SidebarContext.Provider>
  );
}
