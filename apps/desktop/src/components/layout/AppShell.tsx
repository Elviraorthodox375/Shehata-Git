import type { ReactNode } from "react";
import { type PageId, Sidebar } from "./Sidebar";

interface AppShellProps {
  currentPage: PageId;
  onNavigate: (page: PageId) => void;
  title: string;
  description?: string;
  actions?: ReactNode;
  children: ReactNode;
}

export function AppShell({
  currentPage,
  onNavigate,
  title,
  description,
  actions,
  children,
}: AppShellProps) {
  return (
    <div className="flex h-full overflow-hidden">
      <Sidebar currentPage={currentPage} onNavigate={onNavigate} />
      <main className="flex flex-1 flex-col overflow-hidden">
        <header className="flex items-center justify-between border-b border-border px-6 py-4">
          <div>
            <h1 className="text-lg font-semibold tracking-tight">{title}</h1>
            {description && <p className="mt-0.5 text-sm text-muted-foreground">{description}</p>}
          </div>
          {actions && <div className="flex items-center gap-2">{actions}</div>}
        </header>
        <div className="scrollbar-thin flex-1 overflow-y-auto px-6 py-5">{children}</div>
      </main>
    </div>
  );
}
