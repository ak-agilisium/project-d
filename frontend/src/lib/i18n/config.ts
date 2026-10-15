export const locales = ["en", "km"] as const

export type Locale = (typeof locales)[number]

export const defaultLocale: Locale = "en"

export const localeCookieName = "NEXT_LOCALE"

export const localeLabels: Record<Locale, string> = {
  en: "English",
  km: "ខ្មែរ",
}

export function isLocale(value: string | undefined): value is Locale {
  return (locales as readonly string[]).includes(value ?? "")
}
