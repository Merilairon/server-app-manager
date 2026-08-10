# UI Design Specification — Server App Manager

High-fidelity UI mockups for the Server App Manager platform. Each mockup is a
standalone HTML file that renders in any browser and shares a single design
system stylesheet (`styles.css`). The designs are derived from the product
requirements (`docs/PRD.md`), user stories (`docs/USER_STORIES.md`), and the
target architecture (`docs/ARCHITECTURE.md`), and they map directly to the
Angular routes defined in `frontend/src/app/app.routes.ts`.

## Viewing

Open `index.html` in a browser to land on the design gallery, then click any
tile to open the full page mockup. For a quick local server:

```bash
cd designs && python3 -m http.server 8765
# open http://localhost:8765/index.html
```

## File structure

```
designs/
├── DESIGN.md              # this document
├── styles.css             # shared design system (all pages link this)
├── index.html             # gallery landing page
├── login.html             # authentication
├── home.html              # dashboard / overview
├── apps.html              # app store catalog
├── install-modal.html     # install-time parameter prompting + progress
├── my-apps.html           # installed apps lifecycle
├── containers.html        # container health dashboard
├── launch.html            # launch child container form
├── admin-users.html       # user + role management
├── admin-apps.html        # app definition management + YAML editor
├── admin-secrets.html     # Docker secrets CRUD
├── admin-backups.html     # backup history + live log stream
└── admin-settings.html    # SSL, backups, rate limits, security headers
```

---

## Design system

A single stylesheet (`styles.css`) defines tokens, layout primitives, and
reusable components so every page renders with a consistent visual language.

### Design principles

1. **Operational clarity** — status is always visible at a glance through
   color-coded badges, health rows, and live indicators.
2. **Density without noise** — tables and cards pack information tightly while
   generous spacing and a muted palette keep the surface calm.
3. **Dark, DevOps-native aesthetic** — the platform is administered by
   technical users running it on their own hardware; a dark theme reduces
   eye strain during long sessions and matches the infrastructure context.
4. **Consistent shell** — every authenticated page shares the same
   sidebar + topbar layout so navigation is predictable across the 12 pages.
5. **Progressive disclosure** — complex flows (install, launch) break into
   steps and side panels rather than one overwhelming form.

### Color tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--bg` | `#0b1020` | App background |
| `--bg-elev` | `#121a2e` | Topbar, table headers, hover states |
| `--panel` | `#0f1729` | Cards and panels |
| `--sidebar` | `#0a1120` | Navigation sidebar |
| `--border` | `#1f2b45` | Default dividers |
| `--border-strong` | `#2a3a5c` | Inputs, modal edges |
| `--text` | `#e6ecf5` | Primary text |
| `--text-muted` | `#94a3b8` | Secondary text |
| `--text-dim` | `#64748b` | Tertiary / hints |
| `--accent` | `#3b82f6` | Brand blue, primary actions, active nav |
| `--ok` | `#22c55e` | Healthy / success |
| `--warn` | `#f59e0b` | Starting / warning / unsaved |
| `--bad` | `#ef4444` | Unhealthy / error / destructive |
| `--info` | `#06b6d4` | Informational / streaming |

Each status color has a matching `*-soft` translucent variant used for badge
backgrounds (e.g. `--ok-soft` = `rgba(34,197,94,0.14)`).

### Typography

- **Font stack** — `-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
  Helvetica, Arial, sans-serif` for UI text.
- **Monospace** — `"SF Mono", "JetBrains Mono", Menlo, Consolas, monospace`
  for container IDs, URLs, YAML, env keys, and log output.
- **Base size** — 14px body, with a 1.5 line-height. Headings step down from
  16px (topbar) to 11px uppercase section labels.

### Layout shell

All authenticated pages use a two-column grid:

```
┌──────────┬──────────────────────────────────┐
│ Sidebar  │ Topbar (crumb · title · search)  │
│ 240px    ├──────────────────────────────────┤
│          │ Content (max-width 1280px)       │
│ Brand    │                                  │
│ Nav      │                                  │
│ User     │                                  │
└──────────┴──────────────────────────────────┘
```

- **Sidebar** — sticky, full-height. Brand logo, grouped nav (General +
  Administration), and a user card with avatar, name, and role.
- **Topbar** — 56px. Breadcrumb, page title, spacer, search, notifications.
- **Content** — 24px padding. Pages may opt into `content--wide` to drop the
  1280px max-width for dense tables and grids.

The login page (`login.html`) is the only page that bypasses the shell — it
uses a centered card on a gradient backdrop.

### Reusable components

