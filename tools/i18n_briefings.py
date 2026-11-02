"""Writes assets/i18n/en/briefings.ftl from the mission generator's English texts
(assets/mission/briefings.json and the intelligence summaries of assets/mission/missions.ron),
so that every language translates the same messages. Run it after changing either file:

    python tools/i18n_briefings.py

The generator (crates/stealth-mission/src/text.rs) uses the English of the data files and
looks up these messages for the other languages: `bt-<entry>-<variant>`, the objective
templates' `{name}` placeholders becoming Fluent's `{ $name }`.
"""

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MISSION = ROOT / "assets" / "mission"
OUT = ROOT / "assets" / "i18n" / "en" / "briefings.ftl"

# The theater objects' type names (the `{what}` of the objectives): the other languages give
# them with their article where theirs needs one.
OBJECTS = [
    "Aegis Cruiser", "Airbase", "Airplane", "Airstrip", "Bio Wpns Plant", "Bridge", "Buildings",
    "Burning Oil Rig", "Camp", "Cargo Ship", "Chem Wpns Plant", "Depot", "Destroyer", "Hangar",
    "Headquarters", "LPAR Radar", "Merchant Ship", "Missile", "Missile Boat", "Mobile SAM",
    "Nuclear Plant", "OTH Radar", "Offshore Oil Rig", "Oil Refinery", "Oil Rig", "Oil Tanker",
    "POW Camp", "Patrol Boat", "Pump Station", "Refinery", "SA Missile", "SAM Radar",
    "SS Missile", "Ship", "Sub Pen", "T-72 Tanks", "Tank", "Tank Farm", "Tanker",
    "Terrorist Camp", "Tower", "Town", "Village", "Windmill",
]

MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]


def slug(s):
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")


def fluent(text):
    """A template as a Fluent pattern: placeholders, braces, leading and trailing spaces."""
    out = re.sub(r"\{([a-z_]+)\}", lambda m: "{ $" + m.group(1) + " }", text)
    if out.startswith(" "):
        out = '{" "}' + out[1:]
    if out.endswith(" "):
        out = out[:-1] + '{" "}'
    if out == "":
        out = '{""}'
    return out


def ron_briefings():
    """(template id, [summaries]) from missions.ron."""
    text = (MISSION / "missions.ron").read_text(encoding="utf-8")
    out = []
    for m in re.finditer(r'\(id: "([a-z0-9_]+)"', text):
        start = text.index("briefings:", m.end())
        open_ = text.index("[", start)
        depth, i = 0, open_
        while True:
            c = text[i]
            if c == "[":
                depth += 1
            elif c == "]":
                depth -= 1
                if depth == 0:
                    break
            elif c == '"':
                i += 1
                while text[i] != '"':
                    i += 2 if text[i] == "\\" else 1
            i += 1
        body = text[open_ + 1 : i]
        lines = [s.replace('\\"', '"') for s in re.findall(r'"((?:[^"\\]|\\.)*)"', body)]
        out.append((m.group(1), lines))
    return out


def main():
    b = json.loads((MISSION / "briefings.json").read_text(encoding="utf-8"))
    out = [
        "# The mission generator's briefings (crates/stealth-mission/src/text.rs). Written by",
        "# tools/i18n_briefings.py from assets/mission/briefings.json and missions.ron: change",
        "# those, then run the tool. Each entry has variants (-0, -1...), one picked per mission;",
        "# a language may have as many as it likes of the English ones, the missing ones are",
        "# taken from English.",
        "",
        "## Headers, titles, labels",
        "",
    ]
    for i, h in enumerate(b["headers"]):
        out.append(f"bt-header-{i} = {fluent(h)}")
    for k, titles in enumerate(b["titles"]):
        for i, t in enumerate(titles):
            out.append(f"bt-title-{k}-{i} = {fluent(t)}")
    for key, value in b["labels"].items():
        out.append(f"bt-label-{slug(key)} = {fluent(value)}")
    for name in ["pilot", "depart", "land", "fuel"]:
        for i, t in enumerate(b[name]):
            out.append(f"bt-{name}-{i} = {fluent(t)}")
    out += ["", "## Rules of engagement (0 clandestine, 1 limited, 2 full war)", ""]
    for r, texts in enumerate(b["roe"]):
        for i, t in enumerate(texts):
            out.append(f"bt-roe-{r}-{i} = {fluent(t)}")
    out += ["", "## Objectives", ""]
    for key, texts in b["objectives"].items():
        for i, t in enumerate(texts):
            out.append(f"bt-{slug(key)}-{i} = {fluent(t)}")
    out.append("bt-fighter-patrol = { $plane } fighter patrol")
    out += ["", "## The briefing's date", ""]
    for i, m in enumerate(MONTHS):
        out.append(f"bt-month-{i + 1} = {m}")
    out.append("bt-date = { $month } { $day }, { $year }")
    out += ["", "## The targets' types", ""]
    for o in OBJECTS:
        out.append(f"obj-{slug(o)} = {o}")
    out += ["", "## Intelligence summaries, by mission template (missions.ron)", ""]
    for tid, lines in ron_briefings():
        for i, t in enumerate(lines):
            out.append(f"bt-intel-{slug(tid)}-{i} = {fluent(t)}")
    OUT.write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")
    print(f"{OUT}: {len(out)} lines")


if __name__ == "__main__":
    main()
