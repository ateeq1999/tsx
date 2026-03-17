import {
  HeadContent,
  Link,
  Scripts,
  createRootRouteWithContext,
} from "@tanstack/react-router"
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools"
import { TanStackDevtools } from "@tanstack/react-devtools"
import { ThemeProvider } from "next-themes"
import { Toaster } from "@/components/ui/sonner"
import { Button } from "@/components/ui/button"
import { Header } from "@/components/Header"
import { Footer } from "@/components/Footer"
import TanStackQueryProvider from "@/integrations/tanstack-query/root-provider"
import TanStackQueryDevtools from "@/integrations/tanstack-query/devtools"

import "@/env" // validate env vars at startup
import appCss from "@/styles.css?url"

import type { QueryClient } from "@tanstack/react-query"

interface MyRouterContext {
  queryClient: QueryClient
}

const THEME_INIT_SCRIPT = `(function(){try{var stored=window.localStorage.getItem('theme');var mode=(stored==='light'||stored==='dark'||stored==='auto')?stored:'auto';var prefersDark=window.matchMedia('(prefers-color-scheme: dark)').matches;var resolved=mode==='auto'?(prefersDark?'dark':'light'):mode;var root=document.documentElement;root.classList.remove('light','dark');root.classList.add(resolved);if(mode==='auto'){root.removeAttribute('data-theme')}else{root.setAttribute('data-theme',mode)}root.style.colorScheme=resolved;}catch(e){}})();`

function NotFoundPage() {
  return (
    <div className="page-wrap py-24 rise-in text-center">
      <p className="mb-2 text-6xl font-bold" style={{ color: "var(--lagoon)" }}>404</p>
      <h1 className="mb-3 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Page not found</h1>
      <p className="mb-8 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
        The page you're looking for doesn't exist or has been moved.
      </p>
      <div className="flex justify-center gap-3">
        <Button asChild>
          <Link to="/">Go home</Link>
        </Button>
        <Button variant="outline" asChild>
          <Link to="/browse">Browse packages</Link>
        </Button>
      </div>
    </div>
  )
}

function GlobalErrorPage({ error, reset }: { error: Error; reset: () => void }) {
  return (
    <div className="page-wrap py-24 rise-in text-center">
      <p className="mb-2 text-5xl font-bold" style={{ color: "var(--lagoon)" }}>!</p>
      <h1 className="mb-3 text-2xl font-bold" style={{ color: "var(--sea-ink)" }}>Something went wrong</h1>
      <p className="mb-4 text-sm" style={{ color: "var(--sea-ink-soft)" }}>
        An unexpected error occurred. You can try again or return home.
      </p>
      {error?.message && (
        <pre className="mx-auto mb-8 max-w-lg overflow-x-auto rounded-lg px-4 py-3 text-left text-xs" style={{ background: "var(--line)", color: "var(--sea-ink-soft)" }}>
          {error.message}
        </pre>
      )}
      <div className="flex justify-center gap-3">
        <Button onClick={reset}>Try again</Button>
        <Button variant="outline" asChild>
          <Link to="/">Go home</Link>
        </Button>
      </div>
    </div>
  )
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
  head: () => ({
    meta: [
      { charSet: "utf-8" },
      { name: "viewport", content: "width=device-width, initial-scale=1" },
      { title: "tsx registry — universal code pattern registry" },
      { name: "description", content: "Install and publish reusable code patterns for TanStack Start projects" },
    ],
    links: [
      { rel: "stylesheet", href: appCss },
      { rel: "manifest", href: "/manifest.json" },
      { rel: "apple-touch-icon", href: "/logo192.png" },
    ],
  }),
  notFoundComponent: NotFoundPage,
  errorComponent: GlobalErrorPage,
  shellComponent: RootDocument,
})

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_INIT_SCRIPT }} />
        <HeadContent />
      </head>
      <body className="antialiased [overflow-wrap:anywhere]">
        <TanStackQueryProvider>
          <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
            <div className="flex min-h-screen flex-col">
              <Header />
              <main className="flex-1">{children}</main>
              <Footer />
            </div>
            <Toaster />
          </ThemeProvider>
          <TanStackDevtools
            config={{ position: "bottom-right" }}
            plugins={[
              { name: "Tanstack Router", render: <TanStackRouterDevtoolsPanel /> },
              TanStackQueryDevtools,
            ]}
          />
        </TanStackQueryProvider>
        <Scripts />
      </body>
    </html>
  )
}
