# Kokoro — Art Direction

> Visual style: Retro pixel art with a strict 6-color palette. Game Boy aesthetic meets biological simulation.

## Style Reference

Inspired by Game Boy-era pixel art and Noita-style runtime rendering. Everything computed in Rust — no pre-made sprites, no external art tools.

Key characteristics:
- **Pixel art**: 64×64 creature canvas, 16×16 effects, nearest-neighbor upscaling
- **6-color palette**: Cream, Near Black, Red, Teal, Gold, Orange — all UI, creatures, and backgrounds use only these colors (with depth variations for lighter/darker shading)
- **Flat rectangles**: no rounded corners, 2px borders, Game Boy button style
- **Pixel font**: Press Start 2P for all text
- **Strong silhouettes**: each species instantly recognizable by shape + color
- **Runtime rendering**: every pixel computed from genome, species, mood, and growth stage
- **Depth through value**: each species color has lighter (belly) and darker (accent) variants for visual volume

## The Palette

| Name | Hex | Creature | UI Role |
|------|-----|----------|---------|
| Cream | #D9C7AE | Belly highlights | Background, panels, button fills |
| Near Black | #1B130D | All eyes, mouths | Text, borders, outlines |
| Red | #D90D43 | Nyxal body | Danger, hunger stat |
| Teal | #016970 | Skael body | Energy stat |
| Gold | #D9A404 | Moluun body | Happiness stat |
| Orange | #D96704 | Pylum body | Action accent |

## Body Plans (Anatomy-Grounded Alien Biology)

Every body plan exists because of evolutionary pressure in its biome.

### Bipedal (upright, 2 legs)
- **Moluun**: round body, short thick legs, arms tucked close. Built for stability in dense forest lattices.
  - 2 large eyes (forward-facing, primate-like depth perception)
  - Small rounded ears that swivel independently
  - Padded feet for silent movement

### Quadruped (4 legs)
- **New: Terrestrial hunter/grazer type**: low center of gravity, built for endurance or sprinting
  - Could be canine-like, ungulate-like, or feline-like
  - 4 eyes (2 forward for depth, 2 lateral for wide field — like a spider/prey hybrid)
  - Possible: thick armored plates (pangolin-like) or fur mane

### Avian (winged, 2 or 4 wings)
- **Pylum**: egg-shaped torso, 2 broad wings, tail feathers
  - Upgrade: **4 wings** — front pair for lift/glide, back pair for maneuvering (like a dragonfly)
  - 2 main eyes + 2 small dorsal eyes (UV-filtering, for high-altitude light)
  - Sharp articulated beak with gradient plumage

### Serpentine/Crawler (no legs, slides/crawls)
- **Skael**: elongated, scaled body
  - Upgrade: instead of legs, muscular underbody plates for locomotion (like a snake + centipede hybrid)
  - 2 main eyes + 2 heat-sensing pit organs (like pit vipers) — visible as dim red dots
  - Dorsal crests that flush with bioluminescent color

### Cephalopod (tentacles, soft body)
- **Nyxal**: bulbous mantle, 4 tentacles
  - Upgrade: **6 tentacles** (4 locomotion + 2 shorter manipulation arms)
  - 4 eyes arranged in a diamond pattern (full 360° awareness — deep-sea adaptation)
  - Chromatophore skin that shifts color with mood (visible as facet color changes)

### New Body Plans to Consider

| Body Plan | Inspiration | Biome | Alien Twist |
|-----------|------------|-------|-------------|
| **Insectoid** | Beetles, mantis | Underground fungal networks | 6 legs, compound eyes (rendered as faceted gem), mandibles, chitinous armor |
| **Amphibian** | Axolotl, frogs | Swamp/wetland transition zones | 4 legs + external gill fronds, translucent skin showing internal glow, vocal sac for communication |
| **Arboreal** | Monkey, sloth | Canopy (above the Verdance) | 4 arms + prehensile tail, 3 eyes (triangular arrangement), elongated fingers for gripping |
| **Colonial** | Coral, jellyfish | Shallow tidal pools | Not a single creature but a colony — multiple small units that combine into one body, shared nervous system |

## Alien Features (Biologically Justified)

Each "alien" feature has a real-world analog and evolutionary purpose:

