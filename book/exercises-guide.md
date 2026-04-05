# Exercise Structure Guide

> Every chapter ends with exercises that build toward the game. By the end of the book, the reader has built Kokoro themselves.

## Exercise Format

Each chapter has 3 levels of exercises:

### Level 1: Practice (Reinforce the concept)
Quick exercises that test understanding of the chapter's Rust concepts.
- "Modify X to do Y"
- "What would happen if...?"
- "Fix this broken code"

### Level 2: Build It (Add to Kokoro)
Hands-on tasks that directly advance the game.
- "Add a new gene to the Genome struct"
- "Create a test for the FSM transitions"
- "Implement a new food type"

### Level 3: Explore (Go deeper)
Open-ended challenges for curious readers.
- "Design a 5th species and implement its rig"
- "What biological system would improve realism?"
- "Research how real octopuses communicate and model it"

---

## Exercise Map by Chapter

### Part I — Foundations

**Ch1 (Welcome)**
1. Practice: Change the window title to include your creature's name
2. Build It: Add a `--debug` command line flag that prints system info
3. Explore: Research 3 other Rust game frameworks and compare to Bevy

**Ch2 (Ownership)**
1. Practice: Create a function that takes ownership of a String and returns its length
2. Build It: Add a `description()` method to `Genome` that returns a String describing the creature
3. Explore: Write code that demonstrates the difference between `Clone` and `Copy`

**Ch3 (Structs)**
1. Practice: Create a `Food` struct with `name`, `calories`, `protein`, `fiber` fields
2. Build It: Add a `Food` struct to the project with a `nutritional_value()` method
3. Explore: Implement `Display` for `VitalStats` so it prints a nice summary

**Ch4 (Enums)**
1. Practice: Add a `Taxonomy` enum with variants for biological classification
2. Build It: Create a `FoodType` enum (Fruit, Meat, Seed, Plankton) with species preferences
3. Explore: Model the egg stage as an enum with species-specific variants

**Ch5 (Functions)**
1. Practice: Write a function that calculates metabolic rate from body weight and activity level
2. Build It: Refactor `update_mood()` to use helper functions for each mood transition
3. Explore: Implement a `simulate_day()` function that fast-forwards 24 hours of ticks

**Ch6 (Borrowing)**
1. Practice: Write a function that borrows two `Genome`s and returns which has higher curiosity
2. Build It: Add a `compare_species()` function that borrows two creatures and lists differences
3. Explore: Explain why `crossover()` needs `&Genome` (not owned) for both parents

**Ch7 (Error Handling)**
1. Practice: Convert a function that panics into one that returns `Result`
2. Build It: Add proper error handling to the persistence module (no more `.unwrap()`)
3. Explore: Create a custom `KokoroError` enum that covers all game error types

**Ch8 (Collections)**
1. Practice: Use a `HashMap` to track how many times each food has been given
2. Build It: Create a `FoodInventory` resource using `HashMap<FoodType, u32>`
3. Explore: Implement a history tracker using `VecDeque` with a max size

**Ch9 (Traits)**
1. Practice: Define a `Biological` trait with `metabolic_rate()` and `preferred_food()` methods
2. Build It: Implement `Biological` for each species
3. Explore: Use trait objects (`Box<dyn Biological>`) to store heterogeneous creatures

**Ch10 (Modules)**
1. Practice: Split a large file into 3 submodules and re-export publicly
2. Build It: Create the `biology/` module with `metabolism.rs`, `nutrition.rs`, `hygiene.rs`
3. Explore: Compare Rust modules with Python packages and Java packages

### Part II — Game Core (Ch11-16)

Each chapter's "Build It" exercise constructs a piece of the working game:
- Ch11: Spawn a window with a colored background that changes with the day cycle
- Ch12: Insert Genome and Mind as resources, display values in the console
- Ch13: Build the tick system that decays stats every second
- Ch14: Generate 100 random genomes and analyze the distribution of traits
- Ch15: Implement the full FSM and test with manual stat overrides
- Ch16: Add circadian bonus and verify night owls behave differently

### Part III — Intelligence (Ch17-22)

- Ch17: Save and load a creature, verify all fields survive the roundtrip
- Ch18: Build the MLP and test that forward pass produces valid probabilities
- Ch19: Train on synthetic data and verify loss decreases
- Ch20: Cross two parents and verify child genes are within bounds
- Ch21: Switch between 4 creatures and verify state is preserved
- Ch22: Trace an event from button press to visual effect (document the chain)

### Part IV — Visuals (Ch23-28b)

- Ch23: Spawn a creature with procedural meshes, no sprites
- Ch24: Implement the rig for a new 5th species you design
- Ch25: Load sprites and implement the fallback system
- Ch25b: Generate sprites for your custom species
- Ch26: Build the vitals panel showing BPM and breathing rate
- Ch27: Add a new species behavior (e.g., tail wagging for a canine species)
- Ch27b: Make a creature jump when tapped and land with bounce
- Ch28: Extract your custom systems into a standalone plugin
- Ch28b: Add a "skip to adult" cheat in dev mode

### Part V — Biology (Ch29-37)

- Ch29: Classify all 4 species using real-world taxonomy analogs
- Ch30: Implement the egg stage — incubation timer, hatching animation
- Ch31: Create 5 food items with nutritional profiles, test species preferences
- Ch32: Model a simple skeleton with joints, make a limb move
- Ch33: Implement one ability (e.g., Nyxal electric sense as a sonar pulse visual)
- Ch34: Add scent communication — visible scent clouds that other creatures react to
- Ch35: Implement grooming behavior — creature cleans itself, hygiene stat improves
- Ch36: Make the creature refuse a food it's been given too many times
- Ch37: Detect a touch on the creature's ear and trigger a happy reaction

### Part VI — Testing (Ch38-40)

- Ch38: Write 10 unit tests covering Genome, Mind, and Physics
- Ch39: Write an integration test that runs a full creature lifecycle
- Ch40: Profile the app and identify the slowest system

### Part VII — Web (Ch41-45)

- Ch41: Build the health endpoint, test with curl
- Ch42: Implement registration and login, store passwords safely
- Ch43: Create a page that displays your Kobara's stats in the browser
- Ch44: Export creature data from the game and display it on the web
- Ch45: Deploy the API to a free tier service and access from your phone

---

## Teaching Philosophy

1. **Every exercise produces visible output** — no abstract busywork
2. **Exercises accumulate** — Ch3's Food struct is used in Ch31's nutrition system
3. **Difficulty scales** — Level 1 is always doable, Level 3 is always challenging
4. **Biology drives the learning** — Rust concepts are taught through biological modeling
5. **The game is the reward** — each chapter makes the creature more alive
