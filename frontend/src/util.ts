import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export { type ClassValue };
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function plural(text: string, count: number) {
  return `${text}${count === 1 ? "" : "s"}`;
}
