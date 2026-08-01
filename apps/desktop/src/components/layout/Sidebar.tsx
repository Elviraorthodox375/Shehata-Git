import { Activity, Bot, FolderGit2, Home, Settings, Stethoscope, Users } from "lucide-react";
import { LogoLockup, LogoMark } from "@/components/Logo";
import { cn } from "@/lib/utils";

export type PageId =
  | "home"
  | "accounts"
  | "repositories"
  | "repository-detail"
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
    <aside className="liquid-sidebar relative flex h-full w-[4.75rem] shrink-0 flex-col border-r border-white/10 lg:w-[15rem]">
      <div className="border-b border-white/10 px-3 py-5 lg:px-5">
        <div className="hidden lg:block">
          <LogoLockup />
        </div>
        <div className="flex justify-center lg:hidden">
          <LogoMark size={30} />
        </div>
        <div className="mt-4 hidden items-center justify-between border-t border-white/10 pt-3 lg:flex">
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
            const isActive =
              currentPage === item.id ||
              (item.id === "repositories" && currentPage === "repository-detail");
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
                <span className="hidden flex-1 font-medium lg:block">{item.label}</span>
                <span className="hidden font-mono text-[0.625rem] text-muted-foreground/55 lg:block">
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
          <span className="hidden flex-1 text-left font-medium lg:block">System check</span>
          <span className="hidden h-1.5 w-1.5 rounded-full bg-warning lg:block" aria-hidden />
        </button>
        <div className="hidden items-center justify-between px-3 pb-1 pt-3 font-mono text-[0.625rem] uppercase tracking-wider text-muted-foreground/55 lg:flex">
          <span>v0.1.2</span>
          <span>Local first</span>
        </div>
      </div>
    </aside>
  );
}
