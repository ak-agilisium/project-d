import { NextResponse } from "next/server"

/**
 * Login proxy (BFF).
 *
 * Validates credentials and issues an httpOnly session cookie on success.
 * Credentials are hardcoded for now — replace this block with a call to the
 * real authentication backend when it exists; the rest of the flow stays the same.
 */
const VALID_EMAIL = "testdoc1@projectd.com"
const VALID_PASSWORD = "testDoc1@123"

export const SESSION_COOKIE = "pd_session"

export async function POST(request: Request) {
  const { email, password } = await request
    .json()
    .catch(() => ({ email: "", password: "" }))

  const ok =
    typeof email === "string" &&
    typeof password === "string" &&
    email.trim().toLowerCase() === VALID_EMAIL &&
    password === VALID_PASSWORD

  if (!ok) {
    return NextResponse.json(
      { ok: false, error: "Invalid email or password." },
      { status: 401 }
    )
  }

  const response = NextResponse.json({ ok: true })
  response.cookies.set(SESSION_COOKIE, "authenticated", {
    httpOnly: true,
    sameSite: "lax",
    path: "/",
    maxAge: 60 * 60 * 8, // 8 hours
    secure: process.env.NODE_ENV === "production",
  })
  return response
}
