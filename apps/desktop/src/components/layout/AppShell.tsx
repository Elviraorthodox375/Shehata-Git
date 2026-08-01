import { HardDrive, LockKeyhole } from "lucide-react";
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
    <div className="app-canvas flex h-full overflow-hidden">
      <Sidebar currentPage={currentPage} onNavigate={onNavigate} />
      <main className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <header className="flex min-h-[5.4rem] items-center justify-between gap-5 border-b border-white/10 bg-background/45 px-4 backdrop-blur-2xl sm:px-5 lg:px-8">
          <div className="min-w-0">
            <p className="eyebrow mb-1">Shehata / Local identity control</p>
            <div className="flex min-w-0 items-baseline gap-3">
              <h1 className="truncate font-display text-[1.4rem] font-semibold tracking-[-0.025em]">
                {title}
              </h1>
              {description && (
                <p className="hidden truncate text-sm text-muted-foreground xl:block">
                  {description}
                </p>
              )}
            </div>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            {actions}
            <div className="hidden items-center gap-3 border-l border-border pl-4 lg:flex">
              <span className="flex items-center gap-1.5 font-mono text-[0.6875rem] uppercase tracking-wider text-muted-foreground">
                <HardDrive className="h-3.5 w-3.5" aria-hidden />
                On device
              </span>
              <span className="flex items-center gap-1.5 font-mono text-[0.6875rem] uppercase tracking-wider text-success">
                <LockKeyhole className="h-3.5 w-3.5" aria-hidden />
                Secrets isolated
              </span>
            </div>
          </div>
        </header>
        <div className="scrollbar-thin flex-1 overflow-y-auto px-3 py-4 sm:px-5 sm:py-5 lg:px-8 lg:py-7">
          <div className="animate-fade-in">{children}</div>
        </div>
      </main>
    </div>
  );
}
