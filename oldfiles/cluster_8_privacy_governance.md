## Cluster 8 — Privacy, Governance, Hardware, Internationalization

*Answers §21K, L, N, O (questions 34–44). Replaces parts of §9 (Privacy, Safety & Roles), §11 (Hardware & Install), §12 (Tech Stack) parts about auth and storage. Also articulates the project's long-term direction as a stated commitment.*

### 1. Posture

Most of this cluster is the operational expression of principles established earlier. Where Clusters 4, 6, and 7 set commitments ("no admin reveal of children's private items", "kitchen is a safe place", "no data flow to third parties"), this cluster turns those into specific, actionable policies — and articulates the larger trajectory the system is heading toward.

### 2. Privacy and right to be forgotten

**Data locality.** The household's data lives on the household's own hardware. Optional cloud sync exists only as opt-in with end-to-end encryption; even then, the cloud party holds only ciphertext. No third-party data flow, no analytics telemetry that exposes user behaviour, no advertising integration, ever. These are categorical project commitments per Cluster 6 — not configurable settings.

**Member deletion.** A household admin can delete any member at any time. Deletion behaviour:

- Removes the FamilyMember record, all their owned tasks, their private items, their completion log entries, their DietaryProfile, their Anchor list, and their Threads.
- Does *not* automatically delete shared items where they were one of several `assigneeId`s, to preserve the household's historical record. The deleted member's name is replaced with a tombstone label ("removed member") in such records.
- Offers a complete data export as JSON/CSV before deletion, with an unambiguous "you cannot undo this" warning.
- Applies equally to all members, including children. Children's data is no harder and no easier to delete than anyone else's — special-casing GDPR-protected categories down a worse path is itself a form of discrimination.

**Audit log retention.** The system keeps a minimal admin log (member added/removed, visibility setting changes, deletion events, integration changes) for 90 days, then deletes. This is for forensic clarity in cases of disputed actions, not for surveillance. The log is itself privacy-sensitive and visible only to admins.

### 3. Abuse prevention

The categorical commitments from earlier clusters do most of this work, but explicit statements close remaining doors:

**No monitoring features.** The system shall not implement features whose purpose is for one household member to track another's compliance, location, activity, communications, or device usage. This is the affirmative counterpart to "no admin reveal" — not just "we don't let admins override privacy", but "we don't build the tools that an abusive party would want even if technically permitted".

**Admin power is content-asymmetric in only one direction.** Admins manage settings and structure; they have *no extra access* to other members' private content. There is no "see all" mode, "admin override view", or "household summary across privacy boundaries". This is enforced architecturally, not just by policy.

**Help-finding affordance.** A small menu item in every member's settings, titled "If you need help" or equivalent, links to local domestic-violence and family-support resources (NL and EN). The links are constant — never personalised, never tracked — and accessing them generates no audit trail entry. This is one of the few places in the system where a feature exists specifically for a rare, serious case.

### 4. Governance — two-tier model

The household has exactly two governance roles. Heavier permissions models replicate corporate-IT patterns that are wrong for a home.

**Household admin.** At least one adult member, at any one time, must be admin. Multiple admins permitted. Admin powers:

- Create, invite, and delete members.
- Configure household-level settings: quiet hours defaults, language defaults, holiday calendars, integration setup.
- Manage the hub: Wi-Fi, NFC bindings, paired devices.
- Export full household data.

Admins **cannot**: read others' private items, override visibility settings, impersonate other members, or modify content owned by others (beyond what their own membership would normally permit).

**Everyone else (members).** All adult, teen, and age-appropriate child members. Members can:

- Create, edit, and complete their own items (tasks, events, notes, anchors).
- Participate in everything visible to them (shared meals, family events, household chores).
- Manage their own preferences: language, quiet hours, push notification settings, privacy defaults.
- Set per-item visibility on items they own.

PIN/passphrase resets are self-service via the member's own recovery method (a known second device, a recovery passphrase set at onboarding). Admin-assisted reset exists as a fallback only — admin resets PIN to a temporary value, member chooses a new one on first use.

### 5. Change log and conflict transparency

The system does not arbitrate negotiations between household members (§C of the brief). It does keep a factual change log — visible to all family members, not just admins — showing who reassigned what, who marked items complete, who modified shared content. This is for shared transparency, not for blame.

Retention: 90 days rolling. No "conflict resolution" feature, no "blame" view, no notification when someone else makes a change to a shared item (that would be a monitoring feature in disguise).

### 6. House split and merge

Lives change shape. The system does not get in the way.

**Split.** A member departing the household (separation, custody change, moving out) can export all of their own data plus a copy of jointly-owned data they choose to take. They can then spin up a new household instance using the export. The original household instance continues with the remaining members.

**Merge.** New partner, blended family, or roommate joining: standard "add member" flow plus optional import of their existing exported data. No special "merge households" workflow; the data is treated as theirs to bring in.

There is no "support divorce" feature. The architecture just doesn't trap data, so when lives reorganise, the system doesn't become an obstacle.

