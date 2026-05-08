# Kokoro — Art Direction

> Visual style: Retro pixel art with a curated 26-color palette organized in ramps. Game Boy heritage meets cozy ghibli aesthetic, kawaii in cubs maturing to characterful adults.
>
> **Métricas quantitativas** (proporções, ratios kindchenschema, mapeamento por espécie por estágio) vivem em `docs/aesthetic-targets.md`. Este doc cobre a direção/filosofia; o outro cobre os números.

## Style Reference

Inspirado em pixel art Game Boy/SNES-era + simplificações Studio Ghibli + kawaii character design moderno (kindchenschema explícito em cubs). Tudo computado em Rust via DSL — no pre-made sprites, no external art tools, no AI generation in pipeline.

Key characteristics:
- **Pixel art retrô**: nearest-neighbor, sem AA suave, pixel duro
- **Paleta curada de 26 cores em ramps**: nenhuma cor gerada via HSL/genome direto. Genome escolhe **slot e ramp**, não RGB.
- **Flat rectangles em UI**: no rounded corners, 2px borders, Game Boy button style
- **Pixel font**: Press Start 2P
- **Strong silhouettes**: cada espécie reconhecível pelo shape + ramp principal
- **Runtime rendering via DSL**: cada pixel computado a partir de genome, species, mood, growth stage e `kawaii_factor` contínuo
- **Lifecycle de estilo**: Cub kawaii máximo → Adult cozy ghibli → Elder dessaturado/wise. `kawaii_factor: 1.0 → 0.0 → 0.2` controla proporções, blush, detalhe interno e saturação.

## A Paleta Master (26 cores em 9 ramps)

Mantém as 6 cores originais (Cream, Near Black, Red, Teal, Gold, Orange) como família central. Expande com ramps de shadow/highlight e bandas adicionais (earths, greens, biolume).

### Darks / Outlines (4)
| Name | Hex | Uso |
|---|---|---|
| NearBlack | `#1B130D` | outline padrão warm, eyes universal |
| DeepBrown | `#3B2418` | outline pra criaturas warm (Moluun/Pylum/Nyxal) |
| DeepTeal | `#0A2A2D` | outline pra criaturas cool (Skael) |
| Charcoal | `#2A2520` | sombras profundas em background |

### Cream / Highlights (3)
| Name | Hex | Uso |
|---|---|---|
| Cream | `#D9C7AE` | belly, background neutro, glint pequeno |
| CreamLight | `#F0E2C8` | glint forte, bioluminescência peak |
| OffWhite | `#FBF4E2` | sparkle pontual, eye glint |

### Warm earths (4)
| Name | Hex | Uso |
|---|---|---|
| Tan | `#C49870` | skin secundário, ear inner |
| Brown | `#8B5A3A` | fur shadow, soil |
| BrownDark | `#5A3622` | fur deep shadow, bark |
| Sand | `#E5C896` | background quente |

### Gold ramp (3)
| Name | Hex | Uso |
|---|---|---|
| GoldDark | `#A07803` | Moluun body shadow |
| Gold | `#D9A404` | Moluun body, happiness stat UI |
| OrangeBright | `#F08828` | accent (penas, escamas brilhantes), Moluun highlight |

### Orange ramp (3)
| Name | Hex | Uso |
|---|---|---|
| OrangeDark | `#A04C03` | Pylum body shadow |
| Orange | `#D96704` | Pylum body, action accent UI |
| OrangeLight | `#F0883D` | Pylum belly highlight |

### Red ramp (3)
| Name | Hex | Uso |
|---|---|---|
| RedDark | `#A00930` | Nyxal body shadow, Skael thermal pit, danger UI shadow |
| Red | `#D90D43` | Nyxal body, danger/hunger stat UI |
| CoralPink | `#F06B85` | cheek blush universal (Cubs/Young), Nyxal highlight |