| Component | Class | Description |
|-----------|-------|-------------|
| Card | `.card`, `.card__head`, `.card__body`, `.card__foot` | Surface container with optional header/footer dividers |
| Stat tile | `.stat` | Label + large value + delta indicator |
| Button | `.btn`, `.btn--primary`, `.btn--danger`, `.btn--ghost`, `.btn--sm`, `.btn--icon` | Action buttons |
| Badge | `.badge`, `.badge--ok/warn/bad/info/neutral/accent` | Status pill with colored dot |
| Table | `table.tbl` | Sortable-style header, hover row, `.unhealthy` row turns red |
| Form field | `.field`, `.input`, `.select`, `textarea` | Label + control + hint |
| Key/value row | `.kv-row` | Grid for env var / port mapping entry |
| App card | `.app-card` | Store tile with icon, name, category, description, footer |
| Modal | `.modal-overlay`, `.modal`, `.modal__head/body/foot` | Centered dialog with backdrop blur |
| Progress steps | `.progress`, `.progress__step` | Horizontal step indicator (done/active/pending) |
| Progress bar | `.bar`, `.bar__fill` | Linear meter for CPU/mem/disk |
| Log stream | `.log` | Terminal-style block with colored severity lines |
| Tabs | `.tabs`, `.tab` | Underline tab navigation |
| Avatar | `.avatar` | Gradient circle with initials |

### Iconography

Mockups use emoji glyphs as lightweight placeholders (e.g. ☁️ Nextcloud,
🎬 Jellyfin, 🐙 Gitea). These are stand-ins for a real icon set to be chosen
during implementation (Material Icons, Lucide, or Heroicons are all
candidates). Emoji were chosen for the mockups only because they render
without external assets.

---

## Page inventory

Each page below lists its purpose, key UI elements, and the user stories it
satisfies. User-story IDs reference `docs/USER_STORIES.md` and requirement IDs
reference `docs/PRD.md`.

### 1. Login — `login.html`

**Purpose.** Single sign-in entry point. Issues a JWT stored in an HttpOnly
cookie and redirects based on role.

**Key elements.**
- Centered card on a blue/purple radial-gradient backdrop.
- Brand block (logo + tagline).
- Username/email + password fields, "Remember me" checkbox, forgot-password
  link.
- "Server online" status badge.
- Footer note: *Protected by JWT · HttpOnly cookie · RBAC enforced*.

**Maps to.** US-16 / FR-AUTH-1 (Authentication & JWT).

### 2. Dashboard — `home.html`

**Purpose.** Landing page after login. Gives an at-a-glance overview of the
platform state and recent activity.

**Key elements.**
- Four stat tiles: Installed Apps, Healthy Containers, Unhealthy, Last Backup.
- *My Apps* table preview (app, status, URL, open) with "View all" link.
- *System Health* panel with CPU / Memory / Disk / Network progress bars.
- *Recent Activity* audit feed (time, user, action, resource, outcome).

**Maps to.** US-3 (health overview), US-21 (last backup), audit logging,
US-31 (dashboard entry point).

### 3. App Store — `apps.html`

**Purpose.** Browse the YAML-driven app catalog, filter by category, and
trigger one-click install.

**Key elements.**
- Category filter chips (All, Media, Productivity, Development, Security,
  Networking, Home) + sort dropdown.
- Responsive auto-fill grid of `.app-card` tiles, each with icon, name,
  category, description, rating, install count, and an **Install** button.
- Install buttons link to `install-modal.html`.

**Maps to.** US-6 / FR-APP-1 (one-click install), US-7 / FR-APP-2 (YAML app
definitions), US-14 (catalog browsing).

### 4. Install Modal — `install-modal.html`

**Purpose.** Collect install-time parameters, confirm the pre-install backup,
and walk the user through the install → health-check pipeline.

**Key elements.**
- Modal over a dimmed App Store backdrop.
- Four-step progress indicator: Configure → Review → Install → Health check.
- Required parameters: sub-domain (with `.local` suffix preview), admin
  username, admin password (marked as Docker secret), database select,
  storage quota.
- Optional environment-variable key/value rows (add/remove).
- Pre-install backup confirmation card.
- Footer: Cancel + "Install Nextcloud →".

**Maps to.** US-8 / FR-APP-3 (install-time parameter prompting), US-22
(rollback prerequisite — backup), US-25 / FR-RT-1 (live install status
steps).

### 5. My Apps — `my-apps.html`

**Purpose.** Lifecycle view of apps the user has installed: open, pause,
restart, uninstall, and observe live status.

