import { AlertTriangle, Loader2, ShieldCheck, X } from "lucide-react";
import { type ReactNode, useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ConfirmDialogProps {
  eyebrow?: string;
  title: string;
  description: ReactNode;
  detail?: ReactNode;
  confirmLabel: string;
  cancelLabel?: string;
  pendingLabel?: string;
  pending?: boolean;
  tone?: "danger" | "primary";
  onCancel: () => void;
  onConfirm: () => void;
}

export function ConfirmDialog({
  eyebrow = "Confirm action",
  title,
  description,
  detail,
  confirmLabel,
  cancelLabel = "Cancel",
  pendingLabel = "Working…",
  pending = false,
  tone = "danger",
  onCancel,
  onConfirm,
}: ConfirmDialogProps) {
  const cancelButton = useRef<HTMLButtonElement>(null);
  const Icon = tone === "danger" ? AlertTriangle : ShieldCheck;

  useEffect(() => {
    cancelButton.current?.focus();
    function closeOnEscape(event: KeyboardEvent) {
      if (event.key === "Escape" && !pending) onCancel();
    }
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onCancel, pending]);

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-background/80 p-4 backdrop-blur-md">
      <section
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
        aria-describedby="confirm-dialog-description"
        className={cn(
          "liquid-panel w-full max-w-md overflow-hidden rounded-[1rem]",
          tone === "danger" ? "border border-warning/20" : "border border-primary/25",
        )}
      >
        <header className="flex items-start gap-4 border-b border-white/10 p-5 sm:p-6">
          <span
            className={cn(
              "flex h-11 w-11 shrink-0 items-center justify-center rounded-[0.75rem] border",
              tone === "danger"
                ? "border-warning/25 bg-warning/10 text-warning"
                : "border-primary/25 bg-primary/10 text-primary",
            )}
          >
            <Icon className="h-5 w-5" aria-hidden />
          </span>
          <div className="min-w-0 flex-1">
            <p className={cn("eyebrow", tone === "danger" ? "text-warning" : "text-primary")}>
              {eyebrow}
            </p>
            <h2 id="confirm-dialog-title" className="mt-1 font-display text-xl font-semibold">
              {title}
            </h2>
          </div>
          <button
            type="button"
            onClick={onCancel}
            disabled={pending}
            className="flex h-10 w-10 shrink-0 items-center justify-center rounded-[0.6rem] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
            aria-label="Close confirmation"
          >
            <X className="h-4 w-4" aria-hidden />
          </button>
        </header>
        <div className="space-y-4 p-5 sm:p-6">
          <div id="confirm-dialog-description" className="text-sm leading-6 text-muted-foreground">
            {description}
          </div>
          {detail && (
            <div className="rounded-[0.7rem] border border-white/10 bg-background/30 p-3 text-xs leading-5 text-muted-foreground">
              {detail}
            </div>
          )}
        </div>
        <footer className="flex flex-col-reverse gap-2 border-t border-white/10 bg-background/20 p-4 sm:flex-row sm:justify-end">
          <Button
            ref={cancelButton}
            type="button"
            variant="ghost"
            onClick={onCancel}
            disabled={pending}
          >
            {cancelLabel}
          </Button>
          <Button
            type="button"
            variant={tone === "danger" ? "destructive" : "default"}
            onClick={onConfirm}
            disabled={pending}
          >
            {pending && <Loader2 className="animate-spin" aria-hidden />}
            {pending ? pendingLabel : confirmLabel}
          </Button>
        </footer>
      </section>
    </div>
  );
}
