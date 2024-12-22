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
      class={cn(
        "inline-flex align-middle items-center px-2 py-1 me-2 text-sm leading-none font-medium text-zinc-800 bg-zinc-100 rounded dark:bg-zinc-700 dark:text-zinc-300",
        className
      )}
    >
      {children}
      <button
        type="button"
        onClick={onClick}
        class="inline-flex items-center ms-1 text-sm text-zinc-400 bg-transparent rounded-sm hover:bg-zinc-200 hover:text-zinc-900 dark:hover:bg-zinc-600 dark:hover:text-zinc-300"
        data-dismiss-target="#badge-dismiss-dark"
        aria-label="Remove"
      >
        <svg class="w-2 h-2" aria-hidden="true" fill="none" viewBox="0 0 14 14">
          <path
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"
          />
        </svg>
        <span class="sr-only">{screenReaderLabel ?? "Remove"}</span>
      </button>
    </span>
  );
};

export default Badge;
