# Behavioral Nudge Engine — Analysis & Design

> Status: **Analysis** · Owner: UX/Platform · Last updated: 2026-08-10
>
> This document analyzes the Server App Manager (SAM) platform through the lens of
> behavioral science and proposes a **Behavioral Nudge Engine** (BNE): a lightweight,
> policy-driven subsystem that steers administrators and end-users toward safer,
> healthier, and more efficient operations without removing their freedom of choice.
>
> A nudge is *any aspect of the choice architecture that alters people's behavior in a
> predictable way without forbidding any options or significantly changing their
> economic incentives* (Thaler & Sunstein, 2008). The BNE makes those interventions
> explicit, measurable, and tunable.

---

## 1. Why a Nudge Engine for SAM

SAM is operated by technical users on their own hardware. The cost of a bad choice is
disproportionate to the effort of making it: a skipped backup, an unrotated secret, an
ignored unhealthy container, or a permissive role assignment can each lead to data loss
or a breach. Traditional enforcement (hard blocks, mandatory fields) is already present
for the most critical paths; what is missing is **soft steering** for the large surface
of decisions that are technically optional but operationally risky.

The BNE targets four behavioral gaps observed across the product requirements
(`docs/PRD.md`) and user stories (`docs/USER_STORIES.md`):

| Gap | Symptom | Affected requirements |
|-----|---------|------------------------|
| **Inattention** | Unhealthy containers linger; backups drift past retention | FR-CL-2, FR-ADMIN-3 |
| **Present bias** | Users install apps now, configure backups "later" (never) | FR-APP-1, FR-ADMIN-3 |
| **Status quo bias** | Default roles, default secrets, default ports never revisited | FR-AUTH-2, FR-ADMIN-2 |
| **Overconfidence** | Destructive actions (uninstall, delete app) confirmed without context | FR-UNI-1, FR-UNI-2, FR-ADMIN-1 |

A nudge engine closes these gaps with low-friction, evidence-based interventions layered
on top of the existing UI shell (`designs/styles.css`) and Angular routes.

---

## 2. Behavioral Foundations

The engine draws on a compact set of validated behavioral principles. Each is mapped to
the SAM context so interventions stay grounded rather than ornamental.

### 2.1 EAST Framework (Behavioural Insights Team)

> **E**asy, **A**ttractive, **S**ocial, **T**imely.

- **Easy** — reduce friction for the desired action; increase it marginally for risky
  ones. SAM example: one-click "Back up now" from the dashboard; typed confirmation
  string for `Delete app`.
- **Attractive** — draw the eye to the right thing at the right time. SAM example:
  unhealthy rows already turn red (`table.tbl .unhealthy`); the BNE extends this with
  attention-grabbing but non-blocking banners.
- **Social** — surface what peers/defaults do. SAM example: "12 of 12 admins rotate
  secrets within 30 days" on the secrets panel.
- **Timely** — deliver the nudge at the moment of decision, not in a buried settings
  page. SAM example: prompt backup configuration *immediately after* a successful
  install, not on a later admin visit.

### 2.2 Choice Architecture Levers

| Lever | Definition | SAM application |
|-------|------------|-----------------|
| **Defaults** | Pre-selected option adopted unless changed | Secure defaults for new apps, roles, retention |
| **Framing** | Equivalent info presented to emphasize gains/losses | Loss-framed backup reminders ("You'd lose 12 apps' data") |
| **Salience** | Make the relevant cue visually dominant | Color, motion, placement per `--warn`/`--bad` tokens |
| **Social proof** | Show what others do | Adoption counts, "most admins…", install counts (already on app cards) |
| **Commitment** | Elicit a small pledge now to align future behavior | "I'll configure backups within 24h" checkbox post-install |
| **Friction** | Add/remove steps to shift behavior | Typed delete confirmation; one-click healthy re-check |
| **Feedback** | Close the loop on an action's effect | Post-action toast confirming the nudge's predicted outcome |

### 2.3 Ethical Guardrails

Nudges can manipulate. The BNE adheres to four constraints, auditable via the same audit
log used for admin actions (FR-ADMIN-3):

