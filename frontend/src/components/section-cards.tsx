"use client"

import { motion, useReducedMotion, type Variants } from "motion/react"

import { Badge } from "@/components/ui/badge"
import {
  Card,
  CardAction,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { useTranslations } from "@/components/providers/i18n-provider"
import { TrendingUpIcon, TrendingDownIcon } from "lucide-react"

const MotionCard = motion.create(Card)

const containerVariants: Variants = {
  hidden: {},
  show: { transition: { staggerChildren: 0.09, delayChildren: 0.05 } },
}

const cardVariants: Variants = {
  hidden: { opacity: 0, y: 16, scale: 0.98 },
  show: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: { duration: 0.45, ease: [0.22, 1, 0.36, 1] },
  },
}

export function SectionCards() {
  const t = useTranslations()
  const reduce = useReducedMotion()

  return (
    <motion.div
      variants={containerVariants}
      initial={reduce ? false : "hidden"}
      animate="show"
      className="grid grid-cols-1 gap-4 px-4 *:data-[slot=card]:bg-linear-to-t *:data-[slot=card]:from-primary/5 *:data-[slot=card]:to-card *:data-[slot=card]:shadow-xs lg:px-6 @xl/main:grid-cols-2 @5xl/main:grid-cols-4 dark:*:data-[slot=card]:bg-card"
    >
      <MotionCard variants={cardVariants} className="@container/card">
        <CardHeader>
          <CardDescription>{t.dashboard.totalRevenue}</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            $1,250.00
          </CardTitle>
          <CardAction>
            <Badge variant="outline">
              <TrendingUpIcon
              />
              +12.5%
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="line-clamp-1 flex gap-2 font-medium">
            {t.dashboard.totalRevenueTrend}{" "}
            <TrendingUpIcon className="size-4" />
          </div>
          <div className="text-muted-foreground">
            {t.dashboard.totalRevenueSub}
          </div>
        </CardFooter>
      </MotionCard>
      <MotionCard variants={cardVariants} className="@container/card">
        <CardHeader>
          <CardDescription>{t.dashboard.newCustomers}</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            1,234
          </CardTitle>
          <CardAction>
            <Badge variant="outline">
              <TrendingDownIcon
              />
              -20%
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="line-clamp-1 flex gap-2 font-medium">
            {t.dashboard.newCustomersTrend}{" "}
            <TrendingDownIcon className="size-4" />
          </div>
          <div className="text-muted-foreground">
            {t.dashboard.newCustomersSub}
          </div>
        </CardFooter>
      </MotionCard>
      <MotionCard variants={cardVariants} className="@container/card">
        <CardHeader>
          <CardDescription>{t.dashboard.activeAccounts}</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            45,678
          </CardTitle>
          <CardAction>
            <Badge variant="outline">
              <TrendingUpIcon
              />
              +12.5%
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="line-clamp-1 flex gap-2 font-medium">
            {t.dashboard.activeAccountsTrend}{" "}
            <TrendingUpIcon className="size-4" />
          </div>
          <div className="text-muted-foreground">{t.dashboard.activeAccountsSub}</div>
        </CardFooter>
      </MotionCard>
      <MotionCard variants={cardVariants} className="@container/card">
        <CardHeader>
          <CardDescription>{t.dashboard.growthRate}</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            4.5%
          </CardTitle>
          <CardAction>
            <Badge variant="outline">
              <TrendingUpIcon
              />
              +4.5%
            </Badge>
          </CardAction>
        </CardHeader>
        <CardFooter className="flex-col items-start gap-1.5 text-sm">
          <div className="line-clamp-1 flex gap-2 font-medium">
            {t.dashboard.growthRateTrend}{" "}
            <TrendingUpIcon className="size-4" />
          </div>
          <div className="text-muted-foreground">{t.dashboard.growthRateSub}</div>
        </CardFooter>
      </MotionCard>
    </motion.div>
  )
}
