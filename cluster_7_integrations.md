## Cluster 7 — Integrations

*Answers §21J (questions 31–33). Replaces §10 (Integrations) and reshapes the integration footprint to fit the categorical commitments established in Cluster 6.*

### 1. Posture

Every integration with an external system is a potential leak of the boundary drawn in Cluster 6: *the kitchen is a safe place, no surveillance vectors, no data flow that could become an advertising surface*. Each integration is evaluated against that boundary; many fail.

A useful three-category split:

- **Read-only inbound** (ICS feeds, weather, public data). The system pulls; data flows inward only. Low risk.
- **Local-network** (Home Assistant, smart-home devices on the household's own network). Data stays inside the household. Risk depends on the specific device's own posture.
- **Cloud-connected outbound** (email APIs, third-party services, commercial integrations). Data flows out to another party. Highest risk; most should fail evaluation against the categorical commitment.

The MVP integration footprint is deliberately small. Each addition past MVP gets reviewed against the same standard.

### 2. Calendar sources (already in MVP from Cluster 1)

These are integrations even though they don't feel like it — they connect the system to external sources. Already covered in Cluster 1:

- School ICS feeds, per child, per school.
- Sports/club ICS feeds where available.
- Municipal afvalkalender (Zutphen): plastic, paper, GFT, rest.
- Public NL holiday calendar.
- Adult members' personal Google/Apple calendars (read-only on the hub).

All read-only. All inbound. Low risk category. The hub-native calendar is the only thing the hub itself writes to.

### 3. Home Assistant — post-MVP

Home Assistant is the least risky non-trivial integration in this cluster: it runs on the household's own hardware, data stays on local network, and the user controls it entirely. But it's a meaningful system in its own right, and integrating it well requires more design and testing than the MVP can afford.

**Deferred to post-MVP.** When it lands, the integration model is:

- The hub speaks to Home Assistant via its local API.
- Initial scope: *receive* events from HA. Presence detection (member arrived/left), appliance signals (washer done, dishwasher cycle complete), local-network sensor data.
- Households configure which HA events feed into the surfacing layer, and as which item types (Today notice, Soft notice, pantry-level update).
- Sending events *to* HA (e.g. mark bins task done → fire light event) is later still.
- No HA cloud component; the integration runs entirely on local network or it doesn't run at all.

For households without Home Assistant, the system works completely without it. HA is an *amplifier*, never a dependency.

### 4. Email — forwarding address, not parsing

The brief proposed parsing email from schools and clubs to extract dates and fees automatically. **Rejected for MVP and probably for v1.**

Costs of email parsing:

- Requires inbox access, a significant trust delegation. The inbox contains far more than school emails.
- Parsing is fragile across school newsletter formats and club templates. False positives create wrong items; false negatives miss important things. The system shares blame for both.
- Either runs in the cloud (violates Cluster 6 categorical commitments) or requires substantial local compute.

**The replacement**: a per-household forwarding address.

- The system provides an inbound address (`yourhouse@yourdomain.example` or similar).
- Users forward emails they want captured.
- The system extracts datetimes, links, and attachments from the forwarded email and creates an `InboxItem` with the forwarded content attached.
- The user reviews and either accepts a proposed Event/Task or keeps the inbox note as-is.

This puts the user in the loop on every extraction, makes errors visible and correctable, and avoids the inbox-access privacy cost entirely. It gives up "magic" in exchange for honesty — a trade this system should make.

### 5. Location and weather

**Weather: MVP.** The hub shows a one-line weather summary as part of the at-rest baseline (Cluster 5). Weather queries go to a public service; no household data is sent outward beyond approximate location (city-level), which the user configures explicitly.

**On-device geofences: v1, strictly client-side.** When the mobile app is on a phone near a configured location (the supermarket, school), the app can surface the relevant list locally. **The location never leaves the device.** The phone knows where it is; the phone shows the local list. No location data transmits to any server, including the hub, including any third party. This is implementable but must be implemented correctly; most app frameworks make it easy to get wrong by quietly sending location to a server "for convenience".

**Routing, commute estimates, traffic-aware reminders: out of scope.** These require sending location and destination to a routing provider, which violates the boundary. The user already has Google Maps or Apple Maps on their phone for this purpose; the system does not duplicate or proxy that functionality.

### 6. Grocery delivery export — user-initiated, not integrated

Many users will eventually want to send their grocery list to a delivery service. The system supports this as a **user-initiated export action**:

- Generate the grocery list in a structured format (plain text, CSV, shareable text).
- The user copies, pastes, or shares it into their preferred delivery app.
- The system does *not* talk to grocery delivery APIs directly.

This sounds like a smaller feature than "integrate with Picnic" or "integrate with Albert Heijn", and it is — deliberately. Direct integration would create a commercial relationship with a grocery chain, exactly the dynamic the Cluster 6 commitment was drawn to prevent. The user can already paste a list into whatever app they like; the system stays out.

### 7. Photo-based capture — post-MVP, with constraints

Not in the original brief, but raised in conversation: a household member points a phone or hub camera at a package and asks "does this contain gluten?" or "what could I make with this?". This is the visual analogue of push-to-talk: explicit single-shot capture, used as a tool, discarded after use.

**Out of MVP.** When it lands, the constraints are:

- Photo capture is always explicit (button press, never automatic).
- Images are processed and discarded; only resulting answers or extracted text persist.
- On-device vision processing where hardware allows; opt-in cloud fallback otherwise (same model as voice).
- No facial detection or recognition in the image pipeline, ever — even for benign use cases.

These constraints are non-negotiable, on the same grounds as the always-listening prohibition: capturing visual information in the kitchen is the same category of surveillance vector as audio, and the boundary is consistent.

### 8. Integration boundary checklist

Any future proposed integration is evaluated against this short checklist before consideration:

- **Does data flow outward?** If so, to whom, how much, when, and what's the recourse?
- **Could this data become an advertising surface for the receiving party?** If yes, refuse.
- **Does the integration require always-on capture of any kind?** If yes, refuse.
- **Does it create a commercial relationship that gives a third party leverage over the system's behaviour?** If yes, refuse.
- **Can the same value be delivered by user-initiated action instead?** If yes, prefer that path.

This checklist is the operationalisation of the Cluster 6 categorical commitment. It belongs in the consolidated brief as a near-front section.

### 9. What changed from §10

- §10's Phase 1 (Google/Apple/ICS read-write, Reminders/Tasks import): Adjusted. The hub *reads* personal calendars; it does *not* write back to them. Native family events live in the hub-owned calendar (per Cluster 1).
- §10's Phase 2 (Home Assistant): kept, but moved fully to post-MVP with explicit scope on inbound events only.
- §10's Phase 3 (School portals, grocery delivery): substantially reframed. School portal *scraping* is out. Email *parsing* is replaced by manual forwarding. Grocery delivery is user-initiated export, not integration.
- Added: on-device geofences (v1, strictly client-side), photo-based capture (post-MVP, with strict constraints), integration boundary checklist (operational tool).
- Removed: any integration model that involves outbound household data to a third party.