### Teal ramp (3)
| Name | Hex | Uso |
|---|---|---|
| TealDark | `#014045` | Skael body shadow |
| Teal | `#016970` | Skael body, energy stat UI |
| CyanBright | `#4DC3CC` | bioluminescência fria (Nyxal/Skael), water highlight |

### Forest greens (3)
| Name | Hex | Uso |
|---|---|---|
| ForestDark | `#1F3A28` | Verdance bioma deep |
| Forest | `#3D6E45` | Verdance bioma mid |
| Sage | `#87A878` | Verdance highlight, fresh leaves |

### Regra absoluta

Nenhum brush ou primitiva pode invocar `Color::rgb(...)` com valor cru. Tudo passa pelo enum `Palette` (definido em `aesthetic-targets.md` §1). Compilador garante: zero cor fora dessa lista. Genome modula **qual ramp** uma criatura ocupa e **qual posição na ramp** — nunca cores cruas.

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

**Decisão (2026-05-08):** caminho oficial é **DSL procedural em Rust** — uma biblioteca de primitivas, brushes e composições que produz pixel art a partir de RON declarativo + genome. Nada de Blender, nada de AI generation no pipeline, nada de sprite sheets pré-renderizados.

Justificativa:
- Cada criatura única no nível do pixel (genome → parâmetros → pixels)
- Hot-reload via RON: editar visuais sem recompilar
- Aligna 100% com a filosofia "tudo procedural" já estabelecida (zero `.ogg`, backgrounds em código)
- Vira diferencial real e ensinável (Parte VIII do ebook)

Plano completo: ver `aesthetic-targets.md` §3 (especificações por feature) + roadmap das 9 semanas (Fases 0-7.5).

## Color Palette Philosophy

A paleta master de 26 cores acima é a única fonte de verdade. Por bioma, cada espécie ocupa uma **assinatura de paleta** (subset de 6-10 cores), definida em `aesthetic-targets.md` §4.

**Eyes universais:** todas espécies usam `NearBlack` puro com glint `OffWhite`. Bead-like solid, sem pupila/íris. Maximiza contraste em qualquer body color, mantém expressividade no pixel.

**Bioluminescência:** `CyanBright` como base + `CreamLight` como peak. Pixels sólidos cintilando com timing — nunca halo/glow contínuo.

**Cheek blush (kawaii signature):** `CoralPink`, presente em Cubs/Young, ausente em Adult/Elder (matura via `kawaii_factor`).

**Shading:** sem facet shading 3D-style. Pixel art retrô usa **flat color + 1 tom shadow + 1 tom highlight** por região, transição dura. Nada de gradient ramp interno.

## Integration with Existing Systems

O rig system existente já suporta tudo que a DSL precisa:
- **Anchor points**: posições normalizadas [-1,1] por body part
- **Gene offsets**: variação individual em proporções
- **Species templates**: definem quais partes existem e propriedades visuais
- **Mood-reactive parts**: eyes/mouth mudam com FSM

O que muda com a DSL:
1. **Drawing functions**: `draw_circle`/`draw_ellipse` saem. Entram primitivas com intenção (`bumpy_dome`, `tapered_tail`, `feathered_wing`) e brushes (`eye_with_glint`, `fur_edge`, `bioluminescent_speck`).
2. **RON-driven config**: parâmetros visuais por espécie em `assets/species/X.ron`, hot-reloadable
3. **`kawaii_factor` lifecycle**: parâmetro contínuo modula proporções e detalhe entre stages
4. **Editor in-game**: F12 panel expandido com sliders/dropdowns pra cada parâmetro, "Save preset" escreve de volta no RON

Roadmap detalhado: ver fases 0-7.5 em `aesthetic-targets.md` (9 semanas focadas).

## Priority for New Species Design

1. Refine existing 4 species with low-poly style and alien upgrades
2. Add 5th species: **Insectoid** (underground, 6 legs, compound eyes)
3. Add 6th species: **Amphibian** (swamp, translucent skin, external gills)
4. Future: Arboreal, Colonial (more exotic body plans)
