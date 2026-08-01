import { useQuery } from "@tanstack/react-query";
import { ArrowRight, FolderGit2, ShieldCheck, Users } from "lucide-react";
import { LogoMark } from "@/components/Logo";
import type { PageId } from "@/components/layout/Sidebar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { listAccounts, listRepositories, runDoctor } from "@/lib/tauri";

interface HomePageProps {
  onNavigate: (page: PageId) => void;
}

export function HomePage({ onNavigate }: HomePageProps) {
  const doctor = useQuery({ queryKey: ["doctor"], queryFn: runDoctor });
  const accounts = useQuery({ queryKey: ["accounts"], queryFn: listAccounts });
  const repos = useQuery({ queryKey: ["repositories"], queryFn: listRepositories });

  const accountCount = accounts.data?.length ?? 0;
  const repoCount = repos.data?.length ?? 0;
  const systemHealthy = doctor.data?.healthy ?? false;

  return (
    <div className="mx-auto max-w-2xl space-y-5">
      {/* Hero */}
      <div className="flex items-start gap-4 rounded-lg border border-border bg-surface p-5">
        <LogoMark size={48} className="text-primary" />
        <div className="space-y-1">
          <h2 className="text-xl font-semibold tracking-tight">
            One repo. One identity. Zero switching.
          </h2>
          <p className="max-w-lg text-sm leading-relaxed text-muted-foreground">
            Shehata Git connects each of your repositories to the right GitHub account, so every
            push — from this app, your terminal, or your AI coding assistant — goes out with the
            correct identity.
          </p>
        </div>
      </div>

      {/* Status cards — all real data */}
      <div className="grid grid-cols-3 gap-3">
        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium text-muted-foreground">System</CardTitle>
              <ShieldCheck className="h-4 w-4 text-muted-foreground" aria-hidden />
            </div>
          </CardHeader>
          <CardContent>
            {doctor.isLoading ? (
              <p className="text-sm text-muted-foreground">Checking…</p>
            ) : (
              <Badge variant={systemHealthy ? "success" : "warning"}>
                {systemHealthy ? "All good" : "Needs attention"}
              </Badge>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                GitHub accounts
              </CardTitle>
              <Users className="h-4 w-4 text-muted-foreground" aria-hidden />
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-semibold tabular-nums">{accountCount}</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                Repositories
              </CardTitle>
              <FolderGit2 className="h-4 w-4 text-muted-foreground" aria-hidden />
            </div>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-semibold tabular-nums">{repoCount}</p>
          </CardContent>
        </Card>
      </div>

      {/* Getting started */}
      <Card>
        <CardHeader>
          <CardTitle>Getting started</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Step
            number={1}
            done={systemHealthy}
            title="Check your system"
            text="Make sure Git and GitHub CLI are installed and reachable."
            actionLabel="Open system check"
            onAction={() => onNavigate("doctor")}
          />
          <Step
            number={2}
            done={accountCount > 0}
            title="Add your GitHub accounts"
            text="Sign in through your browser. We never see or store your password."
            actionLabel="Go to accounts"
            onAction={() => onNavigate("accounts")}
          />
          <Step
            number={3}
            done={repoCount > 0}
            title="Link a repository"
            text="Pick a project folder and assign it the account it should always use."
            actionLabel="Go to repositories"
            onAction={() => onNavigate("repositories")}
          />
        </CardContent>
      </Card>
    </div>
  );
}

interface StepProps {
  number: number;
  done: boolean;
  title: string;
  text: string;
  actionLabel: string;
  onAction: () => void;
}

function Step({ number, done, title, text, actionLabel, onAction }: StepProps) {
  return (
    <div className="flex items-center gap-3 rounded-md border border-border/60 px-3 py-2.5">
      <span
        className={
          done
            ? "flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-success/15 text-xs font-semibold text-success"
            : "flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-secondary text-xs font-semibold text-muted-foreground"
        }
        aria-hidden
      >
        {done ? "✓" : number}
      </span>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">{title}</p>
        <p className="truncate text-xs text-muted-foreground">{text}</p>
      </div>
      {!done && (
        <Button variant="ghost" size="sm" onClick={onAction}>
          {actionLabel}
          <ArrowRight aria-hidden />
        </Button>
      )}
    </div>
  );
}
