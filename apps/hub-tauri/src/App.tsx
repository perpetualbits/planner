/* App.tsx — The hub capture and inbox view.
 *
 * This is the minimal UI for Task 1: a capture form and a recent items list.
 * It is not the full hub-at-rest design (clock, weather, status patch) —
 * that arrives in a later task. This exercises the full architecture end-to-end.
 *
 * Design constraints (from the task spec and philosophy):
 *   • Atkinson Hyperlegible font loaded as a project asset.
 *   • 60×60px minimum touch target on the submit button.
 *   • Plain calm visual style — no decorative elements, no submit animation,
 *     no toast notifications. Capture happens; item appears in list. That is
 *     the entire feedback (brief §3: "the empty state is a designed state").
 *   • The capture form clears on successful submission.
 *   • The recent list refreshes on mount and after each successful capture.
 */

import { createSignal, onMount, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// ─── Types ────────────────────────────────────────────────────────────────────

/** An inbox item as returned by the Tauri commands. */
interface InboxItem {
  id: string;
  raw_text: string;
  captured_by: string;
  captured_at: string;
  source: string;
  triage_state: string;
  triaged_to?: string;
}

// ─── Component ────────────────────────────────────────────────────────────────

export default function App() {
  /* Local state: the text currently typed in the capture input. */
  const [captureText, setCaptureText] = createSignal("");

  /* The list of recent inbox items shown below the form. */
  const [items, setItems] = createSignal<InboxItem[]>([]);

  /* Error message when capture or list fails; null when no error. */
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);

  /* True while a capture request is in flight; disables the submit button
   * to prevent double-submission without any visual animation or loading state
   * beyond the button being non-interactive. */
  const [submitting, setSubmitting] = createSignal(false);

  /** Load the most recent inbox items from the service. */
  async function loadRecent() {
    try {
      const result = await invoke<InboxItem[]>("list_recent_inbox", {
        limit: 20,
      });
      setItems(result);
    } catch (err) {
      /* A failed refresh is silent — the existing list stays visible.
       * We don't want a failed background refresh to disrupt the user. */
      console.error("list_recent_inbox failed:", err);
    }
  }

  /** Submit the capture form. */
  async function handleSubmit(e: Event) {
    e.preventDefault();

    const text = captureText().trim();
    if (!text) {
      /* Don't submit an empty form. The input already has required, but
       * we guard here too because the service would return 422. */
      return;
    }

    setSubmitting(true);
    setErrorMessage(null);

    try {
      await invoke<InboxItem>("capture_inbox_item", { rawText: text });
      /* Clear the input on success — the captured thought is now stored. */
      setCaptureText("");
      /* Refresh the list so the new item appears immediately. */
      await loadRecent();
    } catch (err) {
      /* Show a plain error message. No toast, no modal, no animation. */
      setErrorMessage(typeof err === "string" ? err : "capture failed");
    } finally {
      setSubmitting(false);
    }
  }

  /* Load the list once when the component mounts. */
  onMount(loadRecent);

  return (
    <main class="hub">
      {/* ── Capture form ────────────────────────────────────────────────── */}
      <section class="capture-section" aria-label="Capture">
        <form class="capture-form" onSubmit={handleSubmit}>
          <label class="capture-label" for="capture-input">
            What's on your mind?
          </label>
          <div class="capture-row">
            <input
              id="capture-input"
              class="capture-input"
              type="text"
              value={captureText()}
              onInput={(e) => setCaptureText(e.currentTarget.value)}
              placeholder="type a thought…"
              autocomplete="off"
              required
              disabled={submitting()}
            />
            {/* The submit button meets the 60×60px minimum touch target
                specified in brief §12.4. Size is set in style.css. */}
            <button
              class="capture-submit"
              type="submit"
              disabled={submitting() || !captureText().trim()}
              aria-label="Capture"
            >
              ↵
            </button>
          </div>
          {/* Plain error text — no colour-as-sole-carrier, no icon animation */}
          {errorMessage() && (
            <p class="capture-error" role="alert">
              {errorMessage()}
            </p>
          )}
        </form>
      </section>

      {/* ── Recent items ────────────────────────────────────────────────── */}
      <section class="recent-section" aria-label="Recent captures">
        {items().length === 0 ? (
          /* The empty state is a designed state (brief §3): say nothing today
           * and mean it. No spinner, no skeleton, no placeholder cards. */
          <p class="empty-state">nothing captured yet</p>
        ) : (
          <ol class="item-list" reversed>
            <For each={items()}>
              {(item) => (
                <li class="item">
                  <span class="item-text">{item.raw_text}</span>
                  <time class="item-time" dateTime={item.captured_at}>
                    {formatTime(item.captured_at)}
                  </time>
                </li>
              )}
            </For>
          </ol>
        )}
      </section>
    </main>
  );
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/** Format an ISO-8601 timestamp as a short human-readable time.
 *
 * For items captured today: "10:42". For older items: "May 24".
 * Simple and locale-aware without a date library dependency.
 */
function formatTime(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();

  const sameDay =
    date.getFullYear() === now.getFullYear() &&
    date.getMonth() === now.getMonth() &&
    date.getDate() === now.getDate();

  if (sameDay) {
    /* 24-hour time — consistent with the Dutch date convention default (brief §16). */
    return date.toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      hour12: false,
    });
  }

  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}
