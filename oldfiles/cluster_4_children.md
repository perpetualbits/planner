## Cluster 4 — Children & Homework

*Answers §21G (questions 22–24). Replaces the homework-related parts of §2 ("Nice-to-have: Simple homework tracking…"), the Homework entity from §3, and the Kid Mode description in §16.*

### 1. Posture

Children are full users of the system to the extent their age allows, not objects managed through it. The system is *partly* for them and *partly* for the household — and where those interests diverge, the design defaults to protecting the child's emerging autonomy, not to maximising parental visibility.

The temptation in this cluster is to build the system the *parents* want. Resisted explicitly.

**Things this system will not do for children:**

- Automated parent reports on homework completion, screen time, or activity.
- Gamification — no streaks, no badges, no confetti, no "well done!" animations.
- Reading logs, screen-free reminders, or other school-style compliance instruments.
- OCR-based homework capture (too unreliable to be trusted with a child's grade).
- "Smart" suggestions about how a child should spend their time.

**Things this system will do:**

- Hold a child's commitments (homework, club practice, things to bring on Friday) in the same model as everyone else's.
- Show each child a calm, age-appropriate view of *their* day.
- Provide kid-invoked focus timers for homework.
- Honour per-item privacy when the child is old enough to set it.
- Stay out of the parent–child relationship except where the household has explicitly invited it.

### 2. Homework as a Task

There is no dedicated `Homework` entity. Homework is a `Task` with conventional tags. This keeps the model uniform and avoids a feature surface that grows into school-app territory.

Conventions:

- `tag: homework`, optionally `tag: <subject>` (`maths`, `dutch`, `history`).
- `ownerId` = the child.
- `dueBy` = the school's deadline.
- `attachments[]` may contain a photo of the assignment sheet (manual capture; not OCR'd).
- `notes` may contain free-text detail.
- An optional `requiresAck: 'parent' | 'none'` field, defaulting to `none`. The child sets this when they want a parent to confirm before submission. The parent does not set it; only the owner does.

This is enough to track homework without building a homework feature.

### 3. Homework capture

**Manual only.** Typed at the hub, mobile, or via voice ("Sam, add maths worksheet due Wednesday, 30 minutes").

**Photo as attachment, not as input.** If a child or parent wants to retain the original assignment sheet, the photo attaches to the manually-entered Task. The system does not OCR it, does not infer a due date from it, does not parse subject or content. The photo is a reference image only.

Rationale: OCR of homework demos beautifully and works often enough to be relied on, then fails in ways that erode trust — a misread due date, an undetected weekend, a child who *thought* the assignment was logged correctly and discovered too late it wasn't. The system would share blame for an outcome it cannot guarantee. Manual entry takes fifteen seconds and never lies.

### 4. Parent involvement features

**Focus timers.** A child can start a 15/20/25-minute focus timer from the homework Task. The timer shows on the hub or mobile, counts down, and stops. That's the whole feature. Specifically:

- The child invokes it. The system does not suggest it.
- The timer's outcome is not logged anywhere parents can see.
- No "you focused for X minutes today" summary.
- No interruption alerts to anyone.

**Approvals (`requiresAck`).** A child can mark a homework item "show me to a parent before I mark it done". When the child taps "done", the parent sees a soft notification ("Sam wants you to check his maths worksheet"); the parent confirms. The flag is per-item, set by the child, never set by parents. The parent cannot retroactively add this flag to a child's task.

**Explicitly not in this cluster:** automated reading logs, device-free time enforcement, screen time tracking, time-on-task reporting, performance scoring of any kind.

### 5. Privacy

**Per-item visibility, set by the owner.** The same `visibility` field that exists on every entity (§3.6) applies. Children old enough can set their items to private (visible only to themselves) or family (visible to all family members).

**No admin reveal.** There is no parental override that exposes a child's private item. Building one creates a coercive-control vector and undermines the privacy guarantee the moment it exists. If a parent needs to know something, the way to find out is to ask the child, not to query the system.

**Age threshold for private status.** The household sets an age below which a child cannot mark items private from parents (default: 13). Children below the threshold can still have items marked "child only" by the *parent* setting them up — this is the parent saying "I'm not looking", and is a different psychological act than the child claiming privacy. The system does not collapse these two cases.

**Calendar density remains visible across privacy boundaries.** A teen marking a 16:00–18:00 Tuesday item private means the system shows them as *busy* in that window to the rest of the family, without showing *with what*. This answers the legitimate parental concern about private time conflicting with family obligations, without exposing the contents of that time. It is how privacy works between adults who trust each other.

**Onboarding the privacy decision.** When a child first marks an item private, the system shows a plain-language explanation: "Your parents will not see this. They will see that you are busy during this time, so they can plan around you. You can change this yourself any time." No legalese. No pretence that this is just a setting like any other — it's a meaningful action and the language reflects that.

### 6. Kid Mode

A view, not a separate app or mode. Each child sees a personalised "Today" that draws from the same data as everyone else, presented appropriately.

**What's in Kid Mode:**

- A short, plain list of what's on for today: homework, club, family events the child is part of.
- Each item is large and tappable. Mark done. Start a timer. Add a note. That's it.
- Plain Dutch / English language. No exclamation marks. No emojis pushed by the system (the child can add their own to their own items if they want).
- A "what about me?" affordance on the same pattern as the adults' Anchor view, when age-appropriate.

**What's *not* in Kid Mode:**

- Streaks, badges, points, levels.
- "Well done!" or "Try harder!" or any system-issued evaluation.
- Confetti or celebration animations on completion.
- Progress bars suggesting completion percentages.
- Leaderboards across siblings.

The reward for a child doing their homework is the parent's quiet acknowledgement, the avoided stress, and eventually the grade. The system stays out of that reward loop.

**Kid Mode auth.** A child can tap an NFC badge, scan a QR, or enter a PIN/emoji code (§O hardware). Switching is fast — the hub is shared and the child should be able to glance at it without ceremony. Visibility filters apply: a child sees their own private items, family items, and the items of others they have visibility into; never another child's private items, never any adult's private items.

### 7. Children and Presence

Children's Presence (§3.5) drives several behaviours in this cluster:

- A child away (`with_other_parent`, school trip, sleepover) doesn't get homework nudges if their own preferences permit, and is muted in family-row views.
- Shared-custody alternation is expressed as recurring Presence windows. Their school homework still exists as Tasks; only the surfacing changes.
- A child away does not appear in chore rotation for that period (Cluster 2).

### 8. What changed from §2, §3 (Homework), §16

- Removed: dedicated `Homework` entity. Homework is a Task with conventional tags. Simpler model, equivalent capability.
- Removed: streaks, confetti, "🔥 4-day streak!", and all gamified affordances from Kid Mode. Replaced with plain calm view.
- Removed from MVP: OCR homework capture, reading logs, device-free time prompts, screen time tracking.
- Added: `requiresAck` field on Task, set by the owner only.
- Added: age threshold for private visibility, set per-household.
- Clarified: privacy is per-item, owner-controlled, no admin reveal. Calendar density visible across privacy boundaries.
- Clarified: Kid Mode is a view over the same data, not a separate app surface.
