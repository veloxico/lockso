import { Link } from "react-router";
import { ChevronRight, Folder } from "lucide-react";
import { cn } from "@/lib/utils";
import { getColor } from "@/lib/colors";

interface BreadcrumbItem {
  label: string;
  href?: string;
  icon?: "vault" | "folder";
  colorCode?: number;
}

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
}

export function Breadcrumbs({ items }: BreadcrumbsProps) {
  return (
    <nav className="flex items-center gap-1 text-sm" aria-label="Breadcrumb">
      {items.map((item, idx) => {
        const isLast = idx === items.length - 1;
        const color = item.colorCode !== undefined ? getColor(item.colorCode) : null;

        return (
          <span key={idx} className="flex items-center gap-1">
            {idx > 0 && (
              <ChevronRight className="h-3.5 w-3.5 text-muted-foreground/40 shrink-0" />
            )}

            {item.href && !isLast ? (
              <Link
                to={item.href}
                className="flex items-center gap-1.5 rounded px-1.5 py-0.5 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              >
                {item.icon === "vault" && color && (
                  <span
                    className={cn(
                      "h-2 w-2 rounded-full shrink-0",
                      color.bg.replace("/15", ""),
                    )}
                  />
                )}
                {item.icon === "folder" && (
                  <Folder className="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
                )}
                <span className="truncate max-w-[120px]">{item.label}</span>
              </Link>
            ) : (
              <span
                className={cn(
                  "flex items-center gap-1.5 px-1.5 py-0.5 truncate max-w-[200px]",
                  isLast
                    ? "font-medium text-foreground"
                    : "text-muted-foreground",
                )}
              >
                {item.icon === "vault" && color && (
                  <span
                    className={cn(
                      "h-2 w-2 rounded-full shrink-0",
                      color.bg.replace("/15", ""),
                    )}
                  />
                )}
                {item.icon === "folder" && (
                  <Folder className="h-3.5 w-3.5 shrink-0 text-muted-foreground/60" />
                )}
                {item.label}
              </span>
            )}
          </span>
        );
      })}
    </nav>
  );
}
