import { cn } from "@/lib/utils";

interface LogoMarkProps {
  className?: string;
  /** Pixel size of the square mark. */
  size?: number;
}

export function LogoMark({ className, size = 32 }: LogoMarkProps) {
  return (
    <img
      src="/logo-mark.svg"
      width={size}
      height={size}
      alt="Shehata Git"
      className={cn("shrink-0 object-contain", className)}
    />
  );
}

interface LogoLockupProps {
  className?: string;
  showTagline?: boolean;
}

export function LogoLockup({ className, showTagline = false }: LogoLockupProps) {
  return (
    <div className={cn("flex items-center gap-3", className)}>
      <span className="flex h-9 w-9 items-center justify-center border border-primary/25 bg-primary/[0.06]">
        <LogoMark size={27} />
      </span>
      <div className="flex flex-col">
        <span className="font-display text-[1.05rem] font-semibold leading-tight tracking-[-0.025em]">
          Shehata Git
        </span>
        {showTagline && (
          <span className="text-xs text-muted-foreground">
            One repo. One identity. Zero switching.
          </span>
        )}
      </div>
    </div>
  );
}
