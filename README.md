# Bevy Match-3 (RustWeek 2026 Workshop)

A bog-standard Match-3 game built with [Bevy](https://bevy.org/) for [RustWeek 2026](https://2026.rustweek.org/).
Polished and playable, but intentionally simple so workshop participants can read, understand, and extend it.

## Expected Background

- Some programming experience in any language
  - You understand for loops, functions, modules, building and running programs
  - You're comfortable reading compiler errors
  - You have written at least one small project of your own in any language
- Basic Rust exposure
  - You can read function signatures
  - You know what `struct`, `enum`, and simple traits are
- No prior Bevy or game development experience required
- No math or art skills needed

## Understanding this Codebase

This project is not intended to be read alone!
For the most part, it is documented and commented as if it were a real, production game.
You will have to supplement it with other learning materials.

Still, we make a few concessions.
To make this more useful as a learning tool, `src/main.rs` provides a high-level codebase overview,
and `game_logic.rs` contains an ASCII gameplay diagram.
In a few places, key learning points are explicitly called out to explore the implications of critical Bevy patterns.

When you encounter a concept you are not familiar with, look it up!
Check out Bevy's [Book](https://bevy.org/learn/book/intro/)
for background information on the key patterns used here.
To look up information on specific APIs, use `cargo doc`, use your IDE's tooltips, or visit Bevy's [docs.rs page](https://docs.rs/bevy/latest/bevy/).
When you're ready to add new features, the [Bevy examples](https://github.com/bevyengine/bevy/tree/latest/examples)
are very helpful for understanding Bevy's tools in a simplified but working context.

If you are completing this workshop in person,
please do not hesitate to ask for help from the instructors or your peers.
If you are exploring this remotely, your best bet is the [Bevy Discord](https://discord.gg/bevy).
It's full of friendly, helpful people who were in exactly the same position as you not long ago.

Indulge your curiosity: the point is to explore and learn!

## Getting Started

- **Rust** (stable, 1.92 or later), install via [rustup](https://rustup.rs/).
- On Linux you may need extra system libraries for audio/windowing.
  See [linux_dependencies.md](https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md).

```bash
git clone https://github.com/alice-i-cecile/bevy-match-3
cd bevy-match-3
cargo run
```

The first build compiles Bevy and dependencies, so it can take a few minutes.
Subsequent builds are much faster due to incremental compilation.

### Dev Mode

Run with the `dev-mode` feature to enable Bevy's built-in debug overlays
(UI layout boxes, entity outlines, etc.):

```bash
cargo run --features dev-mode
```

## How to Play

Match 3+ gems of the same color in a row or column to clear them.
Cleared gems are replaced by new ones falling from above.
New matches from falling gems cascade automatically until the board stabilizes.
If no valid moves remain, the game ends.

**Score:** 10 points per cleared gem.

## Controls

| Action              | Keyboard                  | Mouse          | Gamepad           |
| ------------------- | ------------------------- | -------------- | ----------------- |
| Move cursor         | Arrow keys / WASD         | —              | D-Pad             |
| Select / confirm    | Space or Enter            | Left click     | South (A / Cross) |
| Deselect            | Select the same gem again | Click same gem | South on same gem |
| Restart (game over) | Enter or Space            | —              | South (A / Cross) |

## Workshop Ideas

Some directions to explore once you're comfortable with the codebase.

### Beginner

- **New gem colour:** add a new colour variant to `GemType`.
- **Bonus points for large clears:** award more points when many gems are cleared in one step.
- **Move counter:** track and display the number of swaps made.
- **High score:** persist the best score to disk across sessions.
- **Background:** find or make a background image, and place it behind the game board.
- **Angry wiggles:** make the gems shake angrily when an invalid move is made.
- **New Font:** find a better font, add it to the assets folder, and use it.

### Intermediate

- **Scoring multipliers:** award bonus points for cascade chains.
- **Timer mode:** add a countdown and a game-over screen. For a different flavour, try a limited moves budget instead.
- **Background music:** add a looping music track that plays during a game.
- **Pause menu:** pause the game on Escape and show a menu.
- **Settings:** add a settings menu to control volume, grid size and number of gem colors.

### Advanced

- **Gem sprites:** replace the colored rectangles with sprites of your choice, and update the selection appearance to match.
- **Special gems:** add a bomb or line-clear gem type with a unique clearing effect.
- **Shuffle when stuck:** detect when no valid moves remain and shuffle the board.
- **Particle effects:** spawn short-lived particle entities when gems are cleared.
- **Hint system:** after a few seconds of inactivity, highlight a valid move.
- **Game AI:** add a mode where the game makes matches automatically.
