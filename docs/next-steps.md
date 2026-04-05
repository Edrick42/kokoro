# Kokoro — Next Steps (Ordered)

## Priority 1: MVP Blockers (do these first)

### 1.1 Egg Stage
- Create `creature/egg.rs` with `EggState` component
- Species-specific egg types: cells (Nyxal), egg (Pylum), cub (Moluun), crystal (Skael)
- Incubation timer (player interaction speeds it up)
- Hatching animation → transitions to Baby stage
- New player starts with an egg, not a creature

### 1.2 Realistic Time Cycles
- Mood transitions should be gradual (interpolate over 10-30 seconds, not instant)
- Sleep takes time (can't wake up instantly)
- Hunger builds slowly, not in sudden jumps
- Add mood transition system with lerp/easing
- Energy recovery during sleep is gradual (not +30 instant)

### 1.3 Nutrition System
- Create `Food` struct with: name, calories, protein, fiber, minerals
- Create 8-10 food items (fruit, meat, seeds, plankton, insects, etc.)
- Species preferences: Moluun likes fruit, Pylum likes seeds, Skael likes meat, Nyxal likes plankton
- Replace single "Feed" button with food selection menu
- Creature rejects food it dislikes (based on preference learning)
- Malnutrition effects (missing nutrients → health drops)

### 1.4 Touch Interaction
- Click/touch detection on creature body parts
- Detect which part was touched (head, body, ears, tentacles)
- Petting: increases happiness, builds trust
- Scratching: species-specific sweet spots (behind ears for Moluun, under wing for Pylum)
- Tickling: playful reaction
- Trust meter that grows with positive touch, decreases with rough handling

### 1.5 Learning & Preferences
- `PreferenceMemory` resource tracking food history, activity history
- After N times eating the same food → develops like/dislike
- Creature can initiate: request play (bouncing), refuse sleep (moving away), beg for food
- Neural network integrates preference data into mood predictions

### 1.6 Sound Design
- Ambient biome sounds per species
- Creature vocalizations (happy chirp, hungry whine, sleepy sigh)
- Heartbeat audio synced with BPM
- Breathing audio synced with rate
- UI interaction sounds (menu open/close, button press)

### 1.7 Environment Art
- Background image per species/biome
- Day/night cycle affects background lighting
- Subtle parallax or animated elements (floating spores for Verdance, updrafts for Highlands)

## Priority 2: Depth (after MVP)

### 2.1 Metabolism System
- Metabolic rate per species (affects hunger, energy burn, fat storage)
- Body temperature regulation
- Weight gain/loss based on food intake vs energy expenditure
- Visible body changes (rounder when overfed, thinner when underfed)

### 2.2 Biological Systems (Skeleton, Muscles, Nervous)
- Skeleton model: bone count, joint flexibility per species
- Muscle model: strength, fatigue, recovery rate
- Nervous system: reflex speed, coordination, pain sensitivity
- These drive movement quality: clumsy babies, agile adults, slow elders
- Complex rig with articulated limbs for walking/swimming

### 2.3 Natural Abilities
- Nyxal: electric sense (sonar-like visual pulse in dark)
- Skael: echolocation (wave visual in caves)
- Moluun: scent marking (visible scent clouds)
- Pylum: thermal vision / UV sight (color overlay effect)
- Abilities develop with age, stronger in adults

### 2.4 Communication System
- Sound: species-specific vocalizations (visual sound waves)
- Movement: body language (posture changes)
- Expression: eye/mouth changes beyond mood sprites
- Scent: visible scent clouds that drift and fade
- Color: chromatophore changes (especially Nyxal)
- Inter-creature communication when multiple active

### 2.5 Taxonomy
- Classify species: Moluun → Mammalia-like, Pylum → Aves-like, Skael → Reptilia-like, Nyxal → Cephalopoda-like
- Taxonomy affects biological systems (mammals have fur insulation, reptiles have scale armor)
- Foundation for future species being properly categorized

### 2.6 Hygiene System
- Hygiene stat (0-100) that decays over time
- Species-specific cleaning: Moluun grooms fur, Pylum preens feathers, Skael sheds scales, Nyxal ink-cleans
- Player can help clean (new action)
- Low hygiene → health problems, social issues

## Priority 3: Dev Mode Expansion

### 3.1 Time Manipulation
- "Skip 1 hour" / "Skip 1 day" buttons
- Speed up tick rate (2x, 5x, 10x)
- Pause button

### 3.2 State Overrides
- Force any mood
- Set any stat to any value
- Force growth stage transitions
- Trigger absence effects manually

### 3.3 Hidden Info
- Full neural network weight visualization
- Gene expression details
- Metabolism charts (energy in vs energy out)
- Preference memory dump
- Event log browser

### 3.4 Spawn Controls
- Give any food item
- Trigger any ability
- Reset creature to egg stage

## Priority 4: Web + Distribution

### 4.1 API Endpoints
- GET /api/kobara/:id — creature data
- GET /api/lore/:species — species lore
- POST /api/auth/register — user registration
- POST /api/auth/login — login → JWT
- GET /api/profile — user profile with creatures

### 4.2 Frontend
- Login/register pages
- Creature profile page (stats, genome, history)
- Lore browser (species, biomes, mysteries)
- Community feed (other players' creatures)

### 4.3 Game ↔ Web Sync
- Export button in game → sends data to API
- Web shows real-time creature state
- Optional: WebSocket for live updates

## Priority 5: Testing

### 5.1 Unit Tests (immediate)
- Genome: random_for produces valid ranges, crossover bounds
- Mind: FSM transitions are correct for each threshold
- Physics: ground collision, buoyancy, impulses
- Absence: stat effects for each duration bracket
- Nutrition: food type effects on stats (when built)

### 5.2 Integration Tests
- Full lifecycle: egg → baby → adult → elder
- Save/load roundtrip for all 4 creatures
- Neural network train → predict → override cycle
- Species switch preserves state

## Priority 6: Book + Content

### 6.1 Write remaining chapters
- Priority: Ch7-10 (finish Part I Rust basics)
- Then Ch11-16 (game core — Bevy + ECS)
- Then Ch29-37 (biology — the unique selling point)

### 6.2 Add exercises to existing chapters
- Follow exercises-guide.md format
- 3 levels per chapter (Practice, Build It, Explore)

### 6.3 Start YouTube dev log
- First video: "I'm building a bio-simulation game in Rust"
- Show the creature breathing, reacting to absence, learning preferences
- Keep it authentic, don't over-produce

### 6.4 Publish early access on Gumroad
- Part I (Ch1-6) is complete — can sell as "Rust Basics through Game Dev"
- Price: $9.99 early access, raise as more chapters ship
