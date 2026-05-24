## Cluster 3 — Meals, Groceries, Pantry

*Answers §21F (questions 18–21). Replaces parts of §3 (Recipe, Meal, List entities) and §8 (Meal → Groceries pipeline). Adds DietaryProfile and extends Presence to cover guests with persistent dietary needs.*

### 1. Posture

The meal-to-groceries pipeline is one of the few places in this system where real automation pays for itself. "What's for dinner?" is the most repeated cognitive-load question in most households. If the hub answers it reliably, it has justified its existence on that alone.

The posture is **structure grows with use**, not "set up your recipe database first":

- Meals are labels by default. A label is just a word or phrase attached to a date and slot.
- A label can optionally be linked to a structured Recipe, which enables auto-grocery generation. The user invests in structure only where the payoff is felt.
- The system never demands the structured form. A household can use this for years with nothing but meal labels and a manually-managed grocery list.

The system is not a dietician, a nutrition tracker, a calorie counter, a substitution engine, or an inventory system. It is a planner for the question "what are we eating, who's eating it, and what do we need to buy".

### 2. Meals

**Scope: dinner only, weekly.** Breakfast and lunch are pattern-based and don't need planning. The exception — a kid's school-trip lunchbox, a guest brunch — is an ad-hoc Meal entry. No daily breakfast/lunch tracker.

**A "today" slot for unplanned cooking.** If what you actually ate didn't match the plan, you can update the slot retroactively. The history stays honest, which makes the planner useful as a memory rather than just an aspiration.

```
Meal {
  id, date, slot(enum: dinner | breakfast | lunch | other),
  label,                    # free-text: "pasta pesto", "leftovers"
  recipeId?,                # optional link to structured Recipe
  notes?,
  servings?,                # default: count of present household members
  guestCount?
}
```

A meal label of `leftovers` or `use-first` is a planning convention, not a feature. The system shows the slot as a normal Meal; the user just types what they're using up.

### 3. Recipes (optional structure)

```
Recipe {
  id, title,
  ingredients[ { name, qty?, unit?, optional? } ],
  steps[],                  # optional; the user may or may not bother
  tags[],                   # "vegetarian", "quick", "kids favourite"
  allergens[],              # "gluten", "nuts", "dairy"
  defaultServings,
  notes?
}
```

A Recipe is created only when the user wants its benefits — most prominently, auto-generating grocery items by multiplying ingredient quantities against `servings / defaultServings`.

**Recipe promotion suggestion** (post-MVP, low priority): if a meal label is used repeatedly with the same supply prompt, the system can offer to save it as a Recipe. Never automatic; always a question the user can dismiss without consequence.

### 4. Dietary needs — DietaryProfile and Presence

The system distinguishes **safety flags** from **preference flags**, with structurally different behaviour. Neither is a hard block; both can apply to household members and to recurring guests.

```
DietaryProfile {
  id, name,
  relationship,             # free-text: "Sam (our son)", "Anna (son's girlfriend)"
  safetyFlags[],            # ["gluten", "peanuts", "shellfish"]
  preferenceFlags[],        # ["vegetarian", "no pork", "halal"]
  prepNotes,                # "separate margarine; check sauces;
                            #  she brings her own bread sometimes"
  linkedMemberId?           # if this person is also a household member
}
```

A DietaryProfile is not a contact entity. No phone numbers, no birthdays. It exists *because the person has needs the cook needs to know about*. If they don't, no profile is created.

**Presence extends to guests.** A Presence record can refer to either a `memberId` (household member) or a `guestProfileId` (DietaryProfile). When recording a visit — "Anna here Friday evening through Sunday" — Presence does the rest: her dietary flags become active for that window.

**Behaviour at meal planning time:**

- **Safety flags**: when a meal is planned in a window where someone with a conflicting safety flag is present, the system shows an inline warning that requires acknowledgement. *Not a block* — the user may have context the system doesn't (the person isn't eating that night; the dish has a variant). The warning is structured: it names the person, the flag, and surfaces their `prepNotes`. The acknowledgement is the system saying "are you sure", not "I forbid this".

- **Preference flags**: passive notice only. "This contains meat. Anna is vegetarian." No acknowledgement, no friction. The user knows.

- **Day-pattern hints**: a date can carry household-level tags like `vegetarian-tuesday`. These produce preference-level notices when a conflicting meal is planned. Not enforced.

Both flag types respect Presence. Sam's allergy doesn't warn for meals on days he's at his other parent's house.

**Visibility default**: dietary safety flags are visible to all adults and teens, never to guests. Children's flags are managed by adults. This is medical-adjacent data and should be treated with appropriate visibility.

### 5. Grocery list

