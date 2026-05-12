# Kokoro — Aesthetic Targets

> Métricas quantitativas extraídas de referências curadas. Cada número aqui vira constante no código.

> **2026-05-11 — Anatomy is now simulated, not authored.** Bones, joints,
> muscles, and nerves obey the universal physical laws documented in
> [`biomechanics.md`](biomechanics.md). The visual targets in this file
> define *species averages*; per-creature variation emerges from genome
> modulation of mass, ligament stiffness, muscle force, and nerve health
> — not from authored sprite variants. Poses are no longer drawn — they
> are what physics produces.

## Status

**Parte 1 — Egg & Cub stage (este documento, 2026-05-08)**
- Fontes: 7 pins do Cluster A do board "Pixel art style" (edrick0724)
  - 09 Charizard reimagined, 13 caranguejo-coco, 15 macaco, 17 laranja, 19 passarinho, 21 dinossauro, 06 borboleta+espada (perler)
- Constraint dura: **estética retro** (pixel duro, nearest-neighbor, sem AA suave, paleta curada em ramps)
- Paleta: **expandida** além das 6 cores originais. Mantemos as 6 como família central + ramps adicionais. Spirit retro = paleta curada e intencional, não gerada nem livre

**Nota:** Esta seção supersede a regra "6-color strict" de `docs/art-direction.md` para o trabalho do novo visual DSL. A paleta vira 16-32 cores organizadas em ramps por material, todas escolhidas — não geradas via HSL.

**Decisão de evolução de estilo (Cub → Adult → Elder): B + C combinados**

- **B** maturação gradual nas proporções (kindchenschema afrouxa progressivamente)
- **C** destino estético é "cozy ghibli" — adulto ainda warm e charmoso, mas com personalidade característica e mais detalhe

Em prática, isso significa um eixo contínuo `kawaii_factor: 1.0 (Cub) → 0.0 (Adult) → 0.2 (Elder, gentle return)` que controla:
- Razão olho/face (cai de 0.28 → 0.18 → 0.20)
- Roundness corporal (cai de 0.75 → 0.50 → 0.55)
- Presença de blush (1.0 → 0.0 → 0.0)
- Densidade de detalhe interno (sobe de 0.0 → 0.6 → 0.8 com lines de personalidade)
- Saturação de paleta (Adult mantém saturação, Elder dessatura ~25%)

Métricas exatas por estágio definidas nas Partes 2 e 3.

**Status (2026-05-08, depois do segundo lote do board com 36 pins extraídos de 69):**

| Parte | Estágio/Tópico | Status | Refs |
|---|---|---|---|
| 1 | Egg + Cub | ✅ Completa | 09, 13, 15, 17, 18, 19, 20, 21, 28, 30 |
| 2 (4b) | Young (interpolação) | ✅ Completa | 10, 02, 19 |
| 2 (4c) | Adult cozy ghibli | ✅ Completa | 02, 25, 29, 34, 10 |
| 3 (4d) | Elder gravitas | ✅ Completa | **14** (urso marrom) |
| 4 (5b) | Bioma Abyssal Depths | ✅ Completa | 33, 36 |
| 4 (5b) | Bioma Abyssal Shallows | ✅ Completa | 35 |
| 4 (5b) | Bioma Verdance | ✅ Completa | 41, 37 |
| 4 (5b) | Bioma Highlands (sunset) | ✅ Completa | 38, 39 |
| 5 | Animação keyframes | ❌ Sem refs | — |

---

## 1. Constraints absolutas (não-negociáveis)

| Constraint | Valor | Origem |
|---|---|---|
| Paleta total | 16-32 cores curadas, organizadas em ramps | Pinterest + retro spirit |
| Cores por criatura | 6-10 (subset escolhido da paleta master) | Pinterest A |
| Outline | sempre cor escura dessaturada da paleta, 1px | Pinterest A |
| Olhos | preto dessaturado sólido + glint cream | Pinterest A + art-direction |
| Anti-aliasing suave | NÃO | Retro |
| Cores geradas via HSL/genome direto | NÃO | Retro spirit = curado |
| Pixel grid | nearest-neighbor, sem blur | Retro |
| Round corner em UI | NÃO (flat 90°) | Retro Game Boy |

### Paleta master (proposta v1, ~24 cores)

Mantém as 6 originais como **família central** e expande em ramps por material. Todas escolhidas — nenhuma gerada por shift de HSL.

**Darks / Outlines (4):**
```
Near Black     #1B130D    outline padrão (warm)
Deep Brown     #3B2418    outline pra criatura warm
Deep Teal      #0A2A2D    outline pra criatura cool
Charcoal       #2A2520    sombras profundas
```

