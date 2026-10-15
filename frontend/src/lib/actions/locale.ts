"use server"

import { cookies } from "next/headers"

import { isLocale, localeCookieName, type Locale } from "@/lib/i18n/config"

export async function setLocale(locale: Locale) {
  if (!isLocale(locale)) return

  const cookieStore = await cookies()
  cookieStore.set(localeCookieName, locale, {
    path: "/",
    maxAge: 60 * 60 * 24 * 365,
    sameSite: "lax",
  })
}
