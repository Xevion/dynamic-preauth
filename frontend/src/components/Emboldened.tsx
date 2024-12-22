import { cn, type ClassValue } from "@/util";

type EmboldenedProps = {
  children: string | number;
  className?: ClassValue;
  copyable?: boolean;
};

const Emboldened = ({ children, copyable, className }: EmboldenedProps) => {
  function copyToClipboard() {
    // Copy to clipboard
    navigator.clipboard.writeText(children.toString());
  }

  return (
    <span
      onClick={copyable ? copyToClipboard : undefined}
      className={cn(
        className,
        "bg-zinc-900/40 rounded border border-zinc-700 py-0.5 px-1 font-mono text-teal-400",
        {
          "cursor-pointer": copyable,
        }
      )}
    >
      {children}
    </span>
  );
};

export default Emboldened;