The list is the simplest part of the system, and should stay that way.

```
GroceryItem {
  id, name, qty?, unit?,
  category?,                # "produce", "dairy", "household"
  preferredStore?,
  source(enum: manual | from_meal | from_pantry | from_recipe),
  sourceRefId?,             # which meal/recipe/pantry item put it here
  checked: bool,
  notes?
}
```

**Three ways an item gets on the list:**

1. **Manually**, by anyone: voice ("add milk"), tap, mobile.
2. **From a planned meal**: if the meal has a linked Recipe, its ingredients (after servings-scaling) join the list. If the meal is unlinked, the user can tap "what do you need?" and type/speak items, which are tagged `from_meal` for traceability.
3. **From the pantry**, when a pantry item drops to or below its low threshold (§6 below).

**No deduplication magic.** If "milk" appears twice with different sources, it stays as two entries — the user resolves it in the moment. Auto-merging is the kind of feature that goes wrong subtly. A simple "merge duplicates?" affordance is fine; automatic merging is not.

**Grouping.** The list can group by `category` (default) or by `preferredStore`. Aisle-order sorting is **not** in MVP — stores rearrange themselves and the teaching overhead exceeds the value.

**Substitution rules** ("brand → generic"): not in MVP. Users substitute in the moment.

### 6. Pantry

```
PantryItem {
  id, name, unit?,
  currentLevel(enum: empty | low | ok | full),
  lowThreshold(enum: low | empty),     # auto-list at this level
  preferredStore?,
  dateAdded?,                          # for "show oldest first", not for alerts
  lastUpdatedAt
}
```

**Coarse levels, not quantities.** "We have flour" / "running low" / "we're out" is what people actually know. Numeric tracking is exposed via `unit?` for cases that warrant it (chicken feed by weight, say), but the default is the enum.

**Level updates happen as a side effect of normal activity:**

- A chore checklist can include a step like "How's the dishwasher salt?" with a level picker. The dishwasher chore carries this check; running the chore updates the pantry.
- The cook can answer "is there enough X?" during meal prep — a small button-prompt in the meal view, optional.
- Manual update via the pantry view itself.

When `currentLevel <= lowThreshold`, a corresponding GroceryItem is auto-created tagged `from_pantry`. Marking that item purchased on the list resets the pantry level to `ok`.

### 7. Use-first list (waste minimisation)

**No expiry tracking.** Populating expiry data requires per-item input that nobody sustains for more than two weeks. Build it and watch it become noise.

What is in MVP: a simple `use-first` view. A free-text list, manually populated, of items the cook wants to use this week. "Bunch of asparagus needs eating" — typed in, shown in the meal-planning view as a hint when picking what to make.

```
UseFirstItem {
  id, name,
  addedAt, addedBy,
  notes?
}
```

That's the whole entity. No expiry date, no urgency level, no notifications. The cook glances at it while planning; once an item is used it's marked done and disappears.

### 8. Meal → grocery pipeline (end-to-end summary)

For a typical week:

1. The cook fills in dinner slots for the coming week. Some are labels ("pasta pesto"). Some link to Recipes ("Lasagne — Recipe #4").
2. Presence is checked for each day: who is home, plus any guest with a DietaryProfile.
3. Safety conflicts produce acknowledgement-required warnings; preferences produce passive notices.
4. **Generate Groceries** action (a button on the Meals view):
   - For meals with linked Recipes: ingredients are summed across the week, scaled by servings, grouped by category.
   - For meals without recipes: the cook is prompted once per such meal: "what do you need for X?" — a quick free-text input, voice-friendly.
   - Pantry items below threshold are added.
   - Use-first items are *not* added (they're already at home).
5. The combined list is reviewed, edited as needed, and ready for the shopping trip.

The whole pipeline can run end-to-end without a single Recipe existing. It just falls back to "what do you need for this?" prompts. Households that invest in recipes get the structured payoff; households that don't still get the planning value.

### 9. What changed from §3 and §8

- `Meal`: `recipeId` is now optional. `label` becomes the primary identifier.
- `Recipe`: simplified; some fields (`prepTime`, `cookTime`) deferred to post-MVP.
- `List` entity from original §3: split into `GroceryItem` and `UseFirstItem`. The original generic `List` is too vague; specific types are clearer.
- New: `DietaryProfile`, `PantryItem`, `UseFirstItem`.
- Extended: `Presence` accepts `guestProfileId` in addition to `memberId`.
- §8 (Meal → Groceries) replaced by §8 of this cluster above. Removed: "smart rules" like "always buy milk on Sunday" (covered by pantry thresholds), "double if guests ≥ 6" (covered by servings scaling).