**Cream / Highlights (3):**
```
Cream          #D9C7AE    belly, background neutro, glint pequeno
Cream Light    #F0E2C8    glint forte, bioluminescência base
Off White      #FBF4E2    sparkle pontual, eye glint
```

**Warm earths (4):**
```
Tan            #C49870    skin secundário (Moluun)
Brown          #8B5A3A    fur shadow, soil
Brown Dark     #5A3622    fur deep shadow
Sand           #E5C896    background quente
```

**Gold / Orange ramp (Pylum + Moluun) (3):**
```
Gold Dark      #A07803
Gold           #D9A404    Moluun body base (existente)
Orange Bright  #F08828    accent (penas, escamas brilhantes)
```

**Orange ramp (Pylum body) (3):**
```
Orange Dark    #A04C03
Orange         #D96704    Pylum body (existente)
Orange Light   #F0883D    Pylum belly highlight
```

**Red ramp (Nyxal body + blush universal) (3):**
```
Red Dark       #A00930
Red            #D90D43    Nyxal body (existente)
Coral Pink     #F06B85    cheek blush universal
```

**Teal ramp (Skael body + bioluminescência) (3):**
```
Teal Dark      #014045
Teal           #016970    Skael body (existente)
Cyan Bright    #4DC3CC    bioluminescência fria, water highlight
```

**Greens (Verdance bioma) (3):**
```
Forest Dark    #1F3A28
Forest         #3D6E45
Sage           #87A878    folhagem clara, accent fresco
```

**Total: 26 cores curadas.** Nenhum brush pode invocar cor fora dessa lista. Genome modula **qual ramp** uma criatura usa e **qual posição na ramp** — não cores cruas.

**Implementação Rust:**

```rust
#[derive(Copy, Clone)]
pub enum Palette {
    NearBlack, DeepBrown, DeepTeal, Charcoal,
    Cream, CreamLight, OffWhite,
    Tan, Brown, BrownDark, Sand,
    GoldDark, Gold, OrangeBright,
    OrangeDark, Orange, OrangeLight,
    RedDark, Red, CoralPink,
    TealDark, Teal, CyanBright,
    ForestDark, Forest, Sage,
}
// Brushes recebem `Palette`, nunca `Color::rgb`.
// Compilador garante: zero cor fora da paleta.
```

---

## 2. Cuteness math (kindchenschema aplicado)

Razões medidas dos pins kawaii do Cluster A (em pixels do bounding box do personagem):

| Métrica | Pinterest range | Target Kokoro Cub | Target Kokoro Egg |
|---|---|---|---|
| Head height ÷ total height | 0.45-0.65 | **0.55** | n/a (egg = corpo único) |
| Eye width ÷ head width | 0.20-0.32 | **0.28** | 0 (egg sem olhos visíveis) |
| Eye area total ÷ head area | 0.18-0.28 | **0.22** | — |
| Eye vertical position (0=top, 1=bottom da face) | 0.55-0.70 | **0.62** | — |
| Eye spacing ÷ eye width | 0.4-0.8 | **0.6** | — |
| Mouth width ÷ head width | 0.10-0.18 | **0.13** | — |
| Cheek blush diameter ÷ head width | 0.08-0.14 | **0.11** | — |
| Cheek vertical position | 0.65-0.78 | **0.72** | — |
| Limb length ÷ body height | 0.10-0.25 | **0.18** | n/a |
| Body roundness (1=círculo) | 0.65-0.85 | **0.75** | 0.85 (egg quase oval) |
| Heads-tall ratio | 1.5-2.2 | **1.8** | n/a |

**Implementação Rust (bounded type, exemplo):**

```rust
pub struct CubProportions {
    pub head_to_body: Ratio<0.45, 0.65>,        // compile-time bound
    pub eye_to_head:  Ratio<0.20, 0.32>,
    pub eye_y_pos:    Ratio<0.55, 0.70>,
    pub mouth_to_head: Ratio<0.10, 0.18>,
    pub blush_to_head: Ratio<0.08, 0.14>,
    pub limb_to_body: Ratio<0.10, 0.25>,
    pub roundness:    Ratio<0.65, 0.85>,
}
```

Se o genome tentar gerar fora do range, **não compila**. Fofura vira invariante de tipo.

---

## 3. Especificação por feature

### 3.1 Outline

- Cor: Near Black `#1B130D`, sempre
- Espessura: **1px** estrito (nunca 2px exceto em silhuetas grandes ≥80px)
- Cobertura: 100% da silhueta externa
- Internal lines (separar barriga, asa, perna): **só onde a forma volumetricamente exige**, nunca como "decoração"
- AA: zero. Pixel duro.

### 3.2 Eyes (a feature mais importante)

