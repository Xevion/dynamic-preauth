import { cn, type ClassValue } from "@/util";

type EmboldenedProps = {
  children: string | number | null;
  skeletonWidth?: string;
  className?: ClassValue;
  copyable?: boolean;
};

const Emboldened = ({
  children,
  skeletonWidth,
  copyable,
  className,
}: EmboldenedProps) => {
  function copyToClipboard() {
    // Copy to clipboard
    if (children != null) navigator.clipboard.writeText(children.toString());
  }
  return (
    <span
      onClick={copyable && children != null ? copyToClipboard : undefined}
      className={cn(
        className,
        "bg-zinc-900/40 rounded border border-zinc-700 py-0.5 px-1 font-mono text-teal-400",
        {
          "cursor-pointer": copyable && children,
        }
      )}
    >
      {children ?? (
        <span class="animate-pulse bg-teal-800 max-h-1 overflow-hidden select-none text-transparent">
          {skeletonWidth ?? "?"}
        </span>
      )}
    </span>
  );
};

export default Emboldened;
