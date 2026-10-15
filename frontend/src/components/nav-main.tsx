"use client"

import { cn } from "cn"

import {
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"

export type NavItem = {
  title: string
  url: string
  icon?: React.ReactNode
  isActive?: boolean
  badge?: number
  variant?: "default" | "attention"
}

export function NavMain({ items }: { items: NavItem[] }) {
  return (
    <SidebarGroup>
      <SidebarGroupContent>
        <SidebarMenu className="gap-0.5">
          {items.map((item) => (
            <SidebarMenuItem key={item.title}>
              <SidebarMenuButton
                tooltip={item.title}
                isActive={item.isActive}
                render={<a href={item.url} />}
                className={cn(
                  "h-10 gap-3 rounded-xl px-3 text-[13px] font-medium [&_svg]:size-5",
                  "data-active:bg-blue-600 data-active:text-white data-active:hover:bg-blue-600 data-active:hover:text-white",
                  item.variant === "attention" &&
                    "bg-red-50 text-red-600 hover:bg-red-100 hover:text-red-600 dark:bg-red-950/40 dark:text-red-400 dark:hover:bg-red-950/60"
                )}
              >
                {item.icon}
                <span>{item.title}</span>
              </SidebarMenuButton>
              {item.badge != null && (
                <SidebarMenuBadge className="top-2.5 right-3 size-5 rounded-full bg-red-600 text-white peer-hover/menu-button:text-white peer-data-active/menu-button:text-white peer-data-[size=default]/menu-button:top-2.5">
                  {item.badge}
                </SidebarMenuBadge>
              )}
            </SidebarMenuItem>
          ))}
        </SidebarMenu>
      </SidebarGroupContent>
    </SidebarGroup>
  )
}