- Forma: retângulo arredondado de 3-4px de largura × 4-5px de altura, geralmente vertical
- Cor: Near Black sólido
- Glint: 1 pixel Cream `#D9C7AE` no quadrante superior (top-left ou top-right consistente por espécie)
- Posição: metade inferior do rosto (y_ratio ≈ 0.62)
- Espaçamento entre olhos: 60% da largura de um olho
- **Sem pupila, sem íris** — bead-like solid (definido em art-direction.md)

### 3.3 Mouth

- Forma: 2-4 pixels horizontais, smile sutil `‿` ou neutro `—`
- Cor: Near Black mas **mais magro** (use `(R,G,B)` interpolado ~30% pra Cream pra evitar peso visual)
- Posição: 0.78-0.85 do rosto verticalmente (logo abaixo dos olhos)
- Forma muda com mood (já tem sistema FSM): `‿` happy, `—` neutral, `︵` sad, `o` surprised

### 3.4 Cheek blush (assinatura kawaii)

- Cor: **Coral Pink `#F06B85`** (já incluso na paleta master)
- Forma: 2-3px de diâmetro, oval levemente horizontal
- Posição: y ≈ 0.72, x = ±0.30 do centro do rosto
- Opacidade: integral (não AA)
- **Constraint:** só aparece em Egg/Cub/Young. Adult e Elder perdem (vira gravitas)

### 3.5 Body

- Roundness: 75% (entre círculo perfeito 1.0 e quadrado 0.0)
- Belly: Cream `#D9C7AE` ocupando ~35% da altura inferior do corpo
- Body color: cor da espécie (Gold/Orange/Teal/Red)
- Shadow color: species color × 0.85L (1 tom abaixo) usado em ~10% do corpo (lado oposto à luz)
- **Sem dithering interno** em Cub. Tons sólidos, transições duras.

### 3.6 Limbs

- Comprimento: 18% da altura do corpo
- Largura: 3-4px
- Pés: 2-3 pixels Near Black no fim (parece "patinhas")
- **Cubs frequentemente têm membros como tocos**, não articulados

### 3.7 Background / canvas (Egg/Cub portrait)

- Fundo do canvas onde o Cub é renderizado: Cream `#D9C7AE`
- Drop shadow sob a criatura: 2-3px Near Black com 1 tile horizontal de offset
- **Pixel-noise sutil** (1% dos pixels com Cream-darker): quebra o flat sem virar AA

---

## 4. Mapeamento paleta por espécie (Cubs)

### 4.1 Moluun Cub (forest, gold)

| Slot | Cor | Hex |
|---|---|---|
| Outline | DeepBrown | `#3B2418` |
| Body base | Gold | `#D9A404` |
| Body shadow | GoldDark | `#A07803` |
| Body highlight | OrangeBright | `#F08828` |
| Belly | Cream | `#D9C7AE` |
| Belly highlight | CreamLight | `#F0E2C8` |
| Eye | NearBlack | `#1B130D` |
| Eye glint | OffWhite | `#FBF4E2` |
| Cheek blush | CoralPink | `#F06B85` |
| Ear/paw inner | Tan | `#C49870` |

Total: **10 cores** ✓ (dentro do range 6-10)

### 4.2 Pylum Cub (highlands, orange)

| Slot | Cor | Hex |
|---|---|---|
| Outline | DeepBrown | `#3B2418` |
| Body base | Orange | `#D96704` |
| Body shadow | OrangeDark | `#A04C03` |
| Body highlight | OrangeLight | `#F0883D` |
| Belly | Cream | `#D9C7AE` |
| Belly highlight | CreamLight | `#F0E2C8` |
| Eye | NearBlack | `#1B130D` |
| Eye glint | OffWhite | `#FBF4E2` |
| Cheek blush | CoralPink | `#F06B85` |
| Beak/feet | Gold | `#D9A404` |

Total: **10 cores** ✓

### 4.3 Skael Cub (caverns, teal)

| Slot | Cor | Hex |
|---|---|---|
| Outline | DeepTeal | `#0A2A2D` |
| Body base | Teal | `#016970` |
| Body shadow | TealDark | `#014045` |
| Scale accent | CyanBright | `#4DC3CC` |
| Belly | Cream | `#D9C7AE` |
| Belly highlight | CreamLight | `#F0E2C8` |
| Eye | NearBlack | `#1B130D` |
| Eye glint | OffWhite | `#FBF4E2` |
| Cheek blush | CoralPink | `#F06B85` |
| Thermal pit | RedDark | `#A00930` |

Total: **10 cores** ✓ — RedDark nos pit organs traz a referência biológica (pit viper) sem quebrar a paleta cool.

### 4.4 Nyxal Cub (deep ocean, red + bioluminescence)

