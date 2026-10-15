"use client"

import * as React from "react"

import { NavMain, type NavItem } from "@/components/nav-main"
import { NavUser } from "@/components/nav-user"
import { useTranslations } from "@/components/providers/i18n-provider"
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import {
  BoxIcon,
  CalendarDaysIcon,
  CalendarIcon,
  ClipboardListIcon,
  CommandIcon,
  FileTextIcon,
  HomeIcon,
  PillIcon,
  ReceiptTextIcon,
  TrendingUpIcon,
  UserRoundCheckIcon,
  UserRoundIcon,
  UsersRoundIcon,
} from "lucide-react"

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const t = useTranslations()

  const user = {
    name: "Test Doctor 1",
    email: "testdoc1@projectd.com",
    avatar: "/avatars/shadcn.jpg",
  }

  const navMain: NavItem[] = [
    { title: t.menu.home, url: "#", icon: <HomeIcon /> },
    {
      title: t.menu.todaysCustomers,
      url: "#",
      icon: <CalendarDaysIcon />,
      badge: 1,
      variant: "attention",
    },
    { title: t.menu.customers, url: "#", icon: <UserRoundIcon />, isActive: true },
    { title: t.menu.specialCustomers, url: "#", icon: <UserRoundCheckIcon /> },
    { title: t.menu.appointments, url: "#", icon: <CalendarIcon /> },
    { title: t.menu.billing, url: "#", icon: <ReceiptTextIcon />, badge: 1 },
    { title: t.menu.invoices, url: "#", icon: <FileTextIcon /> },
    { title: t.menu.prescription, url: "#", icon: <ClipboardListIcon /> },
    { title: t.menu.services, url: "#", icon: <BoxIcon /> },
    { title: t.menu.medicines, url: "#", icon: <PillIcon /> },
    { title: t.menu.finance, url: "#", icon: <TrendingUpIcon /> },
    { title: t.menu.staff, url: "#", icon: <UsersRoundIcon /> },
  ]

  return (
    <Sidebar collapsible="offcanvas" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="data-[slot=sidebar-menu-button]:p-1.5!"
              render={<a href="#" />}
            >
              <CommandIcon className="size-5!" />
              <span className="text-base font-semibold">Project D</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={navMain} />
      </SidebarContent>
      <SidebarFooter>
        <NavUser user={user} />
      </SidebarFooter>
    </Sidebar>
  )
}