1. **Transparency** — every nudge is logged with its trigger, target user, and variant.
   Users can open a "Why am I seeing this?" affordance on any nudge.
2. **Opt-out** — a per-user "Reduce suggestions" preference disables non-safety nudges.
   Safety-critical nudges (unhealthy container, expired backup) remain but can be
   snoozed for a bounded period.
3. **No dark patterns** — nudges never hide cheaper/safer options, never use fake
   urgency ("only 1 left!"), and never block a legitimate action the user is authorized
   to perform.
4. **Proportionality** — nudge intensity scales with risk, not with engagement metrics.
   A misconfigured nudge cannot become a nag.

---

## 3. Nudge Inventory

Each nudge is specified with: **trigger** (event that fires it), **audience** (role +
state), **mechanism** (UI pattern), **lever** (§2.2), **expected effect**, and
**measurement**. IDs use the `N-<domain>-<n>` convention for traceability into the audit
log and future A/B framework.

### 3.1 Security domain

#### N-SEC-1 · Secret rotation reminder
- **Trigger** — a Docker secret's `last_rotated_at` exceeds the policy threshold
  (default 90 days) or is null.
- **Audience** — `admin` users on `admin-secrets.html`.
- **Mechanism** — an `--warn` banner above the secrets table listing expiring keys with
  a "Rotate selected" button that pre-fills the rotation modal.
- **Lever** — salience + timely + friction (reduces rotation from modal-hunt to one
  click).
- **Expected effect** — median secret age drops; rotation events cluster within 7 days
  of the reminder.
- **Measurement** — `secret_age_days` p50/p90; rotation events per admin per 30 days.

#### N-SEC-2 · Strong-defaults on new role assignment
- **Trigger** — admin opens the "Add user" / "Edit role" dialog
  (`admin-users.html`).
