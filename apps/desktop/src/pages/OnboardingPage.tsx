import { ArrowLeft, ArrowRight, PartyPopper } from "lucide-react";
import { useState } from "react";
import { LogoLockup, LogoMark } from "@/components/Logo";
import type { PageId } from "@/components/layout/Sidebar";
import { Button } from "@/components/ui/button";
import { AccountsPage } from "./AccountsPage";
import { DoctorPage } from "./DoctorPage";

interface OnboardingPageProps {
  onFinish: (page: PageId) => void;
}

const STEPS = ["Welcome", "System check", "Accounts", "Done"] as const;

/**
 * First-run onboarding. Kept deliberately short: welcome → real system check
 * → real account list → done. Repository linking is introduced on the Home
 * page once the system is healthy.
 */
export function OnboardingPage({ onFinish }: OnboardingPageProps) {
  const [step, setStep] = useState(0);

  const isFirst = step === 0;
  const isLast = step === STEPS.length - 1;

  return (
    <div className="flex h-full flex-col">
      {/* Progress */}
      <div className="flex items-center justify-center gap-1.5 border-b border-border py-3">
        {STEPS.map((label, i) => (
          <div key={label} className="flex items-center gap-1.5">
            <span
              className={
                i === step
                  ? "h-1.5 w-8 rounded-full bg-primary transition-colors"
                  : i < step
                    ? "h-1.5 w-8 rounded-full bg-success transition-colors"
                    : "h-1.5 w-8 rounded-full bg-border transition-colors"
              }
              aria-hidden
            />
          </div>
        ))}
        <span className="ml-3 text-xs text-muted-foreground">{STEPS[step]}</span>
      </div>

      <div className="scrollbar-thin flex-1 overflow-y-auto px-6 py-6">
        {step === 0 && <WelcomeStep />}
        {step === 1 && <DoctorPage />}
        {step === 2 && <AccountsPage />}
        {step === 3 && <DoneStep onFinish={onFinish} />}
      </div>

      {!isLast && (
        <footer className="flex items-center justify-between border-t border-border px-6 py-3">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setStep((s) => s - 1)}
            disabled={isFirst}
          >
            <ArrowLeft aria-hidden />
            Back
          </Button>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={() => onFinish("home")}>
              Skip setup
            </Button>
            <Button size="sm" onClick={() => setStep((s) => s + 1)}>
              {step === 0 ? "Start setup" : "Continue"}
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
    <div className="mx-auto flex max-w-md flex-col items-center gap-5 py-10 text-center">
      <LogoMark size={72} className="text-primary" />
      <LogoLockup showTagline />
      <p className="text-sm leading-relaxed text-muted-foreground">
        If you have more than one GitHub account on this computer, pushes can silently go out under
        the wrong identity. Shehata Git fixes that: you assign each repository its account once, and
        every tool respects it.
      </p>
    </div>
  );
}

function DoneStep({ onFinish }: { onFinish: (page: PageId) => void }) {
  return (
    <div className="mx-auto flex max-w-md flex-col items-center gap-5 py-10 text-center">
      <PartyPopper className="h-10 w-10 text-primary" aria-hidden />
      <h2 className="text-xl font-semibold">You are set up</h2>
      <p className="text-sm leading-relaxed text-muted-foreground">
        Next, link your first repository and assign it an account. After that, a plain{" "}
        <code className="font-mono text-foreground">git push</code> from anywhere — terminal or AI
        tool — uses the right identity automatically.
      </p>
      <div className="flex gap-2">
        <Button onClick={() => onFinish("repositories")}>Add a repository</Button>
        <Button variant="outline" onClick={() => onFinish("home")}>
          View dashboard
        </Button>
      </div>
    </div>
  );
}
