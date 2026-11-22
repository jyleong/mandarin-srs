# Mandarin SRS

A personal terminal spaced-repetition app for Taiwan Mandarin vocabulary: Traditional headwords, English glosses, one review direction.

## Language

**Deck**:
The shipped, read-only collection of Cards (Novice, TOCFL, Extra JSON). Learners do not add or edit Cards in the app.
_Avoid_: library, pack, user deck

**Card**:
One vocabulary item: a Headword, Pinyin, Meaning, and an optional Example pair.
_Avoid_: note, flashcard, phrase card

**Headword**:
The Traditional 漢字 string shown as the prompt. It is the identity of the Card together with its Level, including official TOCFL spellings such as `小孩(子)`.
_Avoid_: word, character, chinese, 汉字 (Simplified)

**Pinyin**:
Hanyu Pinyin for the Headword. Hidden until Reveal. A lenient Pinyin guess is a legal Check (tones, numbering, spaces, and `ü`/`v`/`u:` ignored; `lue`/`nue` count as `lüe`/`nüe` after l/n; `lu` does not match `lǚ`).

**Meaning**:
The English gloss the learner types. Multiple senses are separated by `;`.
_Avoid_: definition, translation, answer key

**Example**:
An optional Traditional usage sentence plus English, shown on Reveal. It is not part of the typed answer.
_Avoid_: sample, sentence card

**Level**:
One of eight exclusive vocabulary bands: Novice 1, Novice 2, TOCFL 1–5, Extra. A Card belongs to exactly one Level.
_Avoid_: HSK, TOCFL 6, deck file, band

**Extra**:
The Level for former HSK Headwords that have no twin on the official TOCFL lists.
_Avoid_: misc, other, leftover (as a product name)

**Review**:
One attempt at a Card: show the Headword, type a Check, then Grade how well it was known.
_Avoid_: quiz, test, drill

**Check**:
Whether the typed guess matches a listed Meaning sense **or** the Card’s Pinyin. It only drives the Reveal ✓/✗. It does not set the Grade.
_Avoid_: score, answer key, auto-grade

**Session**:
One shuffled batch of Cards at a single Level, sized 10, 20, 30, or 50.
_Avoid_: lesson, round, deck

**New**:
A Card with no Progress row yet. At most ten New Cards may be introduced per calendar day, shared across all Levels.
_Avoid_: unseen, unseen-today

**Study-ahead**:
A Card that already has Progress but is not due yet, used to fill leftover Session slots when due Reviews and allowed New Cards run out.
_Avoid_: extra session, cram, preview

**Grade**:
Again (forgotten: due today, requeued in this Session), Good (interval × 2), or Easy (interval × 4). The Grade, not the typed check, updates Progress.
_Avoid_: Hard, score, rating

**Progress**:
Per-Card learning state: interval, due date, and the day it was first graded. Keyed `{prefix}:{Headword}`.
_Avoid_: save file, history, stats