| Slot | Cor | Hex |
|---|---|---|
| Outline | DeepBrown | `#3B2418` |
| Body base | Red | `#D90D43` |
| Body shadow | RedDark | `#A00930` |
| Body highlight | CoralPink | `#F06B85` |
| Belly | Cream | `#D9C7AE` |
| Eye | NearBlack | `#1B130D` |
| Eye glint | OffWhite | `#FBF4E2` |
| Bioluminescent base | CyanBright | `#4DC3CC` |
| Bioluminescent peak | CreamLight | `#F0E2C8` |

Total: **9 cores** ✓ — Nyxal pula blush (CoralPink já é variação do body). Em troca ganha bioluminescência ciano com peaks cream — o efeito que falta nas outras espécies.

---

## 4b. Young stage (transição Cub → Adult, kawaii_factor ≈ 0.7)

Young mantém a maior parte do kindchenschema do Cub mas começa a esticar. Refs principais: 10 (Wartortle), 19 (chick em pé), 02 (proporções intermediárias).

| Métrica | Cub | **Young** | Adult |
|---|---|---|---|
| `kawaii_factor` | 1.0 | **0.7** | 0.0 |
| Head/body | 0.55 | **0.45** | 0.30 |
| Eye/head | 0.28 | **0.24** | 0.18 |
| Eye y_pos | 0.62 | **0.58** | 0.50 |
| Roundness | 0.75 | **0.65** | 0.50 |
| Heads-tall | 1.8 | **2.5** | 3.5 |
| Blush opacidade | 1.0 | **0.5** (sutil) | 0.0 |
| Limb length | 0.18 | **0.30** | 0.45 |
| Detail interno | 0.0 | **0.3** | 0.6 |

Tudo mais idêntico ao Cub — paleta, outline, mouth, glint. Young é interpolação linear, não estágio com personalidade própria. Genome bumps (-/+ 5%) podem fazer Young de gene curiosity alto parecer mais "spunky" (eyes maior, postura ereta) e Young de gene resilience alto parecer mais "calm" (eyes neutro, postura agachada).

---

## 4c. Adult stage (cozy ghibli, kawaii_factor = 0.0)

Refs principais: 02 (variações de outfit/silhouette), 25 (witch — characterful, mature, narrative), 29 (Pokemon dragon — anatomical detail, multi-color), 34 (owl — multi-band palette com unidade), 10 (Wartortle adult-style).

### 4c.1 Princípios

Adult **não é cub crescido**. É um modo estético próprio:

1. **Detail density alta** (0.6) — internal lines comunicando anatomy/material/personality
2. **Palette cresce** (8-10 cores) — mais bandas dentro da assinatura da espécie
3. **Silhueta característica** — feature distinguishing prominent (cauda Skael, asas Pylum, mantle Nyxal, postura/orelhas Moluun)
4. **Pose com peso** — mid-action, não floating-bouncy
5. **Eye shape varia** — pode ser slit (resilience alto), almond (curiosity alto), redondo padrão (default). Glint mantém mas menor proporcionalmente.
6. **Sem blush** absolutamente
7. **Outline mais variado** — pode ter 2 tons de outline (DeepBrown geral + NearBlack onde silhueta é crítica)

### 4c.2 Métricas Adult

| Métrica | Target | Ref |
|---|---|---|
| Head/body | 0.30 | 02, 25 |
| Eye/head | 0.18 | 29, 25 |
| Eye y_pos | 0.50 | 25, 29 |
| Mouth/head | 0.10 | 25 |
| Roundness | 0.50 | 02, 25 |
| Heads-tall | 3.5 | 02, 25 |
| Limb/body | 0.45 | 02, 25 |
| Internal detail density | 0.6 | 29, 34 |
| Cores totais | 8-10 | 29, 34 |
| Pose | mid-action capable | 02, 25, 29 |

### 4c.3 Mapeamento paleta por espécie (Adults)

Adicionam 2-3 cores comparado ao Cub. Cada espécie ganha **uma marking color** + **uma material accent**.

#### Moluun Adult (forest, gold + material)
Adiciona Brown (fur outline), Sage (folhagem-like markings), e marking pattern em cada flank.

| Slot | Cor |
|---|---|
| Outline silhueta | DeepBrown |
| Outline interno | Brown |
| Body base | Gold |
| Body shadow | GoldDark |
| Body highlight | OrangeBright |
| Belly | Cream |
| Belly highlight | CreamLight |
| Eye | NearBlack |
| Eye glint | OffWhite |
| Marking (flank) | Sage |
| Paw inner | Tan |

Total: **11 cores** (excede em 1 — aceitável só no Moluun adult, marking é central na identidade)

