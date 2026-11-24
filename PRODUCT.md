# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

- Mac players: people with an Apple Silicon Mac and their own PS5 games, who want to know if a game runs and how to start it.
- Onlookers: emulation fans and developers following the projects.

Both get equal weight.

## Product Purpose

The site for the slopsmith GitHub org: AI-written software with little code review and a lot of craft. Each project gets its own section under `slopsmith.github.io/<project>`.

- **ps5slop**: a PS5 compatibility layer for Apple Silicon Macs. Its section has the user docs, a compatibility database built from GitHub issues, and (once public) nightly downloads of `ps5slop.app`.
- **86slopper** (rename pending): an x86-32 PC emulator for Apple Silicon Macs that aims to run Windows XP faster than QEMU's TCG, using Hypervisor.framework. It joins the site later.

## Positioning

Every project is written by AI agents with almost no human review of the code, and says so openly. The craft is in what gets built, how it is tested and what ships. The site should look like a real emulator project made with care, not like generic AI output: a page that screams "AI slop" works against the point.

## Capabilities and Constraints

- Static site: Zola, with a Rust tool (`tools/sitegen`) generating docs and compatibility pages before the build. Hosted on GitHub Pages.
- ps5slop's main action is downloading a nightly build. There are no release tags.
- Compatibility statuses: Perfect, Playable, Ingame, Menus, Boots, Nothing.

## Brand Commitments

- Name: slopsmith. Tagline: "AI-written software. Little code review, a lot of craft."
- Layout reference: the Dolphin emulator site, for structure only (it looks dated). Do not copy it.
- The site must say it does not own game assets, and that PlayStation and PS5 are trademarks of Sony. ps5slop contains nothing from Sony.

## Evidence on Hand

- Docs: `docs/user/` in `slopsmith/ps5slop`, due for a rewrite.
- Compatibility data: 7 titles from the old `COMPATIBILITY.md`, as sample issues in `fixtures/ps5slop-issues.json`.
- No screenshots, box art or game icons yet. They will be collected later; game icons are allowed with the disclaimer. Do not invent them.

## Product Principles

- Honest about how the software is made, and about how far each game gets.
- Show the work: screenshots and real results over claims.
- Useful first for someone checking whether their game runs.
