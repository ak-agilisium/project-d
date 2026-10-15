import en from "@/lib/i18n/dictionaries/en"
import { defaultLocale } from "@/lib/i18n/config"
import { getDictionary } from "@/lib/i18n/get-dictionary"
import { getLocale } from "@/lib/i18n/get-locale"

import { I18nProvider } from "./i18n-provider"

export async function LocaleScope({
  children,
}: {
  children: React.ReactNode
}) {
  const locale = await getLocale()
  const dictionary = await getDictionary(locale)

  return (
    <I18nProvider locale={locale} dictionary={dictionary}>
      {children}
    </I18nProvider>
  )
}

export function LocaleFallback({ children }: { children: React.ReactNode }) {
  return (
    <I18nProvider locale={defaultLocale} dictionary={en}>
      {children}
    </I18nProvider>
  )
}
