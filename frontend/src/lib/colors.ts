/**
 * Vault / item color palette.
 * Maps integer colorCode (stored in DB) to Tailwind classes.
 */

export interface ColorDef {
  bg: string;
  text: string;
  ring: string;
  label: string;
}

export const VAULT_COLORS: ColorDef[] = [
  { bg: "bg-blue-500/15", text: "text-blue-500", ring: "ring-blue-500/30", label: "Blue" },
  { bg: "bg-emerald-500/15", text: "text-emerald-500", ring: "ring-emerald-500/30", label: "Green" },
  { bg: "bg-violet-500/15", text: "text-violet-500", ring: "ring-violet-500/30", label: "Purple" },
  { bg: "bg-red-500/15", text: "text-red-500", ring: "ring-red-500/30", label: "Red" },
  { bg: "bg-orange-500/15", text: "text-orange-500", ring: "ring-orange-500/30", label: "Orange" },
  { bg: "bg-teal-500/15", text: "text-teal-500", ring: "ring-teal-500/30", label: "Teal" },
  { bg: "bg-pink-500/15", text: "text-pink-500", ring: "ring-pink-500/30", label: "Pink" },
  { bg: "bg-amber-500/15", text: "text-amber-500", ring: "ring-amber-500/30", label: "Yellow" },
];

export function getColor(code: number): ColorDef {
  // VAULT_COLORS is a non-empty const array, so the fallback is just for TS strictness
  const idx = code % VAULT_COLORS.length;
  return VAULT_COLORS[idx] as ColorDef;
}
