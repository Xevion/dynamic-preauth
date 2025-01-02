import { WindowIcon } from "@heroicons/react/16/solid";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export { type ClassValue };
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function plural(text: string, count: number) {
  return `${text}${count === 1 ? "" : "s"}`;
}

export type Platform = "windows" | "mac" | "linux";

export function os(): Platform | "other" {
  if (navigator.userAgent.includes("win")) return "windows";
  else if (navigator.userAgent.includes("mac")) return "mac";
  else if (navigator.userAgent.includes("linux")) return "linux";
  return "other";
}

export function toHex(value: number): string {
  return "0x" + value.toString(16).toUpperCase();
}

// Either uses the current window's host, or the backend API host depending on the environment
// If the second argument is provided, the first becomes the protocol. The protocol of the window is used otherwise.
// Example: withBackend('/download') -> 'localhost:5800/download'
// Example: withBackend('/download') -> 'dynamic-preauth.xevion.dev/download'
// Example: withBackend('https://', '/download') -> 'https://localhost:5800/download'
// Example: withBackend('https://', '/download') -> 'https://dynamic-preauth.xevion.dev/download'
export function withBackend(arg1: string, arg2?: string): string {
  const path = arg2 != undefined ? arg2 : arg1;
  const protocol = arg2 != undefined ? arg1 : window.location.protocol + "//";
  const host = import.meta.env.DEV ? "localhost:5800" : window.location.host;
  return protocol + host + path;
}
