import { useEffect, useState } from "react";
import { AppShell } from "@/components/layout/AppShell";
import type { PageId } from "@/components/layout/Sidebar";
import { AccountsPage } from "@/pages/AccountsPage";
import { ActivityPage } from "@/pages/ActivityPage";
import { AiIntegrationPage } from "@/pages/AiIntegrationPage";
import { DoctorPage } from "@/pages/DoctorPage";
import { HomePage } from "@/pages/HomePage";
import { OnboardingPage } from "@/pages/OnboardingPage";
import { RepositoriesPage } from "@/pages/RepositoriesPage";
import { RepositoryDetailPage } from "@/pages/RepositoryDetailPage";
import { SettingsPage } from "@/pages/SettingsPage";

const PAGE_META: Record<PageId, { title: string; description: string }> = {
  home: { title: "Overview", description: "Live identity-routing state" },
  accounts: { title: "Identities", description: "GitHub accounts available on this machine" },
  repositories: {
    title: "Repositories",
    description: "Local repositories and their identity routes",
  },
  "repository-detail": {
    title: "Repository workspace",
    description: "Review, commit, and sync one repository safely",
  },
  "ai-integration": {
    title: "Agent bridge",
    description: "Connect coding agents to guarded Git operations",
  },
  activity: { title: "Audit log", description: "Local, redacted operation history" },
  settings: { title: "Preferences", description: "Appearance and safety controls" },
  doctor: { title: "System check", description: "Everything Shehata Git needs, verified live" },
};

const ONBOARDED_KEY = "shehata.onboarded.v1";
const THEME_KEY = "shehata.theme.v1";
const DENSITY_KEY = "shehata.density.v1";
const TRANSPARENCY_KEY = "shehata.transparency.v1";

export default function App() {
  const [page, setPage] = useState<PageId>("home");
  const [repositoryId, setRepositoryId] = useState<string | null>(null);
  const [onboarded, setOnboarded] = useState(() => localStorage.getItem(ONBOARDED_KEY) === "yes");
  const [theme, setTheme] = useState<"dark" | "light">(() =>
    localStorage.getItem(THEME_KEY) === "light" ? "light" : "dark",
  );
  const [density, setDensity] = useState<"comfortable" | "compact">(() =>
    localStorage.getItem(DENSITY_KEY) === "compact" ? "compact" : "comfortable",
  );
  const [transparency, setTransparency] = useState<"glass" | "reduced">(() =>
    localStorage.getItem(TRANSPARENCY_KEY) === "reduced" ? "reduced" : "glass",
  );

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
    localStorage.setItem(THEME_KEY, theme);
  }, [theme]);

  useEffect(() => {
    document.documentElement.dataset.density = density;
    localStorage.setItem(DENSITY_KEY, density);
  }, [density]);

  useEffect(() => {
    document.documentElement.dataset.transparency = transparency;
    localStorage.setItem(TRANSPARENCY_KEY, transparency);
  }, [transparency]);

  function finishOnboarding(target: PageId) {
    localStorage.setItem(ONBOARDED_KEY, "yes");
    setOnboarded(true);
    setPage(target);
  }

  if (!onboarded) {
    return (
      <div className="h-full bg-background">
        <OnboardingPage onFinish={finishOnboarding} />
      </div>
    );
  }

  const meta = PAGE_META[page];

  return (
    <AppShell
      currentPage={page}
      onNavigate={setPage}
      title={meta.title}
      description={meta.description}
    >
      {page === "home" && <HomePage onNavigate={setPage} />}
      {page === "accounts" && <AccountsPage />}
      {page === "repositories" && (
        <RepositoriesPage
          onOpenRepository={(id) => {
            setRepositoryId(id);
            setPage("repository-detail");
          }}
        />
      )}
      {page === "repository-detail" && repositoryId && (
        <RepositoryDetailPage repositoryId={repositoryId} onBack={() => setPage("repositories")} />
      )}
      {page === "ai-integration" && <AiIntegrationPage />}
      {page === "activity" && <ActivityPage />}
      {page === "settings" && (
        <SettingsPage
          theme={theme}
          onThemeChange={setTheme}
          density={density}
          onDensityChange={setDensity}
          transparency={transparency}
          onTransparencyChange={setTransparency}
        />
      )}
      {page === "doctor" && <DoctorPage />}
    </AppShell>
  );
}
