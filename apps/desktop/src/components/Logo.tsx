import { cn } from "@/lib/utils";

interface LogoMarkProps {
  className?: string;
  /** Pixel size of the square mark. */
  size?: number;
}

/**
 * Shehata Git brand mark.
 * An "S" trunk carries identity nodes; two source lines converge from the
 * left into the single correct path — multiple identities, one repository.
 */
export function LogoMark({ className, size = 32 }: LogoMarkProps) {
  return (
    <svg
      viewBox="0 0 64 64"
      width={size}
      height={size}
      fill="none"
      role="img"
      aria-label="Shehata Git logo"
      className={cn("shrink-0", className)}
    >
      <g stroke="currentColor" strokeLinecap="round">
        <path d="M11 12 L23 21.5" strokeWidth="3" />
        <path d="M11 26 L20 27.5" strokeWidth="3" />
        <path
          d="M44 18.5 C36.5 11.5 21.5 13 20.5 23 C19.5 33 45 31.5 45 42 C45 52.5 28 55.5 20 46"
          strokeWidth="6"
        />
      </g>
      <circle cx="11" cy="12" r="3.6" fill="currentColor" />
      <circle cx="11" cy="26" r="3.6" fill="currentColor" />
      <circle
        cx="20.5"
        cy="23"
        r="4.2"
        className="fill-background"
        stroke="currentColor"
        strokeWidth="2.6"
      />
      <circle
        cx="45"
        cy="42"
        r="4.2"
        className="fill-background"
        stroke="currentColor"
        strokeWidth="2.6"
      />
    </svg>
  );
}

interface LogoLockupProps {
  className?: string;
  showTagline?: boolean;
}

export function LogoLockup({ className, showTagline = false }: LogoLockupProps) {
  return (
    <div className={cn("flex items-center gap-3", className)}>
      <LogoMark size={36} className="text-primary" />
      <div className="flex flex-col">
        <span className="text-lg font-semibold leading-tight tracking-tight">Shehata Git</span>
        {showTagline && (
          <span className="text-xs text-muted-foreground">
            One repo. One identity. Zero switching.
          </span>
        )}
      </div>
    </div>
  );
}
