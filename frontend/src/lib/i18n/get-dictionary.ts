import type { Locale } from "./config"
import type { Dictionary } from "./dictionaries/en"

const loaders: Record<Locale, () => Promise<Dictionary>> = {
  en: () => import("./dictionaries/en").then((m) => m.default),
  km: () => import("./dictionaries/km").then((m) => m.default),
}

export async function getDictionary(locale: Locale): Promise<Dictionary> {
  return (loaders[locale] ?? loaders.en)()
}
