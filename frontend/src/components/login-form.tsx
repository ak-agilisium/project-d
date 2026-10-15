"use client"

import * as React from "react"
import { useRouter } from "next/navigation"
import { cn } from "cn"

import { Button } from "@/components/ui/button"
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { useTranslations } from "@/components/providers/i18n-provider"

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"form">) {
  const t = useTranslations()
  const router = useRouter()
  const [email, setEmail] = React.useState("")
  const [password, setPassword] = React.useState("")
  const [error, setError] = React.useState<string | null>(null)
  const [isLoading, setIsLoading] = React.useState(false)

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    setIsLoading(true)
    try {
      const response = await fetch("/api/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      })

      if (response.ok) {
        router.push("/dashboard")
        router.refresh()
        return
      }

      const data = await response.json().catch(() => null)
      setError(data?.error ?? "Something went wrong. Please try again.")
      setIsLoading(false)
    } catch {
      setError("Unable to reach the server. Please try again.")
      setIsLoading(false)
    }
  }

  return (
    <form
      className={cn("flex flex-col gap-8", className)}
      {...props}
      onSubmit={handleSubmit}
    >
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-semibold tracking-tight text-balance">
          {t.login.title}
        </h1>
        <p className="text-sm text-muted-foreground">{t.login.subtitle}</p>
      </div>

      <FieldGroup className="gap-6">
        <Field>
          <FieldLabel htmlFor="email">{t.login.email}</FieldLabel>
          <Input
            id="email"
            name="email"
            type="email"
            placeholder="you@company.com"
            autoComplete="email"
            value={email}
            onChange={(event) => setEmail(event.target.value)}
            required
            className="h-11 border-b-[1.5px] caret-blue-600 focus-visible:border-b-blue-600"
          />
        </Field>

        <Field>
          <div className="flex items-center justify-between">
            <FieldLabel htmlFor="password">{t.login.password}</FieldLabel>
            <a
              href="#"
              className="text-xs font-medium text-muted-foreground underline-offset-4 transition-colors hover:text-blue-600 hover:underline"
            >
              {t.login.forgotPassword}
            </a>
          </div>
          <Input
            id="password"
            name="password"
            type="password"
            autoComplete="current-password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            required
            className="h-11 border-b-[1.5px] caret-blue-600 focus-visible:border-b-blue-600"
          />
        </Field>

        {error && (
          <p role="alert" className="-mt-2 text-sm text-destructive">
            {error}
          </p>
        )}

        <Button type="submit" disabled={isLoading} className="mt-2 h-11 w-full">
          {t.login.submit}
        </Button>
      </FieldGroup>
    </form>
  )
}
