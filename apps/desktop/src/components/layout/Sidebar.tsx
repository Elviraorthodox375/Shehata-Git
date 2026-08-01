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
  icon: React.ComponentType<{ className?: string }>;
}

const NAV_ITEMS: NavItem[] = [
  { id: "home", label: "Home", icon: Home },
  { id: "accounts", label: "Accounts", icon: Users },
  { id: "repositories", label: "Repositories", icon: FolderGit2 },
  { id: "ai-integration", label: "AI Integration", icon: Bot },
  { id: "activity", label: "Activity", icon: Activity },
  { id: "settings", label: "Settings", icon: Settings },
];

interface SidebarProps {
  currentPage: PageId;
  onNavigate: (page: PageId) => void;
}

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="flex h-full w-56 flex-col border-r border-border bg-surface">
      <div className="px-4 pb-4 pt-5">
        <LogoLockup />
      </div>

      <nav className="flex-1 space-y-0.5 px-2" aria-label="Main navigation">
        {NAV_ITEMS.map((item) => {
          const isActive = currentPage === item.id;
          return (
            <button
              type="button"
              key={item.id}
              onClick={() => onNavigate(item.id)}
              aria-current={isActive ? "page" : undefined}
              className={cn(
                "flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-sm transition-colors",
                isActive
                  ? "bg-primary/10 font-medium text-primary"
                  : "text-muted-foreground hover:bg-secondary hover:text-foreground",
              )}
            >
              <item.icon className="h-4 w-4" aria-hidden />
              {item.label}
            </button>
          );
        })}
      </nav>

      <div className="border-t border-border p-2">
        <button
          type="button"
          onClick={() => onNavigate("doctor")}
          aria-current={currentPage === "doctor" ? "page" : undefined}
          className={cn(
            "flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-sm transition-colors",
            currentPage === "doctor"
              ? "bg-primary/10 font-medium text-primary"
              : "text-muted-foreground hover:bg-secondary hover:text-foreground",
          )}
        >
          <Stethoscope className="h-4 w-4" aria-hidden />
          System check
        </button>
        <p className="px-3 pb-1 pt-2 text-[10px] leading-relaxed text-muted-foreground/70">
          v0.1.0 — early development
        </p>
      </div>
    </aside>
  );
}