#### Pylum Adult (highlands, orange + plumage detail)
Adiciona OrangeLight (flight feathers), Gold (chest plumage), CoralPink (wing tip accent — substituindo o blush perdido).

| Slot | Cor |
|---|---|
| Outline silhueta | DeepBrown |
| Body base | Orange |
| Body shadow | OrangeDark |
| Body highlight | OrangeLight |
| Wing primary | Gold |
| Wing tip accent | CoralPink |
| Belly | Cream |
| Eye | NearBlack |
| Eye glint | OffWhite |
| Beak/feet | GoldDark |

Total: **10 cores**

#### Skael Adult (caverns, teal + thermal pits)
Adiciona CyanBright (scale crests), RedDark (thermal pits proeminentes), Charcoal (back ridges).

| Slot | Cor |
|---|---|
| Outline silhueta | DeepTeal |
| Outline interno | Charcoal |
| Body base | Teal |
| Body shadow | TealDark |
| Crest accent | CyanBright |
| Belly | Cream |
| Eye | NearBlack |
| Eye glint | OffWhite |
| Thermal pits | RedDark |
| Underbelly plates | Tan |

Total: **10 cores**

#### Nyxal Adult (deep ocean, red + chromatophores)
Bioluminescência expande — agora inclui mantle peaks. Chromatophore patches em CoralPink.

| Slot | Cor |
|---|---|
| Outline silhueta | DeepBrown |
| Body base | Red |
| Body shadow | RedDark |
| Mantle accent | RedDark |
| Belly | Cream |
| Eye | NearBlack |
| Eye glint | OffWhite |
| Chromatophore patches | CoralPink |
| Bioluminescent specks | CyanBright |
| Bioluminescent peaks | CreamLight |

Total: **10 cores**

---

## 4d. Elder stage (gravitas, kawaii_factor ≈ 0.2 mas via REDUÇÃO, não retorno ao kawaii)

Ref: **14 (urso marrom em pé, plump, slit eyes, postura solene).**

### 4d.1 Princípios — Elder NÃO é Cub dessaturado

Olhar a ref 14 prova: Elder tem sua estética própria. Não é maturação avançada do Adult, e não é regressão ao Cub. É **redução zen**:

1. **Detail density CAI** (de 0.6 Adult → 0.2 Elder). Adult acumula linhas; Elder solta linhas.
2. **Paleta encolhe** (de 8-10 Adult → 4-6 Elder). Material accents somem; restam silhouette + 1 ramp + cream.
3. **Eye shape muda pra slit** (linha horizontal de 1-2px), não round. Comunica wisdom/calm/contentment. **Glint optional, frequentemente ausente.**
4. **Mouth quase invisível**, deslocada lateralmente (sutil sorriso de canto, não centralizado)
5. **Postura grounded** — em pé mas com peso, braços recolhidos próximo ao corpo
6. **Saturação geral cai ~25%** — não dessaturação total, mas paleta puxa pro tom mais terra (Brown ramp ganha presença em todas espécies)
7. **Sem blush**, sem material accents, sem chromatophores ativos, sem markings vibrantes
8. **Roundness moderada** (0.55) — corpo plump mas com massa/peso, não bouncy

### 4d.2 Métricas Elder

| Métrica | Adult | **Elder** | Comentário |
|---|---|---|---|
| `kawaii_factor` | 0.0 | **0.2** | sutil presença de fofura via roundness |
| Head/body | 0.30 | **0.35** | levemente maior (sabedoria centrada na cabeça) |
| Eye shape | round/almond | **slit horizontal** 1-2px tall | dramatic shift, signal forte |
| Eye y_pos | 0.50 | **0.45** | levemente alto (olhar pra dentro) |
| Eye glint | small | **ausente ou 1px raro** | quietude |
| Mouth | 10% width | **5% width**, deslocado | quase invisível, sorriso lateral |
| Blush | 0 | **0** | nunca |
| Roundness | 0.50 | **0.55** | volta um pouco |
| Heads-tall | 3.5 | **3.0** | ligeiramente baixo (postura recolhida) |
| Limb/body | 0.45 | **0.30** | membros mais curtos, recolhidos |
| Internal detail | 0.6 | **0.2** | redução drástica |
| Cores totais | 8-10 | **4-6** | redução drástica |
| Saturação geral | 100% | **75%** | desbotamento sutil |

### 4d.3 Paleta Elder (4-6 cores por espécie)

Cada Elder pega: silhouette dark + body shadow + belly + 1 accent (faded). Material colors saem.

#### Moluun Elder
| Slot | Cor |
|---|---|
| Outline | DeepBrown |
| Body | Brown (puxa pro terra, não Gold cheio) |
| Body shadow | BrownDark |
| Belly | Cream |
| Eye (slit) | NearBlack |

