import { Search, X } from "lucide-react";
import { cn } from "@/lib/utils";

interface SearchFieldProps {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  label: string;
  className?: string;
  resultCount?: number;
}

export function SearchField({
  value,
  onChange,
  placeholder,
  label,
  className,
  resultCount,
}: SearchFieldProps) {
  return (
    <label
      className={cn(
        "group relative flex min-h-12 min-w-0 items-center gap-2 overflow-hidden rounded-[0.8rem] border border-white/10 bg-background/30 p-1.5 shadow-[inset_0_1px_0_hsl(var(--glass-highlight)/0.05)] transition-all focus-within:border-primary/45 focus-within:bg-background/45 focus-within:shadow-[inset_0_1px_0_hsl(var(--glass-highlight)/0.08),0_0_0_3px_hsl(var(--primary)/0.08)]",
        className,
      )}
    >
      <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[0.55rem] border border-white/[0.07] bg-white/[0.035] text-muted-foreground transition-colors group-focus-within:border-primary/20 group-focus-within:bg-primary/[0.08] group-focus-within:text-primary">
        <Search className="h-4 w-4" aria-hidden />
      </span>
      <span className="sr-only">{label}</span>
      <input
        type="search"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className="h-9 min-w-0 flex-1 bg-transparent px-1 text-sm outline-none placeholder:text-muted-foreground/50 focus-visible:ring-0 focus-visible:ring-offset-0 [&::-webkit-search-cancel-button]:hidden"
      />
      {value ? (
        <button
          type="button"
          onClick={() => onChange("")}
          className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[0.55rem] text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          aria-label={`Clear ${label.toLowerCase()}`}
        >
          <X className="h-3.5 w-3.5" aria-hidden />
        </button>
      ) : resultCount !== undefined ? (
        <span className="mr-1 shrink-0 rounded-full border border-white/[0.07] bg-white/[0.035] px-2.5 py-1 font-mono text-[0.625rem] uppercase tracking-[0.12em] text-muted-foreground/70">
          {resultCount} found
        </span>
      ) : null}
    </label>
  );
}
