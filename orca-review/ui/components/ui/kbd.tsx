import { cn } from "@/lib/utils"

const isMac = typeof navigator !== "undefined" && /Mac|iPod|iPhone|iPad/.test(navigator.platform);

const MOD = isMac ? "⌘" : "Ctrl";
const SHIFT = isMac ? "⇧" : "Shift";
const ENTER = isMac ? "↵" : "Enter";

export function formatKeybind(keys: string): string {
  const parts = keys.split("+").map((p) => {
    const k = p.trim().toLowerCase();
    if (k === "mod") return MOD;
    if (k === "shift") return SHIFT;
    if (k === "enter") return ENTER;
    return p.trim();
  });
  return isMac ? parts.join("") : parts.join("+");
}

function Kbd({
  className,
  children,
  ...props
}: React.ComponentProps<"kbd">) {
  const formatted = typeof children === "string" ? formatKeybind(children) : children;
  return (
    <kbd
      className={cn(
        "inline-flex h-5 items-center justify-center rounded border border-border bg-muted px-1.5 font-sans text-[11px] text-muted-foreground",
        className
      )}
      {...props}
    >
      {formatted}
    </kbd>
  )
}

export { Kbd }
