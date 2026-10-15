"use client"

import * as React from "react"
import { LanguagesIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { useLocale, useTranslations } from "@/components/providers/i18n-provider"
import { setLocale } from "@/lib/actions/locale"
import { locales, localeLabels } from "@/lib/i18n/config"

export function LanguageSwitcher() {
  const locale = useLocale()
  const t = useTranslations()
  const [isPending, startTransition] = React.useTransition()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button
            variant="outline"
            size="icon"
            aria-label={t.language.switch}
            disabled={isPending}
          />
        }
      >
        <LanguagesIcon className="size-4" />
        <span className="sr-only">{t.language.switch}</span>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {locales.map((value) => (
          <DropdownMenuItem
            key={value}
            data-active={locale === value}
            className="data-[active=true]:bg-accent data-[active=true]:text-accent-foreground"
            onClick={() => startTransition(() => setLocale(value))}
          >
            {localeLabels[value]}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
