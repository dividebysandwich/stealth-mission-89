# Translations

The game's texts, one folder per language (`en`),
in [Fluent](https://projectfluent.org/fluent/guide/) files. They are built into the game
(`crates/stealth-i18n`): no file is read at run time.

- English (`en/`) is the source. A message missing from a language is shown in English.
- `cargo test -p stealth-i18n` checks every language: no English message missing, none
  English does not have, no variable English does not pass.
- `en/briefings.ftl` is written by `python tools/i18n_briefings.py` from the mission
  generator's data (`assets/mission/briefings.json`, `missions.ron`); run it after changing
  them.
- `--lang <code>` starts the game in a language (`--lang pseudo`: English with accents and
  30% longer, to spot untranslated texts and texts that do not fit).

## What stays in English

Like in the real aircraft and in the other flight simulators:

- What the cockpit shows: HUD, MFD and panel labels, lamps, bezel and console buttons.
- Acronyms: HUD, MFD, FLIR, ILS, TACAN, RWR, ECM, IFF, CCIP, CCRP, SAM, AAA, CSAR, HARM, AWACS,
  SEAD, CAP, CAS, SOI, TGP, HSD, HSI, DED, EMV, ...
- Aircraft, weapon, unit and place names.
- The units the avionics show (ft, kt, nm, lbs on the displays).

In tooltips the title stays the cockpit's label when there is one (FLIR, MASTER CAUTION), the
description is translated.

## Fonts

- Menus, tooltips, the 2D overlays: egui's Hack (Latin, Greek, Cyrillic; no CJK).
- The HUD and displays: a 3x5 pixel font, ASCII capitals only: messages there are shown in
  capitals without accents (`stealth_i18n::ascii_upper`).
- The in-flight menu, the kneeboard, the key guide: a stroke font with the accents of the
  Latin languages. A language in another alphabet (Russian) shows these in English
  (`stealth_i18n::flight_language`, the `tf!` macro).

## Fluent in short

```
message-id = Text with a { $variable }
    .attribute = Another text of the same message
multi-line =
    First line
    Second line
plural = { $n ->
    [one] { $n } hit
   *[other] { $n } hits
}
```

A line of a multi-line text must not start with `[`, `*` or `.`; braces are written
`{"{"}` and `{"}"}`; a leading or trailing space is written `{" "}`.