**Key elements.**
- Status filter chips (All / Running / Stopped / Unhealthy).
- App cards showing status badge, sub-domain, version, CPU/memory, and action
  buttons (Open, Pause, Restart, Uninstall).
- A *Starting* card (Gitea) with a "Live status via SSE" note and Cancel.
- An *Unhealthy* card (Vaultwarden) with red border and "View logs".
- A *Stopped* card (Grafana) with a primary "▶ Start" action.

**Maps to.** US-9 / FR-UNI-1 (easy uninstall), US-25 (live status), US-3
(per-app health visibility).

### 6. Container Dashboard — `containers.html`

**Purpose.** Operations table for every container on the host with health,
resource usage, and real-time updates.

**Key elements.**
- Topbar badges: "Auto-refresh 10s" and "SSE connected".
- Stat tiles: Running / Starting / Unhealthy / Stopped counts.
- Table columns: Container ID, Image, Status, Health, CPU %, Mem %, Last
  checked, Logs action.
- Unhealthy row (`vaultwarden`) is highlighted red per US-3 acceptance
  criteria.
- Filter search + status dropdown + "Launch" button.

**Maps to.** US-3 / FR-CL-2 (container health dashboard), US-4 / FR-CL-3
(admin container view), US-25 (real-time updates).

### 7. Launch Container — `launch.html`

**Purpose.** Admin form to start an arbitrary child container with full
Docker run options.

**Key elements.**
- Two-column layout: form (left) + validation/preview (right).
- Form fields: image, container name, restart policy, env-var key/value rows,
  port mappings (host:container + protocol), command/entrypoint override.
- *Pre-flight checks* side panel: image pullability, port availability,
  resource sufficiency, command syntax, network — each with an OK badge.
- *Compose preview* side panel: rendered YAML in a log block.

**Maps to.** US-2 / FR-CL-1 (launch child containers), US-20 (secret
references), US-18 (validation).

### 8. Admin · Users — `admin-users.html`

**Purpose.** Manage user accounts and inspect the role/permission matrix.

**Key elements.**
- Tabs: Users / Roles & permissions / Audit log.
- Users table: avatar + name + email, role badge (admin/user), status
  (Active/Deactivated), last login, created date, Edit action.
- *Roles & permissions* reference table mapping permissions (apps.install,
  apps.uninstall.own/any, containers.launch, users.manage, secrets.manage,
  settings.manage) to admin vs user, sourced from `roles/roles.yaml`.

**Maps to.** US-13 / FR-AUTH-3 (admin user management), US-11/15 / FR-AUTH-2
(role-based permissions), audit logging.

### 9. Admin · Apps — `admin-apps.html`

**Purpose.** Manage app definitions: enable, disable, delete, and edit the
underlying YAML.

**Key elements.**
- Tabs: Catalog (48) / Enabled (12) / Disabled (3).
- Table: app icon + name, version, category, state badge, install count, YAML
  path, Edit/Disable (or Enable) actions.
- YAML editor card showing `store/nextcloud.yaml` with syntax-highlighted
  placeholders (`{{ subdomain }}`), param annotations, and an "Unsaved
  changes" badge.
- Footer actions: Discard, Validate schema, Save.

**Maps to.** US-12 / FR-ADMIN-1 (admin app management), US-7/14 / FR-APP-2
(YAML definitions), US-18 (schema validation).

### 10. Admin · Secrets — `admin-secrets.html`

**Purpose.** CRUD for Docker secrets with a hard rule that values are never
exposed after creation.

**Key elements.**
- Warning banner: values are never exposed; rotate to set a new value.
- Secrets table: name (mono), scope badge, used-by count, created, last
  rotated, Rotate + Delete actions.
- "New secret" modal: name, scope select, value textarea with a hint that it
  is encrypted at rest and unretrievable.

**Maps to.** US-20 / FR-ADMIN-2 (Docker secrets management), NF-SEC.

### 11. Admin · Backups & Logs — `admin-backups.html`

**Purpose.** Inspect backup history and tail live platform logs.

**Key elements.**
- Tabs: Backups / Audit log / Live logs.
- Stat tiles: Total backups, Success rate, Next nightly (02:00 UTC).
- Backup history table: timestamp, type (Full / Pre-install), trigger,
  size, status badge, Restore + Download actions. Header notes retention
  (14 days) and disk usage (142 GB).
- Live log stream: terminal block with timestamped, severity-colored lines
  (INFO/OK/WARN/ERROR) showing a real install flow (backup → catalog resolve
  → compose up → health check → audit).

**Maps to.** US-21 / FR-ADMIN-3 (backup & log settings), US-29 (log
streaming), US-22 (pre-install backup evidence), audit logging.

