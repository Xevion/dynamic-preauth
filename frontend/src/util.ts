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
