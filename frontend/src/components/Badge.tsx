import { cn, type ClassValue } from "@/util";
import type { JSX } from "astro/jsx-runtime";

type BadgeProps = {
  className?: ClassValue;
  onClick?: () => void;
  children: JSX.Element | JSX.Element[];
  screenReaderLabel?: string;
};

const Badge = ({
  className,
  children,
  onClick,
  screenReaderLabel,
}: BadgeProps) => {
  return (
    <span
      id="badge-dismiss-dark"
      className={cn(
        "inline-flex align-middle items-center px-2 py-2 text-sm leading-none font-medium rounded bg-zinc-700 text-zinc-300",
        className
      )}
    >
      {children}
      <button
        type="button"
        onClick={onClick}
        className="inline-flex zitems-center ms-1 text-sm text-zinc-400 bg-transparent rounded-sm hover:bg-zinc-600 hover:text-zinc-300"
        aria-label="Remove"
      >
        <svg
          className="w-2 h-2"
          aria-hidden="true"
          fill="none"
          viewBox="0 0 14 14"
        >
          <path
            stroke="currentColor"
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"
          />
        </svg>
        <span className="sr-only">{screenReaderLabel ?? "Remove"}</span>
      </button>
    </span>
  );
};

export default Badge;
