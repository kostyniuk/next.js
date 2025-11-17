import type { NextRequest } from 'next/server'
import { NextResponse } from 'next/server'

export default function proxy(request: NextRequest) {
  if (request.nextUrl.pathname === '/home') {
    return NextResponse.redirect(new URL('/', request.url))
  }
  return NextResponse.next()
}