- **Mechanism** — the role `<select>` defaults to `user` (least privilege), and the
  helper text shows the permission delta vs. `admin` in plain language ("Grants
  read:apps, write:containers — no user or settings management").
- **Lever** — defaults + framing.
- **Expected effect** — fewer accidental `admin` grants; reviewable in audit log.
- **Measurement** — share of new users created with `admin` role; revert events within
  24h.

#### N-SEC-3 · Loss-framed backup gap warning
- **Trigger** — no successful backup in `retention_interval * 1.5` OR a backup failure
  logged.
- **Audience** — `admin` on dashboard and `admin-backups.html`.
- **Mechanism** — dashboard "Last Backup" stat tile flips from `--ok` to `--warn`/`--bad`
  and the delta line reads *"No backup in 6h — 12 apps at risk"* instead of the neutral
  timestamp. A "Back up now" button is rendered inline.
- **Lever** — framing (loss) + salience + timely + friction.
- **Expected effect** — time-to-next-backup after a gap shrinks materially.
- **Measurement** — `hours_since_last_good_backup` p90; manual backup trigger rate
  within 1h of warning.

### 3.2 Reliability domain

#### N-REL-1 · Unhealthy container triage nudge
- **Trigger** — a container's health check fails (FR-CL-2 SSE event).
- **Audience** — any user with `read:containers`; admins get an additional action.
- **Mechanism** — dashboard "Unhealthy" stat tile links directly to a filtered
  `containers.html?health=unhealthy` view; for admins, a "Restart + recheck" button
  appears next to the row without leaving the dashboard.
- **Lever** — salience + friction + timely.
- **Expected effect** — mean time to remediate an unhealthy container drops.
- **Measurement** — `mttr_unhealthy_minutes` p50/p90; restarts issued from dashboard vs.
  containers page.

#### N-REL-2 · Dependency-aware uninstall confirmation
- **Trigger** — user clicks Uninstall on an app with running dependents (FR-UNI-2).
- **Mechanism** — the confirmation modal lists dependent containers *with their health
  state* and requires a typed confirmation of the app slug (not a generic "Are you
  sure?"). The dependent list is the salient element; the type-to-confirm is the
  friction.
- **Lever** — salience + friction + framing (loss of dependents).
- **Expected effect** — accidental dependency-breaking uninstalls approach zero.
- **Measurement** — uninstalls aborted at confirmation; dependents left orphaned
  (should be 0).

#### N-REL-3 · Resource-limit suggestion on launch
- **Trigger** — admin launches a child container (FR-CL-1) without `mem_limit`/`cpu_quota`.
- **Mechanism** — the launch form shows a non-blocking `--info` hint with the host's
  current free memory and a suggested limit band; the field is pre-filled with the
  suggestion but editable.
- **Lever** — defaults + social proof (host state) + friction.
- **Expected effect** — fewer OOM-killed containers; more predictable neighbor
  performance.
- **Measurement** — share of launched containers with explicit limits; OOM events per
  100 container-hours.

### 3.3 Adoption domain

#### N-ADOPT-1 · Post-install backup commitment
- **Trigger** — a successful app install (FR-APP-1 success event).
- **Mechanism** — the post-install summary card (already specified in FR-APP-3) gains an
  optional checkbox: *"Remind me to configure backups for {{app}} in 24h."* If checked,
  a single timed nudge fires; if ignored, no nag.
- **Lever** — commitment + timely.
- **Expected effect** — backup coverage for newly installed apps rises.
- **Measurement** — % of installed apps with a backup within 7 days of install.

#### N-ADOPT-2 · App store social-proof refinement
- **Trigger** — rendering of an `.app-card` on `apps.html`.
- **Mechanism** — the existing install count is augmented with a contextual line for
  apps the user's tenant hasn't installed: *"Popular in Media — installed by 8 of 10
  tenants."* (In single-tenant MVP, fall back to global install count + "Recently
  added" badge.)
- **Lever** — social proof.
- **Expected effect** — discovery of high-quality apps improves; install regret (uninstall
  within 24h) does not increase.
- **Measurement** — install-through-rate per card; 24h uninstall rate (guardrail).

#### N-ADOPT-3 · Stale app definition update nudge
- **Trigger** — an enabled app's YAML `version` is older than the store version.
- **Audience** — `admin` on `admin-apps.html`.
- **Mechanism** — a row-level `--info` badge "Update available" with a one-click "Diff &
  upgrade" action that opens the YAML editor pre-loaded with the new version and a
  highlighted diff.
- **Lever** — salience + friction + timely.
- **Expected effect** — mean app-definition age stays close to store age.
- **Measurement** — `app_definition_age_days` p50; upgrade actions per week.

### 3.4 Hygiene domain

#### N-HYG-1 · Empty-state onboarding nudge
- **Trigger** — a fresh tenant with zero installed apps opens the dashboard.
- **Mechanism** — the "My Apps" preview is replaced with a guided empty state: three
  suggested starter apps (e.g., a media app, a productivity app, a dev tool) with
  one-click install and a "Set up nightly backups" secondary CTA.
- **Lever** — defaults + social proof + friction.
- **Expected effect** — time-to-first-value drops; first backup configured earlier.
- **Measurement** — time-to-first-install; time-to-first-backup-config.

#### N-HYG-2 · Audit-log anomaly highlight
- **Trigger** — the recent-activity feed contains a high-risk event (role change,
  secret rotation, app deletion, failed login burst).
- **Mechanism** — the dashboard activity row renders with a `--warn`/`--bad` left border
  and a "Review" link to the filtered audit view.
- **Lever** — salience + timely.
- **Expected effect** — admins review anomalous events sooner.
- **Measurement** — time-from-event-to-review for flagged vs. unflagged events.

---

## 4. Engine Architecture

The BNE is intentionally minimal and reuses existing SAM infrastructure. It is **not** a
separate service; it is a backend policy module plus a frontend directive.

### 4.1 Components

```
┌──────────────────────────────────────────────────────────────┐
│  Backend (Rust / axum)                                        │
│                                                               │
│  ┌───────────────┐   events   ┌──────────────────────────┐   │
│  │ Domain modules │ ─────────▶│ NudgePolicy (rules)      │   │
│  │ (apps, cont.,  │            │  - evaluates triggers    │   │
│  │  secrets, ...) │            │  - selects variant       │   │
│  └───────────────┘            └──────────┬───────────────┘   │
│                                          │ nudge DTO          │
│                                          ▼                    │
│                               ┌──────────────────────────┐   │
│                               │ NudgeStore (Postgres)    │   │
│                               │  - state (snooze, seen)  │   │
│                               │  - audit rows            │   │
│                               └──────────┬───────────────┘   │
│                                          │ SSE push / REST GET │
└──────────────────────────────────────────┼────────────────────┘
                                           │
┌──────────────────────────────────────────┼────────────────────┐
│  Frontend (Angular)                       ▼                    │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ NudgeService (SSE consumer + REST fallback)            │   │
│  │  - dedupe, snooze, opt-out                             │   │
│  └───────────┬────────────────────────────────────────────┘   │
│              │ nudge payload                                   │
│              ▼                                                 │
│  ┌────────────────────────────────────────────────────────┐   │
│  │ <sam-nudge> component (banner / inline / toast)        │   │
│  │  - renders via design-system tokens (styles.css)       │   │
│  │  - "Why am I seeing this?" + snooze + dismiss          │   │
│  └────────────────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────────────────┘
```

### 4.2 Nudge lifecycle

1. **Trigger** — a domain event (install success, health failure, secret age tick) is
   emitted on the existing event bus.
2. **Evaluate** — `NudgePolicy` matches the event against rules, checks `NudgeStore`
   for snooze/opt-out/already-seen state, and selects a variant.
3. **Deliver** — the nudge DTO is pushed over the same SSE channel used for live install
   status (FR-RT-1) and/or fetched on page load.
4. **Render** — `<sam-nudge>` renders in one of three slots: dashboard banner, inline
   next to the related entity, or transient toast.
5. **Resolve** — the user acts, snoozes, or dismisses; the outcome is written to
   `NudgeStore` and the audit log.
6. **Measure** — an offline job rolls up per-nudge metrics (see §6).

### 4.3 Data model (sketch)

```sql
-- One row per nudge instance
CREATE TABLE nudges (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id       UUID NOT NULL REFERENCES users(id),
  tenant_id     UUID NOT NULL,
  nudge_id      TEXT NOT NULL,          -- e.g. 'N-SEC-1'
  variant       TEXT NOT NULL,          -- A/B variant label
  trigger_event JSONB NOT NULL,         -- the event that fired it
  state         TEXT NOT NULL,          -- pending|shown|acted|snoozed|dismissed
  shown_at      TIMESTAMPTZ,
  acted_at      TIMESTAMPTZ,
  snoozed_until TIMESTAMPTZ,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Per-user opt-out / preference
CREATE TABLE nudge_prefs (
  user_id       UUID PRIMARY KEY REFERENCES users(id),
  reduce_suggestions BOOLEAN NOT NULL DEFAULT false,
  safety_snooze_max_hours INT NOT NULL DEFAULT 72
);
```

### 4.4 API surface (additive to existing OpenAPI spec)

| Method | Path | Role | Purpose |
|--------|------|------|---------|
| `GET`  | `/api/v1/nudges` | any auth | active nudges for the current user |
| `POST` | `/api/v1/nudges/{id}/act` | any auth | record an action outcome |
| `POST` | `/api/v1/nudges/{id}/snooze` | any auth | snooze with bounded duration |
| `POST` | `/api/v1/nudges/{id}/dismiss` | any auth | dismiss (logged) |
| `GET`/`PATCH` | `/api/v1/nudges/prefs` | any auth | read/update opt-out prefs |

All endpoints are JWT + tenant-scoped and audited like other admin actions.

---

## 5. Mapping to Existing UI

The BNE reuses the design system in `designs/styles.css`. No new visual language is
introduced.

| Nudge | Page | Existing element extended |
|-------|------|----------------------------|
| N-SEC-1 | `admin-secrets.html` | new `--warn` banner above secrets table |
| N-SEC-2 | `admin-users.html` | role `<select>` default + helper text |
| N-SEC-3 | `home.html` | "Last Backup" `.stat` tile delta line + inline button |
| N-REL-1 | `home.html` + `containers.html` | "Unhealthy" `.stat` tile → deep link; row action |
| N-REL-2 | `my-apps.html` | uninstall `.modal` body (dependent list + typed confirm) |
| N-REL-3 | `launch.html` | `.field` hint + pre-filled `mem_limit`/`cpu_quota` |
| N-ADOPT-1 | `install-modal.html` | post-install summary card checkbox |
| N-ADOPT-2 | `apps.html` | `.app-card` footer social-proof line |
| N-ADOPT-3 | `admin-apps.html` | row-level `--info` badge + diff action |
| N-HYG-1 | `home.html` | "My Apps" preview empty state |
| N-HYG-2 | `home.html` | recent-activity row left border + "Review" link |

---

## 6. Measurement & Evaluation

Each nudge carries a hypothesis and a guardrail. Evaluation is observational first
(before/after on the same tenant), with optional A/B once traffic justifies it.

### 6.1 Primary metrics

| Domain | Metric | Source |
|--------|--------|--------|
| Security | `secret_age_days` p50/p90 | `secrets` table |
| Security | new users granted `admin` (%) | audit log |
| Reliability | `mttr_unhealthy_minutes` p50/p90 | health events |
| Reliability | orphaned dependents post-uninstall | `containers` scan |
| Reliability | OOM events / 100 container-hrs | Docker events |
| Adoption | time-to-first-install (new tenant) | install events |
| Adoption | time-to-first-backup-config | backup settings |
| Hygiene | time-from-event-to-review (flagged) | audit view events |

### 6.2 Guardrails (do no harm)

- **24h uninstall rate** must not rise with N-ADOPT-2 (social proof must not push
  regret installs).
- **Dismissal rate** per nudge > 60% over 30 days triggers a policy review (the nudge is
  likely noise or mistimed).
- **Action rate** per safety nudge < 20% triggers a salience/friction redesign, not a
  stronger block.
- **Audit log volume** from nudge events kept < 5% of total to avoid drowning signal.

---

## 7. Rollout Plan

1. **Phase 0 — Instrumentation (P1, MVP+).** Add `nudges` + `nudge_prefs` tables, SSE
   channel, and `<sam-nudge>` shell with no rules. Ship N-SEC-3 and N-REL-1 only (highest
   value, lowest risk).
2. **Phase 1 — Security & reliability set.** Add N-SEC-1, N-SEC-2, N-REL-2, N-REL-3.
   Enable per-user opt-out and "Why am I seeing this?".
3. **Phase 2 — Adoption & hygiene.** Add N-ADOPT-1/2/3 and N-HYG-1/2 once measurement
   baseline from Phase 0/1 is established.
4. **Phase 3 — Experimentation.** Introduce A/B variants where action rate is marginal;
   gate on the §6.2 guardrails.

Each phase is independently shippable and reversible (rules are data, not code paths).

---

## 8. Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Nudge fatigue / banner blindness | Per-user rate cap (max N active nudges), dismiss/snooze, dismiss-rate guardrail |
| Perceived manipulation | Transparency affordance, opt-out, audit logging, no fake urgency |
| False-positive triggers erode trust | Every trigger event is reviewable; policy rules are editable without redeploy |
| Performance cost of SSE nudge channel | Reuse existing FR-RT-1 SSE; nudges are low-frequency |
| Scope creep into hard enforcement | BNE never blocks authorized actions; hard blocks stay in RBAC/domain logic |

---

## 9. Open Questions

1. Should safety nudges (N-SEC-3, N-REL-1) be snoozable indefinitely by admins, or capped
   at `safety_snooze_max_hours`? Current proposal: capped, per `nudge_prefs`.
2. In single-tenant MVP, what is the social-proof source for N-ADOPT-2? Proposal: global
   install counts + "Recently added" until multi-tenant data exists.
3. Should nudge outcomes feed back into app-definition defaults (e.g., auto-suggest the
   `mem_limit` most often accepted in N-REL-3)? Deferred to Phase 3.

---

## 10. References

- Thaler, R. & Sunstein, C. (2008). *Nudge: Improving Decisions About Health, Wealth, and
  Happiness*.
- Behavioural Insights Team (2012). *EAST: Four simple ways to apply behavioural
  insights*.
- Kahneman, D. (2011). *Thinking, Fast and Slow* (System 1/System 2 framing).
- SAM product context: `docs/PRD.md`, `docs/USER_STORIES.md`, `docs/ARCHITECTURE.md`,
  `designs/DESIGN.md`.
