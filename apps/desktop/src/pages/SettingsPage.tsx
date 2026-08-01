import { useMutation } from "@tanstack/react-query";
import {
  Check,
  ClipboardCheck,
  Contrast,
  Copy,
  Gauge,
  GlassWater,
  Loader2,
  Moon,
  ShieldCheck,
  Sun,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getDiagnosticReport } from "@/lib/tauri";
import { cn } from "@/lib/utils";

interface SettingsPageProps {
  theme: "dark" | "light";
  onThemeChange: (theme: "dark" | "light") => void;
  density: "comfortable" | "compact";
  onDensityChange: (density: "comfortable" | "compact") => void;
  transparency: "glass" | "reduced";
  onTransparencyChange: (transparency: "glass" | "reduced") => void;
}

export function SettingsPage({
  theme,
  onThemeChange,
  density,
  onDensityChange,
  transparency,
  onTransparencyChange,
}: SettingsPageProps) {
  const diagnostic = useMutation({
    mutationFn: getDiagnosticReport,
    onSuccess: async (report) => {
      await navigator.clipboard.writeText(JSON.stringify(report, null, 2));
    },
  });

  return (
    <div className="mx-auto max-w-5xl space-y-4">
      <section className="liquid-hero rounded-[1.2rem] p-6 sm:p-8">
        <p className="eyebrow">Visual system / personal workspace</p>
        <h2 className="mt-3 font-display text-3xl font-semibold tracking-[-0.04em]">
          Make the workspace feel like yours.
        </h2>
        <p className="mt-3 max-w-2xl text-sm leading-6 text-muted-foreground">
          Liquid Glass is the default: translucent layers, precise highlights, and enough contrast
          for long Git sessions. Every option is stored only on this device.
        </p>
      </section>

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <GlassWater className="h-4 w-4 text-primary" aria-hidden />
              <CardTitle>Appearance</CardTitle>
            </div>
            <CardDescription>Choose the light environment and glass intensity.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-5">
            <SettingGroup label="COLOR THEME">
              <ChoiceButton
                selected={theme === "dark"}
                icon={Moon}
                label="Midnight"
                detail="Deep glass"
                onClick={() => onThemeChange("dark")}
              />
              <ChoiceButton
                selected={theme === "light"}
                icon={Sun}
                label="Daylight"
                detail="Bright glass"
                onClick={() => onThemeChange("light")}
              />
            </SettingGroup>
            <SettingGroup label="TRANSPARENCY">
              <ChoiceButton
                selected={transparency === "glass"}
                icon={GlassWater}
                label="Liquid Glass"
                detail="Blur + depth"
                onClick={() => onTransparencyChange("glass")}
              />
              <ChoiceButton
                selected={transparency === "reduced"}
                icon={Contrast}
                label="Reduced"
                detail="Higher opacity"
                onClick={() => onTransparencyChange("reduced")}
              />
            </SettingGroup>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Gauge className="h-4 w-4 text-primary" aria-hidden />
              <CardTitle>Workspace density</CardTitle>
            </div>
            <CardDescription>
              Control how much information fits without changing font size.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <SettingGroup label="LAYOUT">
              <ChoiceButton
                selected={density === "comfortable"}
                icon={GlassWater}
                label="Comfortable"
                detail="More breathing room"
                onClick={() => onDensityChange("comfortable")}
              />
              <ChoiceButton
                selected={density === "compact"}
                icon={Gauge}
                label="Compact"
                detail="More data visible"
                onClick={() => onDensityChange("compact")}
              />
            </SettingGroup>
            <div className="mt-5 rounded-[0.75rem] border border-success/20 bg-success/[0.06] p-4">
              <p className="flex items-center gap-2 text-sm font-semibold text-success">
                <ShieldCheck className="h-4 w-4" aria-hidden /> Accessibility guardrails
              </p>
              <p className="mt-2 text-xs leading-5 text-muted-foreground">
                Reduced motion follows Windows automatically. Keyboard focus remains visible, touch
                targets stay at least 44px, and Reduced transparency improves contrast.
              </p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <ClipboardCheck className="h-4 w-4 text-primary" aria-hidden />
              <CardTitle>Safe diagnostic report</CardTitle>
            </div>
            <CardDescription>
              Copy a support snapshot with versions and readiness only—never tokens, account names,
              repository paths, remotes, or file contents.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Button
              variant="outline"
              onClick={() => diagnostic.mutate()}
              disabled={diagnostic.isPending}
            >
              {diagnostic.isPending ? (
                <Loader2 className="animate-spin" aria-hidden />
              ) : diagnostic.isSuccess ? (
                <Check aria-hidden />
              ) : (
                <Copy aria-hidden />
              )}
              {diagnostic.isPending
                ? "Collecting…"
                : diagnostic.isSuccess
                  ? "Copied safely"
                  : "Copy diagnostic report"}
            </Button>
            {diagnostic.isError && (
              <p className="mt-3 text-sm text-destructive">
                {diagnostic.error instanceof Error
                  ? diagnostic.error.message
                  : String(diagnostic.error)}
              </p>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Safety baseline</CardTitle>
            <CardDescription>
              These protections cannot be disabled by appearance settings.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-muted-foreground">
            <p>Normal pushes only. Force push and remote deletion are never exposed.</p>
            <p>Tokens stay in GitHub CLI and are never written to the Shehata Git database.</p>
            <p className="font-mono text-xs">Shehata Git v0.1.4 · Local-first · MIT</p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function SettingGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div>
      <p className="data-label mb-2">{label}</p>
      <div className="grid gap-2 sm:grid-cols-2">{children}</div>
    </div>
  );
}

function ChoiceButton({
  selected,
  icon: Icon,
  label,
  detail,
  onClick,
}: {
  selected: boolean;
  icon: typeof Moon;
  label: string;
  detail: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={selected}
      className={cn(
        "flex min-h-16 items-center gap-3 rounded-[0.75rem] border p-3 text-left transition-all",
        selected
          ? "border-primary/35 bg-primary/[0.09] shadow-[inset_0_1px_0_hsl(var(--glass-highlight)/0.08)]"
          : "border-white/10 bg-background/15 hover:border-white/20 hover:bg-white/[0.035]",
      )}
    >
      <span
        className={cn(
          "flex h-9 w-9 items-center justify-center rounded-full border",
          selected ? "border-primary/30 bg-primary/10 text-primary" : "border-white/10",
        )}
      >
        <Icon className="h-4 w-4" aria-hidden />
      </span>
      <span className="min-w-0">
        <span className="block text-sm font-semibold">{label}</span>
        <span className="mt-0.5 block text-xs text-muted-foreground">{detail}</span>
      </span>
      {selected && <Check className="ml-auto h-4 w-4 text-primary" aria-hidden />}
    </button>
  );
}
