"use client"

import * as React from "react"

import type { Locale } from "@/lib/i18n/config"
import type { Dictionary } from "@/lib/i18n/dictionaries/en"

type I18nContextValue = {
  locale: Locale
  dictionary: Dictionary
}

const I18nContext = React.createContext<I18nContextValue | null>(null)

export function I18nProvider({
  locale,
  dictionary,
  children,
}: React.PropsWithChildren<I18nContextValue>) {
  React.useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])

  const value = React.useMemo(
    () => ({ locale, dictionary }),
    [locale, dictionary]
  )

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}

function useI18n() {
  const context = React.useContext(I18nContext)
  if (!context) {
    throw new Error("useI18n must be used within an I18nProvider")
  }
  return context
}

export function useTranslations() {
  return useI18n().dictionary
}

export function useLocale() {
  return useI18n().locale
}
