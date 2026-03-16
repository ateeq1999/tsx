export function Footer() {
  return (
    <footer className="site-footer mt-auto py-8">
      <div className="page-wrap flex flex-col items-center justify-between gap-4 text-sm sm:flex-row">
        <p style={{ color: "var(--sea-ink-soft)" }}>
          &copy; {new Date().getFullYear()} tsx registry — universal code pattern registry
        </p>
        <div className="flex gap-6" style={{ color: "var(--sea-ink-soft)" }}>
          <a href="/browse" className="nav-link">Browse</a>
          <a href="https://github.com/ateeq1999/tsx" target="_blank" rel="noreferrer" className="nav-link">GitHub</a>
          <a href="/dashboard" className="nav-link">Dashboard</a>
        </div>
      </div>
    </footer>
  )
}
