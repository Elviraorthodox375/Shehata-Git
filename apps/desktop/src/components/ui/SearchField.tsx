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
        "glass-input group flex min-h-11 min-w-0 items-center gap-3 px-3.5 transition-colors",
        className,
      )}
    >
      <Search
        className="h-4 w-4 shrink-0 text-muted-foreground transition-colors group-focus-within:text-primary"
        aria-hidden
      />
      <span className="sr-only">{label}</span>
      <input
        type="search"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/55 [&::-webkit-search-cancel-button]:hidden"
      />
      {value ? (
        <button
          type="button"
          onClick={() => onChange("")}
          className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
          aria-label={`Clear ${label.toLowerCase()}`}
        >
          <X className="h-3.5 w-3.5" aria-hidden />
        </button>
      ) : resultCount !== undefined ? (
        <span className="shrink-0 font-mono text-[0.65rem] uppercase tracking-wider text-muted-foreground/65">
          {resultCount} found
        </span>
      ) : null}
    </label>
  );
}
