import { ArrowLeft, ArrowRight, Check, GitBranch, ShieldCheck, Terminal } from "lucide-react";
import { useState } from "react";
import { LogoLockup, LogoMark } from "@/components/Logo";
import type { PageId } from "@/components/layout/Sidebar";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { AccountsPage } from "./AccountsPage";
import { DoctorPage } from "./DoctorPage";

interface OnboardingPageProps {
  onFinish: (page: PageId) => void;
}

const STEPS = ["Welcome", "System", "Identities", "Ready"] as const;

export function OnboardingPage({ onFinish }: OnboardingPageProps) {
  const [step, setStep] = useState(0);
  const isFirst = step === 0;
  const isLast = step === STEPS.length - 1;

  return (
    <div className="app-canvas flex h-full flex-col">
      <header className="flex min-h-[4.75rem] items-center justify-between border-b border-border/80 bg-surface/80 px-5 sm:px-7">
        <LogoLockup />
        <ol className="hidden items-center gap-1 sm:flex" aria-label="Setup progress">
          {STEPS.map((label, index) => (
            <li
              key={label}
              className={cn(
                "flex items-center gap-2 border px-3 py-2 font-mono text-[0.65rem] uppercase tracking-wider",
                index === step
                  ? "border-primary/35 bg-primary/[0.08] text-primary"
                  : index < step
                    ? "border-success/20 text-success"
                    : "border-transparent text-muted-foreground/55",
              )}
              aria-current={index === step ? "step" : undefined}
            >
              <span>{index < step ? <Check className="h-3 w-3" /> : `0${index + 1}`}</span>
              {label}
            </li>
          ))}
        </ol>
        <span className="eyebrow sm:hidden">Step {step + 1} / 4</span>
      </header>

      <div className="scrollbar-thin flex-1 overflow-y-auto px-4 py-5 sm:px-7 sm:py-7">
        {step === 0 && <WelcomeStep />}
        {step === 1 && <DoctorPage />}
        {step === 2 && <AccountsPage />}
        {step === 3 && <DoneStep onFinish={onFinish} />}
      </div>

      {!isLast && (
        <footer className="flex min-h-[4.5rem] items-center justify-between border-t border-border/80 bg-surface/75 px-5 sm:px-7">
          <Button variant="ghost" onClick={() => setStep((value) => value - 1)} disabled={isFirst}>
            <ArrowLeft aria-hidden />
            Back
          </Button>
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => onFinish("home")}>
              Skip for now
            </Button>
            <Button onClick={() => setStep((value) => value + 1)}>
              {step === 0 ? "Begin setup" : "Continue"}
              <ArrowRight aria-hidden />
            </Button>
          </div>
        </footer>
      )}
    </div>
  );
}

function WelcomeStep() {
  return (
    <section className="instrument-panel mx-auto grid min-h-[31rem] w-full max-w-6xl overflow-hidden rounded-[0.8rem] lg:grid-cols-[1.1fr_0.9fr]">
      <div className="flex flex-col justify-between p-7 sm:p-10">
        <div>
          <p className="eyebrow">Local Git identity router / v0.1</p>
          <h1 className="mt-5 max-w-2xl font-display text-4xl font-semibold leading-[1.02] tracking-[-0.05em] sm:text-5xl">
            Every repository gets
            <span className="block text-primary">one explicit identity.</span>
          </h1>
          <p className="mt-6 max-w-xl text-base leading-7 text-muted-foreground">
            Stop switching Windows credentials. Shehata Git maps local repositories to authenticated
            GitHub accounts, then lets normal Git and coding agents follow the route.
          </p>
        </div>
        <div className="mt-10 grid gap-4 border-t border-border pt-6 sm:grid-cols-3">
          <WelcomeFact icon={ShieldCheck} code="LOCAL" text="No cloud backend" />
          <WelcomeFact icon={Terminal} code="NATIVE" text="Normal Git commands" />
          <WelcomeFact icon={GitBranch} code="SCOPED" text="Per-repo settings" />
        </div>
      </div>

      <div className="relative flex min-h-72 items-center justify-center overflow-hidden border-t border-border bg-background/35 p-8 lg:border-l lg:border-t-0">
        <div className="absolute inset-0 opacity-35 [background-image:linear-gradient(hsl(var(--border)/0.5)_1px,transparent_1px),linear-gradient(90deg,hsl(var(--border)/0.5)_1px,transparent_1px)] [background-size:2rem_2rem]" />
        <div className="relative">
          <div className="absolute inset-[-3rem] rounded-full border border-primary/10" />
          <div className="absolute inset-[-1.5rem] rounded-full border border-primary/20" />
          <div className="flex h-40 w-40 items-center justify-center border border-primary/35 bg-card shadow-[0_0_80px_hsl(var(--primary)/0.12)]">
            <LogoMark size={88} className="text-primary" />
          </div>
          <span className="absolute -left-20 top-1/2 h-px w-20 bg-gradient-to-r from-transparent to-primary/60" />
          <span className="absolute -right-20 top-1/2 h-px w-20 bg-gradient-to-l from-transparent to-primary/60" />
        </div>
      </div>
    </section>
  );
}

function WelcomeFact({
  icon: Icon,
  code,
  text,
}: {
  icon: typeof ShieldCheck;
  code: string;
  text: string;
}) {
  return (
    <div className="flex items-start gap-3">
      <Icon className="mt-0.5 h-4 w-4 text-primary" aria-hidden />
      <div>
        <p className="data-label text-foreground">{code}</p>
        <p className="mt-1 text-xs text-muted-foreground">{text}</p>
      </div>
    </div>
  );
}

function DoneStep({ onFinish }: { onFinish: (page: PageId) => void }) {
  return (
    <section className="instrument-panel mx-auto flex min-h-[28rem] max-w-3xl flex-col items-center justify-center rounded-[0.8rem] p-8 text-center">
      <div className="flex h-16 w-16 items-center justify-center border border-success/30 bg-success/10 text-success">
        <Check className="h-7 w-7" aria-hidden />
      </div>
      <p className="eyebrow mt-6">Initial checks complete</p>
      <h2 className="mt-2 font-display text-3xl font-semibold tracking-tight">Workspace ready</h2>
      <p className="mt-4 max-w-lg text-sm leading-7 text-muted-foreground">
        Add a repository, assign its GitHub identity, and optionally define its local commit author.
        Credential routing comes online in the next milestone.
      </p>
      <div className="mt-7 flex flex-wrap justify-center gap-2">
        <Button onClick={() => onFinish("repositories")}>Configure first repository</Button>
        <Button variant="outline" onClick={() => onFinish("home")}>
          Open overview
        </Button>
      </div>
    </section>
  );
}
