import { cn } from "@/lib/utils";

interface LocksoLogoProps {
  className?: string;
  size?: "sm" | "md" | "lg";
  showText?: boolean;
}

const iconSizes = {
  sm: "h-6 w-6",
  md: "h-8 w-8",
  lg: "h-10 w-10",
};

const textSizes = {
  sm: "text-xl",
  md: "text-2xl",
  lg: "text-4xl",
};

export function LocksoLogo({
  className,
  size = "md",
  showText = true,
}: LocksoLogoProps) {
  return (
    <div className={cn("flex items-center gap-2.5", className)}>
      <svg
        className={cn("text-primary", iconSizes[size])}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
        <path d="M7 11V7a5 5 0 0 1 10 0v4" />
      </svg>
      {showText && (
        <span
          className={cn(
            "font-bold tracking-tight text-foreground",
            textSizes[size],
          )}
        >
          Lockso
        </span>
      )}
    </div>
  );
}