| Feature | Real-World Analog | Purpose | Visual |
|---------|------------------|---------|--------|
| **4 eyes** | Spiders (8 eyes) | Predator detection, 360° awareness | 2 large front + 2 small side, different glow colors |
| **4 wings** | Dragonflies | Independent control = extreme agility | Front pair larger, back pair for stabilization |
| **6 tentacles** | Octopus (8) | Multitasking, tool use, locomotion | 4 long + 2 short, different colors for each pair |
| **Bioluminescence** | Deep-sea fish, fireflies | Communication, lure prey, camouflage | Pulsing glow on specific body parts (kokoro-sac!) |
| **Chromatophores** | Cuttlefish, chameleons | Mood display, camouflage, social signaling | Facet colors shift — geometric color patches animate |
| **External gills** | Axolotl | Aquatic respiration, sensory organ | Feathery fronds that wave with breathing rhythm |
| **Compound eyes** | Insects | Motion detection, wide-angle vision | Rendered as faceted gem surfaces (fits low-poly style perfectly!) |
| **Thermal pits** | Pit vipers | Infrared sensing (prey detection in dark) | Dim red dots near the eyes, pulse when sensing heat |
| **Vocal sac** | Frogs | Long-distance communication | Inflates/deflates with breathing system |
| **Prehensile tail** | Monkeys, chameleons | Extra limb for gripping, balance | Curls and uncurls, reacts to mood |

## How to Achieve This Visual Style

### Option A: Procedural Low-Poly (Code-Generated)
Current approach evolved. Instead of pixel ellipses, generate **triangulated polygonal meshes**.

Pros:
- Infinite variation per individual (gene-driven vertex positions)
- No external art tools needed
- Fits the "learn Rust" philosophy

Cons:
- Hard to get the "warmth" of hand-crafted art
- Triangle subdivision algorithms are complex

### Option B: Blender → Sprite Sheet (Recommended for quality)
Model each species as a simple low-poly 3D model in Blender, render orthographic views as sprite sheets.

Pros:
- Best visual quality
- Easy to create mood variants (change expression, pose)
- Low-poly models are fast to make (50-200 triangles per creature)
- Free tool (Blender is open source)

Cons:
- Requires learning Blender basics
- Can't generate infinite variation (would need to render per-genome — possible but complex)

### Option C: Hybrid (Best of Both)
- **Base shapes**: Hand-drawn or Blender-rendered low-poly sprites for the "template"
- **Variation**: Code applies genome-driven modifications (color tint, scale adjustments, part positioning via the rig system)
- **Animation**: Code-driven (current system already handles this — breathing, species behaviors)

This is the recommended approach. The rig system already supports per-genome variation. You'd just need better base art.

### Option D: AI-Assisted Generation
Use AI image generation to create the base sprites in the low-poly style, then clean up manually.

Pros: Fast prototyping
Cons: Inconsistent style, copyright concerns, hard to animate

## Color Palette Philosophy

### Base Colors (per biome)
- **Verdance (Moluun)**: warm creams, soft browns, forest greens
- **Highlands (Pylum)**: golds, ambers, warm whites, sky blues
- **Shallows (Skael)**: cool greens, jade, earthy browns
- **Depths (Nyxal)**: deep purples, dark blues, bioluminescent cyan accents

### Alien Accents
- Bioluminescent glow: cyan, magenta, soft green — always on specific organs/features
- Eyes: all species share deep black eyes — a universal Kobara trait (etharin pigment adaptation). Dark bead-like eyes maximize contrast against any body color, enhancing expressiveness at pixel-art scale.

### Facet Shading Rules
- Top facets: brightest (light from above)
- Side facets: medium tone
- Bottom facets: darkest (shadow)
- Accent facets: glow color on specific parts (eyes, kokoro-sac area, tentacle tips)

## Integration with Existing Systems

The current rig system already supports everything needed:
- **Anchor points**: vertex positions for each body part (already normalized [-1,1])
- **Gene offsets**: each individual has slightly different proportions
- **Species templates**: define which parts exist and their visual properties
- **Mood-reactive parts**: eyes/mouth change with mood

What needs to change:
1. **Sprite assets**: replace current pixel art with low-poly style sprites
2. **More anchor points**: current rigs have 6-8 parts. Alien bodies need 12-16 (extra eyes, wings, limbs)
3. **New body plan rigs**: quadruped, serpentine, insectoid rigs in addition to bipedal/avian/cephalopod
4. **Animation**: more complex part movement (4 wings need phase-offset flutter, 6 tentacles need wave patterns — we already have this pattern for Nyxal)

## Art Pipeline (Step by Step)

1. **Sketch** silhouettes for each species (pen and paper or digital)
2. **Choose method**: Blender low-poly or enhanced procedural generation
3. **Create one species** in the new style as proof of concept (start with Moluun — simplest shape)
4. **Validate** it works with the rig system (does the rig still resolve correctly?)
5. **Iterate** — apply to remaining species, then design new ones
6. **Animate** — verify breathing, species behaviors, and mood effects still look good

## Priority for New Species Design

1. Refine existing 4 species with low-poly style and alien upgrades
2. Add 5th species: **Insectoid** (underground, 6 legs, compound eyes)
3. Add 6th species: **Amphibian** (swamp, translucent skin, external gills)
4. Future: Arboreal, Colonial (more exotic body plans)
