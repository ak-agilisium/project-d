"use client"

import { LoginForm } from "@/components/login-form"
import { LoginBackdrop } from "@/components/three/login-backdrop"
import { GalleryVerticalEndIcon } from "lucide-react"

export default function LoginPage() {
  return (
    <div className="grid min-h-svh lg:grid-cols-2">
      {/* Left: brand lockup, form, footer — one left-aligned column */}
      <div className="relative flex flex-col gap-10 bg-linear-to-b from-background to-muted/40 p-8 md:p-12">
        <a href="#" className="flex w-fit items-center gap-2.5 font-semibold">
          <div className="flex size-8 items-center justify-center rounded-md bg-linear-to-br from-blue-600 to-sky-400 text-white shadow-sm">
            <GalleryVerticalEndIcon className="size-4" />
          </div>
          <span className="tracking-tight">Project D</span>
        </a>

        <div className="flex flex-1 items-center">
          <div className="w-full max-w-sm">
            <LoginForm />
          </div>
        </div>

        <p className="text-xs text-muted-foreground">
          © 2026 Project D. All rights reserved.
        </p>
      </div>

      {/* Right: animated geometric backdrop — the hero */}
      <div className="relative hidden overflow-hidden bg-muted lg:block">
        <div className="absolute inset-0 bg-linear-to-br from-blue-500/10 via-background to-sky-400/10 dark:from-blue-500/10 dark:via-background dark:to-sky-400/5" />
        <LoginBackdrop />
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-44 bg-linear-to-t from-background/85 to-transparent" />
        <div className="absolute bottom-10 left-10 max-w-xs">
          <p className="text-sm font-medium text-foreground/85">
            Intelligence for every decision.
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            Dashboards, metrics, and insights in one place.
          </p>
        </div>
      </div>
    </div>
  )
}