### 12. Admin · Settings — `admin-settings.html`

**Purpose.** Global platform configuration across general, SSL, backups,
rate limiting, and security.

**Key elements.**
- Tabs: General / SSL / TLS / Backups / Rate limiting / Security.
- *General* — platform name, base domain, default language (English /
  Nederlands per US-31 i18n).
- *SSL / TLS* — certificate mode (Let's Encrypt / custom / self-signed), ACME
  email, DNS challenge provider, custom cert upload. "ACME active" badge.
- *Backups* — nightly time (UTC), retention days, backup path, pre-install
  snapshot toggle.
- *Rate limiting* — installs/user/hour, installs/app/hour, login attempts/5
  min, API requests/min.
- *Security headers* — grid showing CSP, X-Frame-Options, HSTS,
  Referrer-Policy, CORS (all On) and CSRF (Off, with an "Enable CSRF"
  button).

**Maps to.** US-24 / FR-SSL-1 (SSL management), US-21 / FR-ADMIN-3 (backup
schedule), US-25 / FR-RT-2 (rate limiting), US-5/26/32 / NF-SEC (security
headers), US-31 / NF-A11Y (i18n).

---

## Navigation model

The sidebar groups routes into two sections that mirror the RBAC structure in
`roles/roles.yaml`:

```
General (user + admin)
  Dashboard          home.html
  App Store          apps.html
  My Apps            my-apps.html
  Containers         containers.html
  Launch             launch.html        (admin-only action)

Administration (admin-only)
  Users              admin-users.html
  Apps               admin-apps.html
  Secrets            admin-secrets.html
  Backups & Logs     admin-backups.html
  Settings           admin-settings.html
```

The active state is shown with the `--accent` color and a soft accent
background. The existing Angular app (`frontend/src/app/app.html`) currently
uses a top-nav bar; these mockups introduce a sidebar because the admin scope
(12+ pages) no longer fits comfortably in a horizontal nav. Switching the
Angular shell to a sidebar during implementation is recommended.

---

## Mapping to implementation

The mockups are intentionally framework-agnostic HTML/CSS so they can guide
Angular implementation without coupling to it. Suggested mapping:

| Mockup component | Angular target |
|------------------|----------------|
| Sidebar + topbar shell | `App` component (`app.html` / `app.scss`) — replace top-nav with sidebar |
| Page content | One standalone component per page under `frontend/src/app/pages/` |
| `.card`, `.badge`, `.btn`, `table.tbl` | Shared UI components under `frontend/src/app/shared/` |
| `styles.css` tokens | Migrate to `styles.scss` as SCSS variables / CSS custom properties |
| App store cards | `Apps` component + an `AppCard` presentational component |
| Install modal | New `InstallModal` component (not yet in routes) |
| Container dashboard | New `Containers` component (not yet in routes) |
| Admin sub-pages | Split `Admin` component into tabbed or routed sub-components |

Routes to add to `app.routes.ts` (currently only `''`, `login`, `apps`,
`admin` exist): `my-apps`, `containers`, `launch`, and admin children
(`admin/users`, `admin/apps`, `admin/secrets`, `admin/backups`,
`admin/settings`).

---

## Accessibility & i18n notes

The mockups are visual targets only; production markup must add:

- ARIA roles for the sidebar (`navigation`), tabs (`tablist`/`tab`/`tabpanel`),
  and modal (`dialog`, `aria-modal="true"`, focus trap).
- `aria-live="polite"` on the live log stream and SSE status badges.
- Visible focus styles (the design system uses `:focus` box-shadows on
  inputs; extend to all interactive elements).
- WCAG 2.1 AA color contrast — the current palette meets AA for body text on
  panel backgrounds; verify the `--text-dim` on `--bg` pair at implementation.
- Translation of all visible strings via `@ngx-translate` (en/nl already
  configured per US-31).

---

## Open design decisions

1. **Sidebar vs top-nav** — mockups use a sidebar; the current Angular shell
   uses a top-nav. A decision is needed before implementation.
2. **Icon set** — emoji are placeholders. Pick Material Icons, Lucide, or
   Heroicons and replace consistently.
3. **Real-time transport** — both SSE and WebSocket satisfy US-25. The
   mockups label the transport generically as "SSE"; confirm the choice.
4. **Admin page structure** — mockups use tabs within a single Admin page.
   Alternatively, admin can be a routed parent with child routes. Pick one
   before building.
5. **Empty/loading/error states** — not in scope for these mockups.
   Implementation should design skeletons, empty-state illustrations, and
   error banners consistent with this system.
