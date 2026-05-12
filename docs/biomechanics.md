# Biomechanics — the physical laws of Kokoro's world

Every joint of every kobara obeys the **same** physical law. No
animation cycles, no canned poses, no "decorative" properties. A creature
moves because something physical caused it to move — a muscle contracted,
gravity pulled, something hit it. This document is the source of truth
for that model.

The biomechanics layer is split across two crates so each piece can be
tested headlessly:

- [`kokoro-rig`](../kokoro-rig/) — bones + joints + physics integrator.
  The mechanical substrate. Bevy-free.
- [`kokoro-body`](../kokoro-body/) — muscles + nerves + actuation
  pipeline. The soft-tissue layer above the rig. Also Bevy-free.

The mind lives in `src/mind/` and produces *intents* for nerves. The
renderer in `src/visuals/` reads world positions from the rig and paints
pixels — it never invents motion.

---

## 1. The five layers

Real anatomy is layered, and Kokoro mirrors that layering exactly so each
kind of pathology lands on a separate struct:

```text
Mind   (intent: "flick the tail", "tuck legs")
  │
  ▼
Nerve  (carries signal with latency, attenuation, health)
  │
  ▼
Muscle (force = max_force × activation × (1 − fatigue))
  │
  ▼
Joint  (sums torques: muscles + spring + friction + gravity + contact;
        integrates angle each dt)
  │
  ▼
Bone   (rigid structural element; FK propagates joint angles to world
        positions for the renderer)
```

Each layer is its own struct, never folded into another. A torn ligament
weakens a `Joint`; an atrophied muscle weakens a `Muscle`; a damaged
nerve weakens a `Nerve`. The layers don't leak into one another.

---

## 2. Bone — rigid structural element

`kokoro_rig::Bone` is purely geometric. It does not bend, does not
animate, does not have stiffness of its own.

Fields that matter for physics:

| Field | Meaning |
|---|---|
| `length` | Distance from base to tip (pixels). |
| `tissue: BoneTissue` | `mass: f32` today; future fields (density, fracture state, marrow) live here. |
| `genome_length_scale`, `genome_width_scale` | Per-creature genome modulation. |

The integrator derives moment of inertia from `mass` and `length` using
the textbook rod-about-end formula:

```text
I = (1/3) × mass × length²
```

---

## 3. Joint — where motion happens

`kokoro_rig::Joint` describes a single articulation between a bone and
its parent. Fields:

| Field | Meaning |
|---|---|
| `kind: JointKind` | `Hinge` (default), `Ball` (reserved for 3D), `Fused` (no DoF). |
| `rest_angle` | The neutral position ligaments pull toward (radians). |
| `range_min`, `range_max` | Hard ROM limits relative to `rest_angle`. |
| `ligament_k` | Spring constant for the passive ligament restoring torque (N·m/rad). |
| `surface: JointSurface` | `friction: f32` today; future fields (cartilage condition, synovial fluid) live here. |

`JointState` holds the time-varying part: `angle` and `angular_velocity`.
The integrator reads `Joint` constants and writes `JointState`.

---

## 4. The universal law (per joint, per frame)

The function `physics::integrate_joint` implements the law. It is a pure
function — same inputs, same outputs, every time.

```text
total_torque
  = muscle_torque                              # from kokoro-body
  + contact_torque                             # from collisions
  + gravity_torque                             # from world gravity × CoM
  − ligament_k × (angle − rest_angle)          # passive spring
  − friction    × angular_velocity             # cartilage friction

α        = total_torque / I
velocity ← velocity + α × dt                   # semi-implicit Euler
angle    ← clamp(angle + velocity × dt, rest+min, rest+max)
```

When the ROM clamp hits, the velocity in the violating direction is
zeroed so the joint doesn't keep grinding against the wall.

Semi-implicit Euler is chosen over explicit Euler because it stays
stable for high spring constants (stiff joints like the spine).

---

## 5. Muscle — active force producer

`kokoro_body::Muscle` connects two bones via attachment points. Each
attachment holds a `bone: BoneId` and a `lever_arm: f32` (pixels from the
joint axis to the attachment).

Force production:

```text
current_force = max_force × activation × (1 − fatigue)
```

State that evolves over time:

- `activation: f32` — chases the nerve signal at `contraction_rate` per
  second. Models the finite speed of myosin cross-bridge cycling.
- `fatigue: f32` — climbs proportional to activation, recovers
  proportional to (1 − activation). Caps the effective force.

The muscle composition lives in an enum so we can grow into fibre-level
detail without breaking the API:

```rust
pub enum MuscleComposition {
    Aggregate { max_force: f32 },
    // future: Fibers(Vec<Fiber>),
}
```

---

## 6. Antagonist pairs

Real joints are driven by **pairs** of muscles on opposite sides. The
flexor pulls one way, the extensor pulls the other. `MusclePair` holds
both. The net torque on the joint is:

```text
net = extensor_force × extensor_lever − flexor_force × flexor_lever
```

Sign convention: positive torque rotates the joint in the +angle
direction (clockwise in image space, since y grows down).

Both muscles in a pair can be active at once — that's *co-contraction*,
which produces stiffness without motion (think of locking your elbow
straight by tensing both biceps and triceps).

---

## 7. Nerve — signal pathway

`kokoro_body::Nerve` is the connection between the mind and a muscle.
Today the pathway is a fixed-latency ring buffer plus a multiplicative
attenuation:

```text
delivered = buffer[t − latency] × attenuation × health
```

The future variant `NervePathway::Network(...)` will simulate individual
neurons; adding the variant is a forward-compatible change because the
enum is the only thing callers pattern-match on.

`health: f32` is a multiplier that the genome initialises near 1.0;
injury or age decrement it.

---

## 8. The actuation pipeline (one frame)

`kokoro_body::actuate` runs one tick of the pipeline for one joint:

1. The mind emits a `PairIntent { flexor, extensor }`.
2. Each nerve delivers a delayed, attenuated signal to its muscle.
3. Each muscle ramps its activation toward the signal, updating fatigue.
4. The muscle pair sums into a net torque.

That torque is then passed to `kokoro_rig::Skeleton::step_physics`, which
calls `integrate_joint` for every joint with the supplied applied
torques. `Body::step` ties the whole pipeline together for an entire
creature.

---

## 9. Genome interaction

The genome modulates physical parameters at creature creation time. It
**never** sets state directly — the body is created with parameters that
reflect the genes, and from then on physics runs.

Examples of gene → physical-parameter mappings (proposed):

| Gene | Modulates |
|---|---|
| `resilience` | `Muscle::recovery_rate`, `Nerve::health` |
| `appetite` | `Bone::tissue.mass` (body mass scaling) |
| `curiosity` | `Joint::range_min/max` slightly widened (more exploratory motion) |
| species traits | Average `ligament_k`, `friction`, `max_force` per body part |

---

## 10. Forward-compatibility checklist

When extending the model, follow these rules:

1. Add fields to existing structs (`JointSurface`, `BoneTissue`,
   `PairIntent`) — don't introduce parallel hierarchies.
2. Add variants to existing enums (`MuscleComposition`, `NervePathway`,
   `JointKind`) — don't fork the type.
3. Add new layers above the mind, not between the mind and the body.
4. Keep the universal law pure: never special-case a creature, a body
   part, or a mood inside `integrate_joint`.

If a new feature needs a different law, it belongs in a higher layer
(mind / behavior / environment), not inside the integrator.