**5 cores.** Gold quase desaparece — sobra só hint nas pontas. Lê-se como "ancião marrom-dourado" não "ancião dourado".

#### Pylum Elder
| Slot | Cor |
|---|---|
| Outline | DeepBrown |
| Body | OrangeDark (não Orange cheio) |
| Body shadow | BrownDark |
| Wing edge | GoldDark (pena fading) |
| Belly | Cream |
| Eye (slit) | NearBlack |

**6 cores.**

#### Skael Elder
| Slot | Cor |
|---|---|
| Outline | DeepTeal |
| Body | TealDark (não Teal cheio) |
| Body shadow | Charcoal |
| Belly | Cream |
| Crest hint | Teal (pequeno toque, fading) |
| Eye (slit) | NearBlack |

**6 cores.** Bioluminescência apaga. Calm.

#### Nyxal Elder
| Slot | Cor |
|---|---|
| Outline | DeepBrown |
| Body | RedDark (não Red cheio) |
| Body shadow | Charcoal |
| Belly | Cream |
| Bioluminescent (faint) | CyanBright (1-2 specks só) |
| Eye (slit) | NearBlack |

**6 cores.** Bioluminescência reduzida a memória — 1-2 pontos, não constelação.

---

## 5. Egg stage

Egg é mais simples. Sem olhos visíveis, sem membros. Apenas:

- Forma: oval vertical, roundness 0.85, ratio altura/largura 1.2:1
- Outline: Near Black 1px
- Body: cor da espécie (Gold/Orange/Teal/Red)
- Padrão: 2-3 manchas Cream irregulares (variação por genome.hue: posição shift)
- Belly highlight: Cream em forma de elipse pequena no quadrante superior-esquerdo (simulando luz vinda de cima-esquerda)
- Sem face. **A cara aparece quando o egg eclode.**

---

## 5b. Biomas (Parte 4 — parcial: Shallows + Depths)

Refs: **33 (caverna oceânica vertical com shaft de luz)**, **35 (caverna de cristais horizontal)**, **36 (underwater layered horizontal com fauna silhueta).**

### 5b.1 Princípios universais de bioma (aplicam aos 4)

1. **Layering em 3 níveis mínimo** — foreground (silhueta dark sólida), midground (paleta principal do bioma), background (paleta dessaturada/clara, fog effect)
2. **Vertical aspect preferencial** — telas mobile são tall, criatura ocupa centro vertical, bioma flui acima e abaixo
3. **Light source consistente** — em águas/caves, vem de cima (shaft de Cyan/Cream); em florestas/highlands, vem do canto superior
4. **Paleta restrita por bioma** — 1 ramp dominante (~3-4 cores) + 1 ramp accent (~2 cores) + Cream/CreamLight pra highlights + silhouette dark
5. **Particles/specks** — 1-2px scattered. Densidade ~2-3% dos pixels totais. Movimento sutil (drift down em água, drift up em floresta).
6. **Sem objetos centralizados** que competem com a criatura — bioma é cenário, não foco. Detalhes ricos vão pras laterais.

### 5b.2 Abyssal Depths (Nyxal biome) — ref 33, 36

**Mood:** profundo, quieto, vasto, com pontos de luz como sopros de vida no escuro.

**Layering vertical (de cima pra baixo):**
- Top 20%: shaft de luz forte (CyanBright + CreamLight), particles brilhantes drift descendo
- Mid 50%: gradient de Teal → TealDark → Charcoal (background água)
- Bottom 30%: rocha/coral foreground (silhueta DeepTeal + Charcoal sólido), bioluminescência CyanBright pontual em corais/rochas

**Paleta restrita do bioma (8 cores):**
| Slot | Cor |
|---|---|
| Foreground silhueta | Charcoal |
| Foreground body | DeepTeal |
| Mid water dark | TealDark |
| Mid water | Teal |
| Light shaft base | CyanBright |
| Light shaft peak | CreamLight |
| Particle drift | CreamLight (1-2px scattered) |
| Bioluminescent coral | CyanBright clusters |

**Density specs:**
- Light shaft: cobre ~25% da largura no topo, afina pra ~10% no fundo
- Particles: ~0.5% pixels do bioma (sutil)
- Bioluminescent coral: 3-5 clusters de 4-8 pixels, espalhados na zona inferior

### 5b.3 Abyssal Shallows (Skael biome) — ref 35

**Mood:** caverna iluminada por dentro, cristais como velas, mais íntimo que Depths.

