# slopsmith.github.io

The website for the [slopsmith](https://github.com/slopsmith) projects: AI-written software with little code review and a lot of craft.

Served by GitHub Pages at `https://slopsmith.github.io`. Each project gets a folder, for example `/ps5slop`.

The repo is private and Pages is not set up yet.

## Plan for ps5slop

- Pages from the ps5slop user docs (`docs/user/` in `slopsmith/ps5slop`) and a compatibility database that replaces `COMPATIBILITY.md`.
- Compatibility reports are GitHub issues in a separate repo, `slopsmith/ps5slop-compat`. One issue per report. The issues are the database: nothing writes reports into git.
- Two issue forms:
  - `launcher-report`: opened by the ps5slop launcher with every field filled in. All fields are text inputs, since GitHub does not prefill dropdowns from the URL.
  - `report`: the manual form the website links to. Status is a dropdown. Environment (macOS version, chip, ps5slop version) is required.
- The launcher asks the reporter six questions and derives the status from the answers:
  1. Anything on screen? No: ⚫ Nothing
  2. Reached the title screen or menus? No: 🔴 Boots
  3. Reached gameplay? No: 🟠 Menus
  4. Played through, or a long session for endless games? No: 🟡 Ingame
  5. Anything broken? Yes: 🟢 Playable
  6. Played it fully? No: 🟢 Playable, yes: ⭐ Perfect
- Reports without an environment are rejected. An Action may add labels and comments; it never writes to a repo.
- The site build reads the issues through the GitHub API and produces the pages plus `compat.json`. The launcher reads `compat.json` and shows the status from reports that match the user's environment.

## How the site is built

- [Zola](https://www.getzola.org) builds the site. A small Rust tool, `tools/sitegen`, runs first and writes the generated pages and data that Zola reads.
- Each project is a Zola section: `content/<project>/` and `templates/<project>/`, with a shared base template.
- Docs: each build copies `docs/user/` from the tip of `trunk` in `slopsmith/ps5slop`. There are no release tags, only nightly builds. The user docs link only to each other and to the compatibility pages, never to developer docs.
- Compatibility: each build reads the issues in `slopsmith/ps5slop-compat` that an Action has labelled as accepted. The rows in `COMPATIBILITY.md` are filed there as issues, and the file is then removed.
- `compat.json`: its schema is defined in this repo and agreed with the ps5slop launcher.
- ps5slop, ps5slop-compat and this repo will all be public, so the build needs no token to read them.
