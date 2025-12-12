import type { Executable } from "@/components/useSocket";
import MobileWarningModal from "@/components/MobileWarningModal";
import { cn, isMobile, withBackend } from "@/util";
import {
  Button,
  Menu,
  MenuButton,
  MenuItem,
  MenuItems,
  MenuSeparator,
} from "@headlessui/react";
import {
  BeakerIcon,
  ChevronDownIcon,
} from "@heroicons/react/16/solid";
import { FaWindows, FaApple, FaLinux } from "react-icons/fa";
import { useRef, useState } from "react";

const MOBILE_WARNING_KEY = "mobile-warning-acknowledged";

type DownloadButtonProps = {
  disabled?: boolean;
  executables: Executable[] | null;
  buildLog: string | null;
};

type SystemType = "windows" | "macos" | "linux";

function getSystemType(): SystemType | null {
  const userAgent = navigator.userAgent.toLowerCase();
  if (userAgent.includes("win")) {
    return "windows";
  } else if (userAgent.includes("mac")) {
    return "macos";
  } else if (userAgent.includes("linux")) {
    return "linux";
  } else {
    return null;
  }
}

function getPlatformIcon(id: string, className?: string) {
  const platformId = id.toLowerCase();
  switch (platformId) {
    case "windows":
      return <FaWindows className={className} />;
    case "macos":
      return <FaApple className={className} />;
    case "linux":
      return <FaLinux className={className} />;
    default:
      return null;
  }
}

function getPlatformDisplayName(id: string): string {
  const platformId = id.toLowerCase();
  switch (platformId) {
    case "windows":
      return "Windows";
    case "macos":
      return "macOS";
    case "linux":
      return "Linux";
    default:
      return id;
  }
}

export default function DownloadButton({
  disabled,
  executables,
  buildLog,
}: DownloadButtonProps) {
  const menuRef = useRef<HTMLButtonElement>(null);
  const [showMobileWarning, setShowMobileWarning] = useState(false);
  const [mobileAcknowledged, setMobileAcknowledged] = useState(() => {
    if (typeof window === "undefined") return false;
    return sessionStorage.getItem(MOBILE_WARNING_KEY) === "true";
  });

  function getExecutable(id: string) {
    return executables?.find((e) => e.id.toLowerCase() === id.toLowerCase());
  }

  const mobile = isMobile();
  const detectedPlatform = mobile ? null : getSystemType();
  const platformExecutable = detectedPlatform ? getExecutable(detectedPlatform) : null;
  const canAutoDownload = platformExecutable != null;

  function acknowledgeMobileWarning() {
    sessionStorage.setItem(MOBILE_WARNING_KEY, "true");
    setMobileAcknowledged(true);
  }

  function handleMobileButtonClick() {
    if (!mobileAcknowledged) {
      setShowMobileWarning(true);
    } else {
      menuRef.current?.click();
    }
  }

  function handleMobileWarningClose() {
    setShowMobileWarning(false);
  }

  function handleMobileWarningContinue() {
    acknowledgeMobileWarning();
    setShowMobileWarning(false);
    menuRef.current?.click();
  }

  async function handleDownload(id: string) {
    const executable = getExecutable(id);
    if (executable == null) {
      console.error(`Executable ${id} not found, cannot download`);
      return;
    }

    // Open the download link in a new tab
    window.open(withBackend(`/download/${executable.id}`), "_blank");
  }

  function handleDownloadAutomatic() {
    if (canAutoDownload && detectedPlatform) {
      handleDownload(detectedPlatform);
    }
  }

  return (
    <>
      <MobileWarningModal
        open={showMobileWarning}
        onClose={handleMobileWarningClose}
        onContinue={handleMobileWarningContinue}
      />
      <div
        className={cn(
          "[&>*]:py-1 overflow-clip transition-[background-color] text-sm/6 flex items-center shadow-inner align-middle text-white focus:outline-none data-[focus]:outline-1 data-[focus]:outline-white",
          !disabled
            ? "divide-white/[0.2] shadow-white/10 bg-emerald-800 data-[hover]:bg-emerald-700 data-[open]:bg-emerald-700"
            : "divide-white/[0.1] shadow-white/5 animate-pulse-dark data-[hover]:bg-[#064e3b] cursor-wait",
          "rounded-md divide-x h-full rounded-l-md"
        )}
      >
        <Button
          onClick={
            mobile
              ? handleMobileButtonClick
              : canAutoDownload
                ? handleDownloadAutomatic
                : undefined
          }
          suppressHydrationWarning
          disabled={disabled || (!mobile && !canAutoDownload)}
          className={cn("pl-3 font-semibold pr-2.5", {
            "hover:bg-white/5 cursor-pointer": !disabled && (mobile || canAutoDownload),
            "cursor-default": !mobile && !canAutoDownload,
          })}
        >
          {mobile
            ? "Download for Desktop"
            : canAutoDownload && detectedPlatform
              ? `Download for ${getPlatformDisplayName(detectedPlatform)}`
              : "Download"}
        </Button>
      <Menu>
        {mobile && !mobileAcknowledged ? (
          <button
            onClick={handleMobileButtonClick}
            disabled={disabled}
            className={cn("pl-1.5 min-h-8 pr-2 py-1", {
              "hover:bg-white/5": !disabled,
            })}
          >
            <ChevronDownIcon className="size-4 fill-white/60" />
          </button>
        ) : (
          <MenuButton
            ref={menuRef}
            suppressHydrationWarning
            disabled={disabled}
            className={cn("pl-1.5 text-transparent min-h-8 pr-2", {
              "hover:bg-white/5": !disabled,
            })}
          >
            <ChevronDownIcon className="size-4 fill-white/60" />
          </MenuButton>
        )}
        <MenuItems
          transition
          anchor="bottom end"
          className="w-40 z-20 mt-1 origin-top-right rounded-xl border border-white/[0.08] bg-zinc-900 shadow-md p-1 text-sm/6 text-zinc-200 transition duration-100 ease-out [--anchor-gap:var(--spacing-1)] focus:outline-none data-[closed]:scale-95 data-[closed]:opacity-0"
        >
          {executables?.map((executable) => (
            <MenuItem key={executable.id}>
              <button
                className="group flex w-full items-center justify-between gap-2 rounded-lg py-1.5 pl-2 pr-2.5 data-[focus]:bg-white/10"
                onClick={() => handleDownload(executable.id)}
              >
                <div className="flex items-center gap-1.5">
                  {getPlatformIcon(executable.id, "size-4 fill-white/40")}
                  {getPlatformDisplayName(executable.id)}
                </div>
                <div className="text-xs text-zinc-500">
                  {(executable.size / 1024 / 1024).toFixed(1)} MiB
                </div>
              </button>
            </MenuItem>
          ))}
          {buildLog != null ? (
            <>
              <MenuSeparator className="my-1 h-px bg-white/10" />
              <MenuItem>
                <a
                  className="group flex w-full items-center gap-2 rounded-lg py-1.5 px-2 data-[focus]:bg-white/10"
                  href={buildLog.startsWith('/') ? withBackend(buildLog) : buildLog}
                  target="_blank"
                >
                  <BeakerIcon className="size-4 fill-white/40" />
                  Build Logs
                </a>
              </MenuItem>
            </>
          ) : null}
        </MenuItems>
      </Menu>
    </div>
    </>
  );
}