**Layering horizontal (de fundo pra frente):**
- Background 30%: cave wall escuro (Charcoal + DeepTeal sólidos, formas angulares)
- Midground 40%: cristais grandes (Teal → CyanBright ramp, formas geométricas com facets)
- Foreground 30%: chão de pedras + cristais pequenos (DeepBrown silhueta + clusters CyanBright + CoralPink/RedDark accents pequenos)

**Paleta restrita (9 cores):**
| Slot | Cor |
|---|---|
| Cave wall dark | Charcoal |
| Cave wall mid | DeepTeal |
| Crystal dark | TealDark |
| Crystal mid | Teal |
| Crystal bright | CyanBright |
| Crystal peak | CreamLight |
| Floor stone | DeepBrown |
| Pink crystal accent | CoralPink |
| Red crystal accent | RedDark |

**Density specs:**
- Crystal facets: 4-6 grandes no midground, 8-12 pequenos no foreground/background
- Pink/red accents: 2-3 clusters totais, pequenos (3-5 pixels), criam variação cromática numa cena dominante azul
- Sem light shaft (caverna fechada). Iluminação vem dos próprios cristais.

### 5b.4 Verdance (Moluun biome) — refs 41 (Totoro pixel forest), 37 (Stardew village forest)

**Mood:** acolhedor, vivo, brisa, luz filtrada por folhas, casa segura.

**Layering vertical (de fundo pra frente):**
- Top 35%: céu azul claro com nuvens pixeladas em Cream/CreamLight; folhas drift drift drift caindo (Sage/ForestDark specks)
- Mid 40%: massa de copas (ForestDark + Forest), tronco de árvore lateral em Brown/BrownDark, possíveis cogumelos/flores como motifs (CoralPink/Gold pequenos)
- Bottom 25%: chão de grama (Forest + Sage gradient duro), drop shadow da criatura, talvez 1-2 pedrinhas (Brown silhueta)

**Paleta restrita do bioma (10 cores):**
| Slot | Cor |
|---|---|
| Sky base | CyanBright (faded — usar com mistura, ver nota) |
| Sky highlight (cloud) | CreamLight |
| Cloud shadow | Cream |
| Tree trunk | Brown |
| Tree trunk shadow | BrownDark |
| Foliage dark | ForestDark |
| Foliage mid | Forest |
| Foliage highlight | Sage |
| Grass mid | Forest |
| Flower/mushroom accents | CoralPink ou Gold (escolher 1 por mapa, não dois juntos) |

> **Nota sobre céu:** CyanBright sozinho fica frio demais pra Verdance. Para o céu, usar CyanBright **dithered com Cream** num padrão 1:1 — produz percepção de azul pastel sem precisar nova cor. Mesmo trick que Game Boy usava pra "expandir" 4 tons.

**Density specs:**
- Leaf particles: ~1.5% pixels do bioma (mais que Depths/Shallows — Verdance tem vento)
- Tree canopy ocupa ~40-50% da largura no topo (não simétrico)
- Tronco lateral: 1 árvore visível em pelo menos um lado, varia por cena
- Acentos florais: 2-3 clusters de 2-4 pixels, espalhados na zona inferior

### 5b.5 Highlands (Pylum biome) — refs 38 (mountain sunset), 39 (mountain minimalist)

**Mood:** vasto, alto, vento, luz dramática (sunset) OU silêncio frio (minimalist). **Escolha:** vamos de **sunset** como base padrão (ref 38) — alinha com paleta Pylum (Orange/Gold) e diferencia visualmente das Shallows/Depths que são frias.

**Layering vertical (de fundo pra frente):**
- Top 40%: céu sunset em bandas horizontais — **Charcoal (cloud edge)** → **OrangeDark** → **Orange** → **OrangeBright** → **Gold** → **CreamLight** descendo do topo até o horizonte
- Mid 35%: silhuetas de picos distantes (TealDark sólido, sem detail interno) — sense of vastness
- Bottom 25%: pico principal foreground em Charcoal silhouette + neve/rocha iluminada (CreamLight no lado iluminado, TealDark no lado sombra), plus rocha ao longo da base

**Paleta restrita do bioma (10 cores):**
| Slot | Cor |
|---|---|
| Cloud silhouette | Charcoal |
| Sky band 1 (top) | OrangeDark |
| Sky band 2 | Orange |
| Sky band 3 | OrangeBright |
| Sky band 4 (horizon glow) | Gold |
| Sky band 5 (peak) | CreamLight |
| Distant peaks | TealDark |
| Foreground peak silhouette | Charcoal |
| Peak lit side (snow/rock) | CreamLight |
| Peak shadow side | DeepTeal |

