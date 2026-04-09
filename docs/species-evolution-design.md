# Species Evolution Design — Visual Progression by Growth Stage

> Every species transforms dramatically from egg to elder.
> Alien features emerge gradually, all biologically justified.

## Growth Stages

| Stage | Age (ticks) | Scale | Proportion (head:body) |
|-------|-------------|-------|----------------------|
| Egg | pre-birth | 0.4 | n/a |
| Filhote (Cub) | 0–500 | 0.6 | 70:30 (mostly head) |
| Jovem (Young) | 500–2,000 | 0.8 | 50:50 |
| Adulto (Adult) | 2,000–10,000 | 1.0 | 40:60 (body dominant) |
| Elder | 10,000+ | 0.95 | 40:60 with details |

## Moluun — Forest Mammal → Alien Primate

### Filhote
- Round ball body, stub arms, short legs
- 2 large front eyes + 2 dormant dark dots on sides (future lateral eyes)
- No ears defined, rosy cheeks
- Velvet skin with faint micro-luminescent patterns
- Vibe: hamster baby

### Jovem
- Oval body, bigger arms, clawed paws
- 4 active eyes (2 front dark + 2 smaller lateral dark)
- Pointy ears appearing with translucent membranes
- Claws with suction pads
- Vibe: teen fox

### Adulto
- Upright biped, strong torso, muscular legs, functional arms with claws
- 4 complete eyes (front dark, lateral dark) with different iris colors
- Curved bone horns covered in moss/lichen (symbiosis — etharin antenna)
- Bioluminescent fiber mane
- Prehensile tail with sensory tip
- Symmetrical face markings (natural war paint)
- Vibe: wolf/bear standing with antler-mane — imposing but warm

### Elder
- Branching coral-like horns
- Moss partially covers body
- 4 eyes glow constantly
- Face pattern extends across entire body
- Vibe: one with the forest

## Pylum — Highland Bird → Alien Wyvern

### Filhote
- Fluffy ball with tiny beak
- 2 eyes + 2 heat-sensing pit dots on forehead
- Wing-stubs (down feathers), iridescent shimmer
- Vibe: cute chick

### Jovem
- Oval body, wings growing, thin legs appearing
- 3 eyes (2 normal + 1 central forehead UV-filter eye)
- 2 wings + buds of 2 smaller wings
- Metallic gradient plumage
- Vibe: young falcon

### Adulto
- Aerodynamic body, regal posture, raptor legs
- 4 wings (main pair huge + stabilizer pair)
- 4 eyes (2 lateral raptor + 2 dorsal with UV membrane)
- Fan-opening feather crest (display)
- 2-part articulated beak
- Sensory tail feathers (air pressure)
- Raptor talons
- Fractal geometric plumage patterns
- Vibe: griffin/phoenix — majestic aerial predator

### Elder
- Partially crystalline feathers (highland mineral)
- Fractal patterns glow
- Wing edges refract light (prism effect)

## Skael — Cave Reptile → Alien Dragon

### Filhote
- Fat cylinder with short tail
- 2 dark eyes + 2 infrared pit organs (red dots)
- Soft scales with faint bioluminescence
- Vibe: chubby lizard

### Jovem
- Elongating body, thick tail, semi-erect posture
- 4 eyes (2 dark main + 2 IR lateral active)
- Horns sprouting, dorsal plate starting
- Initial tail spines
- Vibe: young iguana

### Adulto
- Quadruped/semi-erect, armored body, massive tail
- 6 eyes (2 large front dark + 2 IR lateral + 2 tiny dorsal)
- Branching horns with luminous crystal veins
- Armored dorsal plates
- 4-way mandible jaw (Predator-style)
- Crystalline tail club
- 3-finger claws with thermal suction pads
- Hexagonal scale armor
- Dorsal crest pulses with breathing
- Vibe: terrestrial dragon — armored tank, intimidating

### Elder
- Crystal from horns grows along spine
- Scales fuse with stone
- Eyes deepen to absolute black
- Dorsal plate becomes continuous ridge
- Vibe: becoming the cave itself

## Nyxal — Abyssal Squid → Alien Leviathan

### Filhote
- Translucent gelatinous ball (organs visible)
- 2 large dark eyes
- 2 short tentacles
- Random color spots (uncontrolled chromatophores)
- Vibe: baby squid

### Jovem
- Body defining shape, mantle growing
- 4 eyes (diamond arrangement)
- 4 tentacles
- Controlled chromatophores (color shifts with mood)
- First voluntary bioluminescence
- Vibe: teen octopus

### Adulto
- Large body, imposing mantle, floating posture
- 6 eyes (2 large front + 2 medium lateral + 2 dorsal)
- 8 tentacles (4 long locomotor + 2 medium manipulator + 2 short sensory)
- Mantle with lateral fins
- Chromatophores form communicative geometric patterns
- Multi-color bioluminescence (each tentacle glows different)
- Hidden chitinous beak between tentacles
- Fractal skin texture
- Ink organ (defense — visible as dark cloud)
- Vibe: intelligent kraken — alien, hypnotic

### Elder
- Almost fully translucent
- Brain visible pulsing with bioluminescence
- Tentacles with rhythmic luminous rings
- Chromatophores form mandalas
- Eyes absorb all light — the deepest black on Ethara
- Vibe: living nebula — pure luminous intelligence

## Implementation Notes

Each growth stage needs:
1. Different vertex positions in the sprite generator (separate function per stage)
2. Different part count (filhote has fewer parts than adult)
3. Different rig anchors (adult Nyxal has 8 tentacle anchors vs filhote's 2)
4. Different sprite files (stored in subdirectories: `moluun/filhote/`, `moluun/adult/`)
5. Dynamic rig loading based on growth stage (rig changes when creature evolves)

The spawn system needs to:
1. Check creature age → determine growth stage
2. Load the correct rig for that stage
3. Load sprites from the correct subdirectory
4. When stage changes → despawn + respawn with new rig (evolution moment!)