### 7. Hardware

**Form factor.** The hub is the first form of a household-owned home node (see §10 below). Two viable physical realisations:

- **All-in-one touchscreen computer.** A single device with sufficient compute, storage, and a kitchen-suitable display. Many commercial all-in-ones (used in retail, hospitality) are suitable. Simpler to install: one device, one power cable, one wall mount.
- **Separate compute + display.** A small home server (Raspberry Pi 5 or similar small-board computer with NVMe storage; or a mini-PC) running in a closet or utility area, with a touch display in the kitchen acting as the primary client over local network. Allows quieter thermals, easier hardware refresh, multiple displays.

The architecture supports either: storage and services run on the compute device, frontends (touch display, mobile apps, browsers) talk to it over local network. For MVP, an Android tablet in kiosk mode is the lowest-friction entry point for a first family; the home-node form factors are intended for the project's growth phase.

**Storage.** 1TB minimum on the home node. The planner's own data fits in tens of MB; the storage capacity is for photo attachments (homework, receipts), forwarded email contents, eventual voice transcript history if the household opts to retain it, and — most importantly — the user's own files chosen to be held on their own infrastructure rather than in commercial cloud silos. The planner is the first application; the home node is the platform.

**Power and reliability.** A home node is always-on. Reasonable expectations: fanless or quiet thermals, low power draw (10–30W typical), small UPS or battery backup for graceful handling of brief outages. PoE is supported on the Raspberry Pi route. A regular all-in-one touchscreen uses standard AC power.

**Display.** Always-on with daylight dimming. The hub at-rest display (Cluster 5) is also the screensaver — a calm clock with weather is what the device shows when nothing else needs attention.

### 8. Authentication at the hub

Already established in Cluster 6, restated for completeness:

- **NFC tap** (each member has a card or sticker). Fastest member switch.
- **PIN / emoji code** (short, memorable; children get an emoji sequence).
- **No facial recognition, no biometric capture by the system, no presence-detection-based auto-switching.** All ruled out by the Cluster 6 categorical commitment.
- **Session timeout** after 5 minutes idle, returning to shared *Today* view (household-visible items only).

### 9. Internationalization

**Languages at launch: EN and NL.** Per-member preference (mixed-language households are common). Date formats follow Dutch convention by default (day-first, 24-hour); per-member override possible.

**NL public holiday calendar** is loaded at install from a maintained public source and refreshed annually. Drives the holiday-skip behaviour for chores tagged `household` or `chore`.

**Regional school holiday schedules.** NL has Noord, Midden, Zuid regional school holiday schedules. The household configures region per child during onboarding; the system pulls a public schedule for that region if the school doesn't provide its own ICS.

**Municipal afvalkalender.** Already in the calendar source list (Cluster 1). Configured per household by entering postcode and house number; the system pulls the appropriate calendar.

The system stores values in locale-neutral form (ISO datetimes, units with explicit denominations) and renders to the user's preferred language at display time. No user-supplied data is locked to one language at storage time.

### 10. Long-term direction

This belongs near the front of the consolidated brief as a stated commitment:

> *The hub is the first form of a household-owned home node. The MVP runs on a tablet for accessibility; the architecture is designed to migrate to a small home server (compute + storage + always-on) without rewrite. All-in-one touchscreen computers can serve as both server and primary display; the architecture also supports a separate compute device with a touch display as one client among many.*
>
> *Long-term, the home node becomes the substrate for additional household-owned services, with the family planner as its first application. The project's ultimate trajectory is post-cloud household computing: data the user owns, on hardware the user controls, with no commercial party in the middle.*
>
> *Distributed peer-to-peer backup via networks like I2P or Tahoe-LAFS — encrypted, erasure-coded, hosted on the hardware of trusted peers — is one explicit candidate for that trajectory. The technology is mature; the social and bootstrap challenges are real. Not for MVP, but the architecture today should not foreclose it tomorrow.*

This is a *direction*, not a roadmap. It explains, to future contributors and to the project's author in years to come, why certain decisions in MVP are made the way they are: the categorical privacy commitments, the rejection of cloud integrations, the home-node storage sizing, the insistence on local-first data — all are early forms of the trajectory above.

### 11. What changed from §9, §11, §12

- §9 (Privacy, Safety & Roles): expanded into precise policies — member deletion behaviour, audit log retention, abuse-prevention features. The role gradient (Owner / Adult / Teen / Child / Guest) is replaced by the simpler two-tier admin / member model, with per-item visibility (set by owner) doing the work that role-based visibility used to do.
- §11 (Hardware & Install): refined into the home-node concept. Tablet for MVP, home-node forms (all-in-one or separate compute+display) for project maturity. Storage sizing rationale articulated.
- §12 (auth and storage parts): no biometric or presence-based authentication. Storage architecture commits to local-first with future-ready service/frontend separation.
- New: §3 (abuse prevention as explicit), §5 (change log scope and visibility), §6 (split/merge support via export/import), §10 (long-term direction).