**Density specs:**
- Sky bands: 5 bandas horizontais cada com altura ~7-10% do total da tela. Bandas vão dithered nas transições (1px linha intercalada) pra suavizar sem AA.
- Distant peaks: 2-3 silhuetas em camadas, decrescendo em valor (mais escuro = mais perto)
- Foreground peak: ocupa ~40% da largura, centralizado-direita ou centralizado-esquerda (não centro)
- Wind particles: opcional, 1-2px Cream specks drift horizontal (não vertical como em outros biomas)

**Variação Highlands "Cold/Day":** quando time of day = day em vez de sunset, paleta puxa pra TealDark + Teal + Cream + CyanBright (tipo ref 39). Detalhamento dessa variação fica pra Fase 1 quando montarmos o sistema de DayCycle integrado com bioma.

### 5b.6 Resumo das paletas por bioma

| Bioma | Cores totais | Ramp dominante | Accent | Light source |
|---|---|---|---|---|
| Verdance | 10 | Forest + Brown | CoralPink ou Gold | Filtered top-down |
| Highlands (sunset) | 10 | Orange ramp + Charcoal/Teal | CreamLight glow | Horizon (lateral) |
| Abyssal Shallows | 9 | Teal/Cyan | CoralPink/RedDark | Crystal interior |
| Abyssal Depths | 8 | Teal/Charcoal | CyanBright | Top shaft |

Total único de cores entre todos biomas: **18** (todas dentro da paleta master 26). Sobra: Tan, Sand, Red, GoldDark, OrangeLight, RedDark, NearBlack, DeepBrown — disponíveis para criaturas + UI.

---

## 6. Negative space (o que NÃO fazer)

Decisões explícitas pra evitar drift:

- ❌ Anti-aliasing suave (gradient blur). Pixel duro sempre.
- ❌ Cores fora da paleta master (mesmo "só pra esse detalhe")
- ❌ Outline preto puro `#000000` — sempre tom escuro dessaturado da paleta
- ❌ Mais de 10 cores por criatura
- ❌ Cores geradas via shift de HSL/genome direto. Genome escolhe **slot da paleta**, nunca cor crua.
- ❌ Eyes muito redondos perfeitos (deixar levemente quadrado, vibe pixel)
- ❌ Detalhe interno gratuito (cada pixel não-silhueta tem que justificar peso)
- ❌ Glow/blur efeitos contínuos (bioluminescência = pixels Cream sólidos cintilando, não halo)
- ❌ Drop shadow forte (2-3px máx, sutil)
- ❌ Sombra própria com gradient (use 1 tom darker, transição dura)
- ❌ Round corner rectangles em UI (Game Boy é flat 90°)

---

## 7. Edição rápida (preview do `visual-edit-map.md`)

Cada métrica acima vira parâmetro editável em `assets/species/X.ron`:

```ron
MoluunCub(
    proportions: (
        head_to_body: 0.55,
        eye_to_head: 0.28,
        eye_y_pos: 0.62,
        mouth_to_head: 0.13,
        blush_to_head: 0.11,
        roundness: 0.75,
    ),
    palette_slots: (
        body: Gold,
        body_shadow: GoldDark,
        belly: Cream,
        accent: Orange,
        blush: RedLight,
    ),
    features: (
        ears: Floppy(droop: 0.35, length: 0.20),
        tail: Stub(length: 0.15),
    ),
)
```

**Quer mudar a cabeça do Moluun Cub?** Edite `head_to_body`. Hot-reload pega.
**Quer mudar a forma da orelha?** Edite `ears`. Hot-reload pega.
**Quer adicionar uma forma de orelha nova?** Aí mexe em `src/visuals/dsl/parts/ear.rs`. Recompila uma vez.

---

## 8. Próximas referências necessárias

Pra fechar Parte 2 do documento, preciso de:

- **5-10 pins de bioma/environment**: Verdance (forest), Highlands (cliffs), Abyssal Shallows (caverns), Abyssal Depths (deep ocean) — pixel art retro, paleta consistente com a 6-cores
- **3-5 pins de Adult creature pixel art**: criaturas adultas, ainda warm mas com mais detalhe e proporções menos exageradas que cubs
- **2-3 pins de Elder/wise creature**: vibe "wisdom", paleta lavada, formas mais quietas

Sugestão de busca no Pinterest: "pixel art forest game boy", "gba rpg landscape pixel", "stardew valley pixel scenery", "low res 32x32 elder creature", "ghibli pixel art".

---

## 9. Decisões travadas (2026-05-08)

| Decisão | Resolução |
|---|---|
| Paleta master v1 (26 cores) | ✅ Aprovada |
| Evolução estilística Cub→Adult→Elder | ✅ B + C combinados — `kawaii_factor` contínuo, destino cozy ghibli |
| `art-direction.md` | ✅ Atualizar oficialmente (paralelo a este doc) |
