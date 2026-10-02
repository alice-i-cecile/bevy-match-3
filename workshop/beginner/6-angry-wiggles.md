# Angry Wiggles

**Goal:** Make the two gems involved in an _invalid_ swap shake angrily, so
the player gets visual feedback in addition to the existing audio cue.

**You'll learn:** making a component for short-lived animations,
how to drive position from elapsed time, and how to keep two animation
systems from fighting over the same `Transform`.

## Where to look

- `src/game_logic.rs`: `process_swap` (line ~80) detects the invalid case
  and reverts the swap. This is where you attach the marker.
- `src/gems.rs`: `Falling` (line ~125 onward) is the existing marker-driven
  animation. Mirror its structure.

## Steps

1. **Define the component.** In `gems.rs`:

   ```rust
   #[derive(Component)]
   pub struct Wiggling {
       pub elapsed: f32,
   }
   ```

2. **Attach it on invalid swap.** In `process_swap`, you already have the two
   `GridPos` values, `pa` and `pb`. Look up the entities and insert
   `Wiggling`:

   ```rust
   if let (Some(ea), Some(eb)) = (grid.entity_at(pa), grid.entity_at(pb)) {
       commands.entity(ea).insert(Wiggling { elapsed: 0.0 });
       commands.entity(eb).insert(Wiggling { elapsed: 0.0 });
   }
   ```

   Add `mut commands: Commands` to the `process_swap` signature.

3. **Animate it.** Add an `animate_wiggling_gems` system to `GemsPlugin` next
   to `animate_falling_gems`. Pick numbers that feel right; these are
   reasonable starting values:

   ```rust
   const WIGGLE_FREQ: f32 = 35.0;       // radians per second
   const WIGGLE_AMPLITUDE: f32 = 6.0;   // pixels
   const WIGGLE_DURATION: f32 = 0.35;   // seconds

   fn animate_wiggling_gems(
       mut commands: Commands,
       time: Res<Time>,
       mut query: Query<(Entity, &GridPos, &mut Wiggling, &mut Transform)>,
   ) {
       for (entity, pos, mut wiggle, mut transform) in &mut query {
           wiggle.elapsed += time.delta_secs();
           let target = pos.to_world();

           if wiggle.elapsed >= WIGGLE_DURATION {
               transform.translation = target;
               commands.entity(entity).remove::<Wiggling>();
               continue;
           }

           // Damping factor falls to 0 at the end so the wiggle dies out.
           let damping = 1.0 - wiggle.elapsed / WIGGLE_DURATION;
           let dx = (wiggle.elapsed * WIGGLE_FREQ).sin() * WIGGLE_AMPLITUDE * damping;
           transform.translation.x = target.x + dx;
           transform.translation.y = target.y;
       }
   }
   ```

## Stretch

- Add a small rotation to the wiggle, not just translation:
  `transform.rotation = Quat::from_rotation_z(angle)`.
- Flash the gem briefly red before it wiggles, by writing to `Sprite::color`.
- Screen-shake on multiple wrong moves in a row: add a resource holding the
  current number of wrong moves in a row, and insert `Wiggling` on the
  `Camera2d` entity, or on a parent of the whole board.
