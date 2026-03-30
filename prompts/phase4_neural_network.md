# Phase 4 — Neural Network Architecture

## Goal

Replace part of the hardcoded FSM mood rules with a small local neural network
that learns each owner's specific interaction patterns. The creature should
develop a unique personality over time — adapting to when you feed it, how
often you play, your daily schedule.

## Architecture Overview

```
Inputs (12 neurons)          Hidden (8 neurons)        Outputs (7 neurons)
─────────────────           ──────────────────        ─────────────────
hunger         ─┐                                    ┌─ Happy
happiness      ─┤           ┌─ h1 ─┐                ├─ Hungry
energy         ─┤           ├─ h2 ─┤                ├─ Tired
health         ─┼──────────►├─ h3 ─┼───────────────►├─ Lonely
hour_of_day    ─┤           ├─ h4 ─┤                ├─ Playful
ticks_since_fed─┤           ├─ h5 ─┤                ├─ Sick
ticks_since_play┤           ├─ h6 ─┤                ├─ Sleeping
curiosity      ─┤           ├─ h7 ─┤                └─────────────────
appetite       ─┤           └─ h8 ─┘                  (softmax → mood)
resilience     ─┤
circadian      ─┤
loneliness_sens─┘
```

**Type**: Feedforward neural network (MLP)
**Size**: 12 → 8 → 7 (~167 weights + 15 biases = 182 parameters)
**Activation**: ReLU (hidden), Softmax (output)
**Inference**: ~182 multiply-adds per tick — negligible CPU cost

## Training Strategy

### Data Collection (already in place)

Events are already logged to SQLite via `save::log_event()`:
- Feed, Play, Sleep actions with timestamps
- Mood state at each tick
- Vital stats snapshots (via autosave)

### Training Signal

The network learns to predict **what mood the creature should be in**,
given the current state. The training labels come from two sources:

1. **Player corrections** — when the player feeds a hungry creature, that's
   an implicit signal that "Hungry was the right mood to show" (it got attention).
   When the player ignores a mood for a long time, that mood was less effective.

2. **Outcome reward** — moods that lead to player interaction within 30 ticks
   get a positive label. Moods that get ignored for 120+ ticks get suppressed.

### Training Schedule

- Train locally every 500 ticks (~8 minutes of real time)
- Use the last 2000 events as training data (sliding window)
- 10 epochs per training session, learning rate from genome's `learning_rate` gene
- Weights saved to SQLite alongside the genome/mind data

## Integration with Existing FSM

The network doesn't replace the FSM entirely — it modulates it:

```rust
// Pseudocode
let fsm_mood = mind.fsm_decide_mood(&genome);
let nn_probs = network.forward(&input_vector);

// Blend: FSM has veto power for critical states (Sick, Sleeping)
let final_mood = if fsm_mood == Sick || fsm_mood == Sleeping {
    fsm_mood  // Safety: don't let NN override critical states
} else {
    // Pick the NN's top choice, weighted by confidence
    let nn_mood = nn_probs.argmax();
    if nn_probs[nn_mood] > 0.6 {
        nn_mood  // NN is confident
    } else {
        fsm_mood  // Fall back to FSM when NN is unsure
    }
};
```

## Dependencies

| Crate | Purpose | Size |
|-------|---------|------|
| `candle-core` | Tensor operations | ~2MB |
| `candle-nn` | Layer definitions (Linear, ReLU) | ~500KB |

Alternative: implement the 182-parameter network manually with plain arrays
(no dependencies, ~100 lines of Rust). Recommended for v1 since the network
is tiny.

## Files to Create

| File | Purpose |
|------|---------|
| `src/brain/mod.rs` | Network struct, forward pass, weight storage |
| `src/brain/train.rs` | Training loop, data preparation from SQLite |
| `src/brain/plugin.rs` | Bevy plugin: periodic training + mood inference |

## Implementation Order

1. Add `ticks_since_fed` and `ticks_since_play` tracking to `Mind`
2. Implement the forward pass (manual, no dependencies)
3. Wire inference into `update_mood()` as a blend with FSM
4. Implement training from SQLite event log
5. Save/load weights via persistence system
6. Add `candle` only if manual implementation proves too limiting

## Emergent Behaviour Examples

After training on a specific owner's patterns:

- **Night owl owner** → creature learns to be Playful in the evening
  (when interaction is most likely), even if stats suggest Tired
- **Morning feeder** → creature shows Hungry right before the usual
  feed time, "anticipating" the routine
- **Neglectful owner** → creature learns that Lonely gets the most
  response, so it shows Lonely more often (manipulative! but emergent)
