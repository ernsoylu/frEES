import React, { Suspense, lazy } from 'react'
import ReactDOM from 'react-dom/client'
import '@mantine/core/styles.css'
import '@mantine/notifications/styles.css'
import '@mantine/spotlight/styles.css'
// KaTeX's stylesheet is imported by Latex.tsx, its only consumer, NOT here.
// Importing it at the entry pulled the whole `katex` manual chunk onto the boot
// path: manualChunks routes anything matching /katex/ — the stylesheet included
// — into that chunk, so the entry's CSS dependency made its 254 kB of JS a
// static import of index.js, and through the chunk graph of every other chunk
// in the app. KaTeX only ever renders inside <Latex>, which only the (lazy)
// Help page mounts.
import { Center, createTheme, Loader, MantineProvider } from '@mantine/core'
import { Notifications } from '@mantine/notifications'
import ErrorBoundary from './ErrorBoundary'
import { setupPwa } from './pwa'
import './index.css'

// The editor app and the (separate /help route) Help page are split into their
// own chunks and loaded on demand, so visiting the editor never downloads the
// large docs catalog and example library that only the Help page needs.
const App = lazy(() => import('./App'))
const HelpPage = lazy(() => import('./HelpPage'))

// A deploy replaces the hashed /assets chunks, so a tab left open across it
// 404s on its next lazy import ("Failed to fetch dynamically imported
// module"). Reload once to pick up the fresh index.html; if the failure
// repeats within a minute (a genuinely broken deploy, not a stale tab), let
// the error through to the ErrorBoundary instead of reload-looping.
globalThis.addEventListener('vite:preloadError', (event) => {
  const RELOADED_AT_KEY = 'frees.chunkReloadAt'
  const last = Number(sessionStorage.getItem(RELOADED_AT_KEY) ?? 0)
  if (Date.now() - last > 60_000) {
    sessionStorage.setItem(RELOADED_AT_KEY, String(Date.now()))
    event.preventDefault()
    globalThis.location.reload()
  }
})

const theme = createTheme({
  primaryColor: 'teal',
  fontFamilyMonospace:
    "'Cascadia Code', 'Fira Code', ui-monospace, 'SF Mono', monospace",
  defaultRadius: 'md',
})

// Route relative to the deploy base (Wave E, closing Phase 11's gap 4):
// `pathname === '/help'` only worked for an origin-root deploy. BASE_URL is
// '/' in dev and whatever `vite build --base` was given, so a sub-path
// deploy's `/tools/frees/help` routes correctly too. Trailing-slash tolerant.
const helpPath = `${import.meta.env.BASE_URL.replace(/\/$/, '')}/help`
const isHelpPage = globalThis.location.pathname.replace(/\/$/, '') === helpPath

// Register the service worker (no-op in dev, where the plugin disables it).
setupPwa()

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <MantineProvider theme={theme} defaultColorScheme="dark">
      <Notifications position="bottom-right" />
      <ErrorBoundary>
        <Suspense
          fallback={
            <Center h="100vh">
              <Loader color="teal" />
            </Center>
          }
        >
          {isHelpPage ? <HelpPage /> : <App />}
        </Suspense>
      </ErrorBoundary>
    </MantineProvider>
  </React.StrictMode>,
)
