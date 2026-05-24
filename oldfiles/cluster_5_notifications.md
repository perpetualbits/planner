## Cluster 5 — Notifications & Attention

*Answers §21H (questions 25–27). Replaces §2 must-have #6 (notifications detail), and significantly reshapes §4 (Today tab description) and §17 (smart reminders algorithm).*

### 1. Posture

The system's notifications are its voice in the household's day. The default volume is near-silent.

This is the cluster where the design principle from earlier clusters — *the system holds, displays, and gets out of the way* — becomes the concrete moment-to-moment experience. Every notification fires at a cost: it interrupts, it teaches the household whether to trust the channel, and it shapes what people pay attention to. Most of the time the cost exceeds the benefit. When in doubt, the system does not speak.

The asymmetry guiding all defaults: under-notifying produces a concrete recoverable cost (a missed bin, a forgotten errand). Over-notifying produces an unrecoverable cost (the family stops looking at the hub). The cost of the second is existential to the project. So the system errs toward silence.

### 2. Three notification levels

Three, not more. Configuration grows with levels; fewer levels means the system has thought clearly.

**Critical.** Things with real-world consequence if missed, recoverable only by acting now. Bin truck in 12 hours and the bin is still in the kitchen. Medication for a household member who depends on it. Departure-window-now items where missing it means missing the thing entirely.

- Channels: push to relevant members' phones; single audible chime on hub; pulsing amber LED.
- Pierces quiet hours.
- Frequency target: ~3 per week in a typical household, often fewer. A household firing Critical alerts daily has miscalibrated its priority levels and the system should make this visible (see §6 below).

**Today.** Things that matter today, that the family already implicitly knew about. Kid's dentist 14:30, school pickup, dinner timing.

- Channels: visible in the hub *Today* view. No push to phones by default. No chime.
- Per-item opt-in to phone push exists, for items the user wants individually elevated.
- Does not fire during quiet hours — appears silently in the hub the next morning.

**Soft.** Everything else. Thread "been a while" prompts, project milestones still weeks out, anchor pull-views, low-cadence reminders.

- Channels: visible only when the user opens the relevant view.
- No alert, no chime, no LED change.
- Present, not announced.

### 3. Quiet hours and presence

**Quiet hours.** Default 22:00–07:00 per member, configurable. Today and Soft simply do not send during these windows; Critical pierces. No "morning digest" — the *Today* view is the digest.

**Presence-aware silencing.** Members in `away`, `offshore`, or `with_other_parent` Presence states receive only items that name them directly. They don't get general household nudges (bins, family dinner) because they're not present and those items aren't theirs. This is automatic, not a mode the user toggles.

**Per-item-type push preference (mobile).** The mobile app lets each member opt their phone in to push for specific item types or specific tags. Some users want phone-push for `appointment`, none for anything else. Some want everything in the hub and nothing on the phone. The defaults are conservative; the user can elevate.

### 4. Repeat, not escalate

The system does not automatically pass an unacknowledged item to a second person. Escalation chains imply the system has a stake in compliance; it doesn't. Pinging a backup reframes the original assignee's autonomy as something to be policed, which is the opposite of what this system is for.

What the system does instead:

- **Repeat once, to the same person.** A Critical item that isn't acknowledged after 4 hours repeats — same recipient, same channel, single repeat. After that, no further automatic action.
- **Manual reassign in one tap.** If the assignee can't do it, they hand off (Cluster 2 mechanism). The new assignee receives the alert. The handoff is the human escalating, not the system.
- **Opt-in escalation per-item.** A specific item — most commonly medication for a dependent household member — can be configured with `escalatesTo: [memberId]` set explicitly. The system will then notify the backup if the primary doesn't acknowledge within the configured window. This is opt-in per item, rare in practice, and the household consciously chooses it.

### 5. Hub at rest

The hub is a physical object in the kitchen. It is visible to everyone in the room, all day. Its baseline state shapes what people stop noticing.

**Permanent fixtures (always visible):**

- Day and time, large.
- Weather, one short line ("8°, rain by afternoon").
- A small status patch: "3 things today" or "nothing today". Tapping opens *Today*.
- Ambient LED status (see §6 below).

**That is the entire baseline.** The full *Today* view is one tap away. The hub at rest looks more like a calm clock than a dashboard.

The reasoning: a hub showing eight things permanently *teaches* the household to tune it out, because nothing on it is ever new. A hub showing two or three things permanently *trains* the household to notice when something changes. Constancy is the baseline; only changes catch the eye.

**Empty state is a designed state.** When nothing demands attention, the status patch reads "nothing today" — in plain, calm language. Not "all clear!" or "you're caught up! 🎉". Just nothing today. This is the visible counterpart to the surfacing layer's empty state from Cluster 1.

### 6. Ambient LED

A glanceable colour cue, visible from across the room:

- **Off** or very dim white: hub at rest, nothing pending.
- **Soft warm white**: Today items present, no urgency.
- **Pulsing amber**: a Critical alert is currently active and unacknowledged.

No red, ever. Red is the language of alarms; this system is not an alarm. No green "all good" either — the absence of a cue is its own message and doesn't need positive reinforcement.

### 7. Self-calibration

A small reflective feature, post-MVP: if Critical alerts are firing more than ~5 times per week consistently, the hub surfaces a quiet note: "You're getting frequent critical alerts. Some items may be miscategorised — want to review?" It does not block, does not nag. Once dismissed, it doesn't return for a month.

The purpose: notification noise compounds over time, often through accumulated miscategorisation rather than any single mistake. A periodic invitation to look at the categorisation distribution is a gentle correction mechanism that doesn't require the user to audit themselves on their own.

### 8. What changed from §2, §4, §17

- Replaced: §2's "Notifications: push to phones; on‑hub chimes; optional LED light cue" — refined into the three-level model with clear defaults.
- Replaced: §17's "Smart reminders" with situational algorithm. The notification trigger is just `dueAt` or a per-item rule, not an inferred "free window" calculation.
- Reshaped: §4's Today-tab description. The Today *view* still exists and is rich; the hub *at rest* is minimal.
- Added: ambient LED states defined precisely. Quiet hours defaulting and behaviour. Presence-aware silencing. Self-calibration affordance (post-MVP).
- Removed: automatic escalation chains. The escalation feature exists per-item as opt-in only.
- Removed: red as a colour state anywhere in the system.
