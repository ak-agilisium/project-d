"use client"

import { useTheme } from "next-themes"
import { MonitorIcon, MoonIcon, SunIcon } from "lucide-react"

import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { useTranslations } from "@/components/providers/i18n-provider"

export function ThemeToggle() {
  const { theme, setTheme } = useTheme()
  const t = useTranslations()

  const options = [
    { value: "light", label: t.theme.light, icon: SunIcon },
    { value: "dark", label: t.theme.dark, icon: MoonIcon },
    { value: "system", label: t.theme.system, icon: MonitorIcon },
  ] as const

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        render={
          <Button variant="outline" size="icon" aria-label={t.theme.toggle} />
        }
      >
        <SunIcon className="size-4 scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90" />
        <MoonIcon className="absolute size-4 scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0" />
        <span className="sr-only">{t.theme.toggle}</span>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {options.map(({ value, label, icon: Icon }) => (
          <DropdownMenuItem
            key={value}
            data-active={theme === value}
            className="data-[active=true]:bg-accent data-[active=true]:text-accent-foreground"
            onClick={() => setTheme(value)}
          >
            <Icon />
            {label}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
