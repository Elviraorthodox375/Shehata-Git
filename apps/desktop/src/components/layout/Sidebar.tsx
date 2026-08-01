import { Activity, Bot, FolderGit2, Home, Settings, Stethoscope, Users } from "lucide-react";
import { LogoLockup } from "@/components/Logo";
import { cn } from "@/lib/utils";

export type PageId =
  | "home"
  | "accounts"
  | "repositories"
  | "ai-integration"
  | "activity"
  | "settings"
  | "doctor";

interface NavItem {
  id: PageId;
  label: string;
  shortcut: string;
  icon: React.ComponentType<{ className?: string }>;
}

const NAV_ITEMS: NavItem[] = [
  { id: "home", label: "Overview", shortcut: "01", icon: Home },
  { id: "accounts", label: "Identities", shortcut: "02", icon: Users },
  { id: "repositories", label: "Repositories", shortcut: "03", icon: FolderGit2 },
  { id: "ai-integration", label: "Agent bridge", shortcut: "04", icon: Bot },
  { id: "activity", label: "Audit log", shortcut: "05", icon: Activity },
  { id: "settings", label: "Preferences", shortcut: "06", icon: Settings },
];

interface SidebarProps {
  currentPage: PageId;
  onNavigate: (page: PageId) => void;
}

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="relative flex h-full w-[15rem] shrink-0 flex-col border-r border-border/80 bg-surface/95">
      <div className="border-b border-border/70 px-5 py-5">
        <LogoLockup />
        <div className="mt-4 flex items-center justify-between border-t border-border/60 pt-3">
          <span className="eyebrow">Local engine</span>
          <span className="flex items-center gap-1.5 font-mono text-[0.65rem] uppercase tracking-wider text-success">
            <span className="h-1.5 w-1.5 rounded-full bg-success shadow-[0_0_10px_hsl(var(--success)/0.7)]" />
            Ready
          </span>
        </div>
      </div>

      <nav className="flex-1 px-3 py-5" aria-label="Main navigation">
        <p className="eyebrow px-3 pb-2">Workspace</p>
        <div className="space-y-1">
          {NAV_ITEMS.map((item) => {
            const isActive = currentPage === item.id;
            return (
              <button
                type="button"
                key={item.id}
                onClick={() => onNavigate(item.id)}
                aria-current={isActive ? "page" : undefined}
                className={cn(
                  "group relative flex min-h-11 w-full items-center gap-3 rounded-[0.55rem] border px-3 text-left text-sm transition-all",
                  isActive
                    ? "border-primary/20 bg-primary/[0.08] text-foreground"
                    : "border-transparent text-muted-foreground hover:border-border/70 hover:bg-secondary/60 hover:text-foreground",
                )}
              >
                <span
                  className={cn(
                    "absolute -left-3 h-5 w-0.5 rounded-r-full transition-colors",
                    isActive ? "bg-primary" : "bg-transparent",
                  )}
                />
                <item.icon
                  className={cn(
                    "h-4 w-4 transition-colors",
                    isActive ? "text-primary" : "text-muted-foreground group-hover:text-foreground",
                  )}
                  aria-hidden
                />
                <span className="flex-1 font-medium">{item.label}</span>
                <span className="font-mono text-[0.625rem] text-muted-foreground/55">
                  {item.shortcut}
                </span>
              </button>
            );
          })}
        </div>
      </nav>

      <div className="border-t border-border/70 p-3">
        <button
          type="button"
          onClick={() => onNavigate("doctor")}
          aria-current={currentPage === "doctor" ? "page" : undefined}
          className={cn(
            "flex min-h-11 w-full items-center gap-3 rounded-[0.55rem] border px-3 text-sm transition-colors",
            currentPage === "doctor"
              ? "border-primary/20 bg-primary/[0.08] text-foreground"
              : "border-border/60 bg-background/25 text-muted-foreground hover:bg-secondary/60 hover:text-foreground",
          )}
        >
          <Stethoscope className="h-4 w-4" aria-hidden />
          <span className="flex-1 text-left font-medium">System check</span>
          <span className="h-1.5 w-1.5 rounded-full bg-warning" aria-hidden />
        </button>
        <div className="flex items-center justify-between px-3 pb-1 pt-3 font-mono text-[0.625rem] uppercase tracking-wider text-muted-foreground/55">
          <span>v0.1.0</span>
          <span>Local first</span>
        </div>
      </div>
    </aside>
  );
}
