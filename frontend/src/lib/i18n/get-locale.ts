import { cookies } from "next/headers"

import { defaultLocale, isLocale, localeCookieName, type Locale } from "./config"

export async function getLocale(): Promise<Locale> {
  const cookieStore = await cookies()
  const value = cookieStore.get(localeCookieName)?.value
  return isLocale(value) ? value : defaultLocale
}
