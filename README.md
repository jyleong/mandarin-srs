# Mandarin SRS

A terminal spaced-repetition app for Mandarin: you see **汉字**, type the **English meaning**, then rate how well you knew it. Pinyin and the gloss stay hidden until after you answer.

Built as a small Rust + [ratatui](https://ratatui.rs/) TUI. The select screen is **TOCFL 1–5** (official 八千詞 Levels 1–5) plus **Extra** (Novice 1–2 and leftover HSK-only words). Headwords and examples are **Taiwan Traditional**. Progress is stored locally as JSON.

## Requirements

- Rust toolchain (edition 2024; `rustup` stable is fine)
- A terminal that can show CJK (most modern terminals)
- Run from the **crate root** so `data/tocfl1.json` … `tocfl5.json` and `data/extra.json` resolve

## Run

```bash
cargo run
```

First launch creates `data/progress.json` as you grade cards. That file is yours (intervals and due dates); it is safe to delete if you want a clean slate.

```bash
cargo test    # domain tests (matching, intervals, due dates)
```

## Keys

| Where | Keys |
|-------|------|
| **Level select** | `↑` `↓` / `j` `k` · `1`–`5` TOCFL · `6` Extra · `[` `]` / `h` `l` batch (10/20/30/50) · `Enter` start |
| **Prompt** | type English · `Enter` check · `Backspace` |
| **Reveal** | `1` / `a` Again · `2` / `g` Good · `3` / `e` Easy · `←` browse |
| **Browse** | `←` `→` history (read-only) · `→` past last returns to reveal/quiz |
| **Summary** | any key → level select |
| **Anywhere** | `Esc` or `Ctrl+Q` quit |

Answers are lenient: case, extra spaces, and either side of a `;` gloss (`dad` or `father`).

Scheduling (after you press 1/2/3):

| Grade | Next due |
|-------|----------|
| Again | today, and the card is queued again later in this session |
| Good | previous interval × 2 (min 2 days from new) |
| Easy | previous interval × 4 |

A session is a **batch** of 10 / 20 / 30 / 50 cards (default 50) at the chosen TOCFL band or Extra, shuffled: due **reviews** first, then up to **10 new** cards per calendar day (shared across levels), then **study ahead** (cards not due yet) so you can still start when the due/new queues are empty. Ahead grades still update intervals. Level select shows `review · new · ahead / total`. The quiz header adds `N again` while Again-queued cards are still waiting. Reveal and browse show **You typed** next to the gloss.

## Interaction flow

```mermaid
flowchart TD
  start([cargo run]) --> select[TOCFL level select]
  select -->|Enter, if any due| prompt[Prompt: show 汉字, type English]
  select -->|nothing due| select
  prompt -->|Enter| reveal[Reveal: pinyin + meaning + ✓/✗]
  reveal -->|1 Again / 2 Good / 3 Easy| save[Save interval + due_date]
  reveal -->|left arrow| browse[Browse session history]
  browse -->|left / right| browse
  browse -->|right past last| reveal
  save -->|more cards| prompt
  save -->|session done| summary[Summary: answered / correct / wrong]
  summary -->|any key| select
  select -->|Esc / Ctrl+Q| exit([quit])
  prompt -->|Esc / Ctrl+Q| exit
  reveal -->|Esc / Ctrl+Q| exit
```

Text version of the same loop:

```text
  cargo run
      │
      ▼
 ┌─ TOCFL / Extra select (due / total per level) ◄──────┐
 │        Enter                                       │
 ▼                                                    │
 Prompt (汉字) ──Enter──► Reveal                      │
                            │                         │
                   1 / 2 / 3 grade + save             │
                            │                         │
              more cards ───┴─── last card            │
                   │                    │             │
                   ▼                    ▼             │
                Prompt              Summary ──any key─┘
```

## Data

| File | Role |
|------|------|
| `data/tocfl1.json` … `tocfl5.json` | Official TOCFL Levels 1–5: `chinese`, `pinyin`, `meaning`, `level`; optional `example` / `example_en` |
| `data/extra.json` | Novice 1–2 plus HSK-only leftovers (`level` 0, ids `extra:漢字`) |
| `data/id_map.json` | Old card id → new id (script, wording, or re-level). Applied when loading progress. |
| `data/progress.json` | Per-card `interval_days` and `due_date` (`YYYY-MM-DD`), keyed by `tocflN:漢字` or `extra:漢字`. Older keys are rewritten on load (`progress.json.bak`). |

Word counts (unique Traditional, first official band wins): Extra 496 · L1 339 · L2 477 · L3 1091 · L4 2200 · L5 2661 (7264 total). New official words use CC-CEDICT glosses; inherited HSK rows keep their examples.

## Layout (code)

```text
src/main.rs          load deck + progress, run TUI
src/models/          Card, Deck, progress, next_interval
src/app/tui/         terminal loop, App state, keys, view
src/utils/           calendar today()
```

## License

MIT. See [LICENSE](LICENSE).
