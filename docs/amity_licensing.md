# Amity — Licensing

*This document records the licensing decisions for the Amity project and the reasoning behind them. It is supplementary to the [LICENSE](LICENSE) file (the legally operative document) and the [philosophy document](amity_philosophy.md) (which states the values these licensing choices protect).*

## Code license

Amity is released under the **GNU Affero General Public License, version 3 or any later version** (AGPL-3.0-or-later).

The full license text lives in the [LICENSE](LICENSE) file at the root of the repository. Every source file carries a short header indicating its license.

## Why AGPL-3.0-or-later

The AGPL was chosen over plain GPL-3.0, MIT/BSD/Apache, and various source-available licenses for a specific reason: Amity is the kind of software where the network-service loophole in plain GPL would matter.

A plain GPL fork could be deployed as a hosted service — "FamilyConnect, powered by an internal Amity fork" — and never trigger the copyleft obligation, because no binary is distributed to end users. The fork's modifications, including those that violate the philosophy commitments, would never need to be published. The AGPL closes this loophole: a network service derived from Amity must publish its source under the AGPL as well.

The "-or-later" suffix lets the project move to a future AGPL version (4.0, etc.) if and when one exists, without requiring unanimous re-consent from every past contributor. This is the conventional choice and aligns with FSF recommendations.

## What this license does and does not do

**What AGPL-3.0 prevents:**

- Closed-source forks for distribution.
- Closed-source forks deployed as network services.
- Proprietary derivative works.

**What AGPL-3.0 does not prevent, and what no software license realistically can:**

- A commercial entity forking the code, renaming the fork, and shipping a values-stripped version — including one with advertising, telemetry, or surveillance features — as long as their source is also AGPL.
- A commercial entity building a paid service around the code, including one that contradicts Amity's principles, as long as their changes are published.
- A fork that calls itself "Amity" or something confusingly similar (the license does not govern naming; trademark does, and Amity does not currently hold a trademark — see below).

The license protects the code's openness. It does not protect the project's values. That protection is the work of the philosophy document, the project's governance, and — when warranted — a future trademark.

## Contributions

Contributions are accepted under the same AGPL-3.0-or-later license. We use a lightweight **Developer Certificate of Origin (DCO)** sign-off, not a heavyweight Contributor License Agreement (CLA). Contributors retain copyright in their contributions; they grant the project the right to distribute under AGPL.

To contribute, sign off on each commit with `git commit -s` (or by adding the `Signed-off-by:` trailer manually). This certifies that the contributor has the right to submit the code under the project's license. The full DCO text is included in [CONTRIBUTING.md](CONTRIBUTING.md).

This is deliberately not a CLA. A CLA would centralise rights with the project entity and create an asymmetric relationship between the project and its contributors. Some larger projects need that asymmetry (to dual-license, or to defend against patent claims with unified standing). Amity does not currently need it, and adding it would create unnecessary friction for contributors who are giving their time freely.

If the project's situation changes in the future and a CLA becomes necessary (for instance, to relicense for a specific institutional adoption), it would require explicit re-consent from existing contributors. That cost is intentional.

## Trademark

The name "Amity" is not currently registered as a trademark. This decision is deliberate but provisional.

The project may register the name as a trademark at a future point, when:

- A prior-art search confirms the name is available in the relevant trademark classes (typically class 9 software, class 42 SaaS/hosting, class 41 education/publishing) in the relevant jurisdictions (Benelux at minimum, possibly EU).
- The project has reached enough public identity that the name is materially associated with this codebase and these values.
- A fork or impersonation attempt makes the protection useful.

Until that point, the protection of the name relies on the visibility of the philosophy document, the project's governance practices, and social pressure within the community of users and contributors. This is a real but partial protection. Users and contributors should be aware that "Amity" as a project name is not legally protected, and a future fork could legally use the name (though they could not legally claim to be the project itself in a way that would mislead users).

## Distinguishing the project from forks

Even without a trademark, the project takes the following practical steps to make its identity legible:

- Releases are made from a canonical repository under the project's chosen GitHub organisation. Source tarballs are signed by the maintainer.
- The philosophy document is referenced from the README. Any release that omits or contradicts it is, by definition, not the project.
- The categorical commitments in the philosophy document are stated in language that makes a values-stripped fork awkward to market under the original name without an obvious contradiction.

A fork that retains the name "Amity" but adds advertising, telemetry, or surveillance would be in plain contradiction with the philosophy document the fork would have inherited. This is not legally enforceable; it is socially conspicuous.

## What is not licensed under AGPL

The following project assets are excluded from the AGPL and carry their own licensing terms:

- **Project documentation** (this file, the brief, the philosophy document) is licensed under **Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0)**. This allows free reuse and adaptation, with attribution and share-alike obligations.
- **Project name and logo** are not licensed for use in a way that would suggest endorsement or affiliation with the project. (This is the trademark-adjacent restriction that exists by default, even without registration — but is weaker than registered trademark protection.)
- **Third-party dependencies** retain their own licenses; the project's use of them is governed by their respective terms. The dependency manifest tracks these.

## Summary

- Code: **AGPL-3.0-or-later**
- Documentation: **CC BY-SA 4.0**
- Contributions: DCO sign-off; no CLA
- Trademark: deferred; revisit when project gains public identity
- Soul protection: philosophy document + governance + future trademark

This combination is the strongest realistic protection for an open-source household-tech project at this stage, balancing genuine openness with meaningful resistance to soul-stripping forks.
