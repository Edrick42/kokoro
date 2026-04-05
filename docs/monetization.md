# Kokoro — Monetization Strategy

> Contexto: 5-10h/semana, zero audiência, começa em PT-BR, expande pra EN.
> Objetivo: transformar em renda principal ao longo do tempo.

---

## A Realidade

Com 5-10h/semana e sem audiência, o caminho **não é** lançar tudo de uma vez.
O caminho é uma **escada** onde cada degrau financia e alimenta o próximo:

```
Degrau 1: Conteúdo gratuito (construir audiência)
    ↓
Degrau 2: Produto barato (validar que pessoas pagam)
    ↓
Degrau 3: Produto principal (game + livro)
    ↓
Degrau 4: Comunidade (receita recorrente)
    ↓
Degrau 5: Escalar (EN, mobile, crowdfunding)
```

**Regra de ouro**: Não pule degraus. Cada um leva 2-4 meses no seu ritmo.

---

## Degrau 1: Construir Audiência (Meses 1-3)

### Custo: R$0 | Receita: R$0 | Objetivo: 500 seguidores

Sem audiência, ninguém vai comprar nada. Primeiro, você precisa que pessoas
saibam que Kokoro existe e que **você** é interessante de acompanhar.

### O que fazer:

**Twitter/X** (principal — comunidade dev vive aqui):
- Postar 3-5x por semana
- Conteúdo: progresso do jogo, aprendizados de Rust, clips da criatura
- Formato: thread curta com GIF/video do jogo
- Exemplos de posts que funcionam:
  - "Dia 47 de aprender Rust construindo um pet virtual. Hoje implementei batimento cardíaco." + GIF
  - "Meu bicho virtual aprendeu a recusar comida que ele não gosta. Neural network de 167 parâmetros." + video
  - "A diferença entre ownership e borrowing explicada pelo meu código de crossover genético" + code snippet

**YouTube (PT-BR)** — dev logs curtos (5-10 min):
- Frequência: 1x a cada 2 semanas (cabe em 5-10h/semana)
- Formato: gravação de tela + narração casual
- Não precisa ser polido — autenticidade > produção
- Títulos: "Criando um pet virtual em Rust #1 — O Genoma"
- Cada vídeo mostra algo que funciona no final

**Por que PT-BR primeiro:**
- Pouquíssimo conteúdo de Rust em português (pouca concorrência)
- Comunidade brasileira de dev é enorme e engajada
- Você se diferencia imediatamente — "game dev em Rust" é raro em PT
- Quando for pra EN, já vai ter experiência com o formato

### Métricas de sucesso:
- 500 seguidores no Twitter
- 10 vídeos no YouTube
- 100 inscritos no YouTube
- 50 pessoas numa lista de email

### Como montar a lista de email:
- Crie uma landing page grátis no Carrd.co (R$0)
- Texto: "Kokoro — Um jogo onde cada criatura é biologicamente única. Feito em Rust."
- Botão: "Quero acompanhar o desenvolvimento" → coleta email (Buttondown ou Mailchimp free)
- Link na bio do Twitter e descrição dos vídeos

---

## Degrau 2: Primeiro Produto Pago (Meses 3-5)

### Custo: R$0 | Receita: R$500-2.000 | Objetivo: Validar que pessoas pagam

**Produto: Ebook "Part I — Aprenda Rust Construindo Vida Artificial"**

Você já tem 6 capítulos escritos (Ch1-6). Isso é um produto real.

### Onde vender:

| Plataforma | Preço | Por quê |
|-----------|-------|---------|
| **Gumroad** | R$29-49 | Aceita Real, paga via PayPal/Stripe, 90%+ pra você |
| **Hotmart** | R$29-49 | Maior plataforma BR, afiliados promovem pra você |

### Estratégia de preço:
- **Lançamento**: R$19,90 (preço de validação — "será que alguém paga?")
- **Após 50 vendas**: R$29,90
- **Versão completa**: R$49,90

### O que incluir na Part I:
- 6 capítulos de Rust do zero
- Exercícios práticos (Build It)
- Código fonte do projeto até aquele ponto
- Acesso ao repositório privado
- "Comprou a Part I? A Part II vem com desconto"

### Como vender sem audiência grande:
- Poste no Twitter: "Lancei meu ebook sobre Rust. Me custou 3 meses. Custa menos que um almoço."
- Poste nos subreddits: r/rust, r/gamedev, r/learnprogramming (em inglês — para testar)
- Comunidades BR: TabNews, dev.to em PT, grupos Telegram de Rust/programação
- Cada vídeo no YouTube termina com: "O livro que acompanha esse projeto está em [link]"

### Por que isso funciona:
- Validação real (pessoas pagam → projeto tem valor)
- Receita imediata (mesmo que pouca, é motivadora)
- Conteúdo promove conteúdo (vídeo → livro → jogo)

---

## Degrau 3: O Jogo + Livro Completo (Meses 5-10)

### Custo: R$0-200 | Receita: R$2.000-10.000 | Objetivo: Produto principal no ar

**Dois produtos crescendo em paralelo:**

### O Livro (expansão contínua)
- A cada mês, adicione 2-3 capítulos
- Quem comprou Part I recebe desconto na versão completa
- Preço final: R$49,90 (BR) / $19.99 (EN)
- Quando tiver 20+ capítulos, publique no Amazon KDP (alcance global)

### O Jogo (MVP)
- Desktop primeiro (itch.io, R$0 pra publicar)
- Preço: R$14,90 / $4.99
- Modelo: pague uma vez, tenha tudo (sem microtransações)
- itch.io é a melhor plataforma pra começar (sem aprovação, comunidade indie)
- Mais tarde: Steam ($100 listing fee, alcance muito maior)

### Bundle (o poder do combo):
- "Game + Livro" por R$49,90 / $14.99
- "Compre o jogo, ganhe 50% de desconto no livro"
- "Compre o livro, ganhe acesso antecipado ao jogo"

---

## Degrau 4: Comunidade (Meses 8-12)

### Custo: R$0 | Receita: R$500-2.000/mês recorrente

**Discord** (grátis):
- Canal público: gameplay, screenshots, dúvidas
- Canal exclusivo (quem comprou o jogo/livro): dev updates, vote em features

**YouTube Memberships ou Patreon** (R$5-15/mês):
- Acesso antecipado a novos capítulos
- Dev logs exclusivos (decisões de design, erros, aprendizados)
- Pedidos de feature (votação mensal)

### Quando faz sentido:
- Depois de ter pelo menos 500 compradores (jogo + livro)
- Quando pessoas pedirem "onde posso acompanhar mais de perto?"

---

## Degrau 5: Escalar (Meses 12+)

### Custo: R$100-500 | Receita: potencial R$5.000+/mês

**Tradução pra inglês:**
- Livro em EN no Amazon KDP + Gumroad
- Jogo já está em inglês (código todo em EN)
- Vídeos em EN (ou legendados)
- Mercado 50x maior

**Mobile (iOS + Android):**
- 10x a base de usuários do desktop
- Mesmo preço: $4.99
- cargo-mobile2 já é viável

**Crowdfunding (Kickstarter):**
- Só faz sentido DEPOIS de ter audiência e produto validado
- Com 1.000+ compradores, um Kickstarter de $5K-15K é realista
- Financia: sound design, arte profissional, mobile port

---

## O que NÃO fazer

1. **Não gaste dinheiro antes de validar** — Zero investimento até ter vendas reais
2. **Não lance mobile antes de desktop** — Mobile é mais caro e complexo de manter
3. **Não faça Kickstarter sem audiência** — Crowdfunding sem base = fracasso garantido
4. **Não tente EN e PT ao mesmo tempo** — Domine um mercado primeiro
5. **Não faça vídeos "perfeitos"** — Autenticidade > produção no início
6. **Não crie Discord antes de ter compradores** — Comunidade vazia é pior que nenhuma
7. **Não compare com projetos AAA** — Seu diferencial é profundidade biológica + ensino de Rust, não gráficos

---

## Resumo: O Plano em Uma Página

| Quando | O quê | Receita esperada |
|--------|-------|-----------------|
| **Mês 1-3** | Twitter + YouTube PT-BR + landing page | R$0 (construindo audiência) |
| **Mês 3** | Lançar ebook Part I no Gumroad/Hotmart | R$500-1.000 |
| **Mês 5** | Lançar MVP do jogo no itch.io | R$1.000-3.000 |
| **Mês 6** | Bundle jogo + livro | R$2.000-5.000 |
| **Mês 8** | Livro completo + comunidade Discord | R$1.000-2.000/mês |
| **Mês 12** | Tradução EN + mobile + possível Kickstarter | R$5.000+/mês |

### A única métrica que importa no início:
**"Quantas pessoas abrem o jogo/livro hoje que não abriam ontem?"**

Se esse número cresce toda semana, tudo funciona. Se não cresce, ajuste o conteúdo.

---

## Ferramentas Necessárias (Todas Gratuitas)

| Ferramenta | Pra quê | Custo |
|-----------|---------|-------|
| **Carrd.co** | Landing page | Grátis |
| **Buttondown** | Lista de email | Grátis até 100 subs |
| **Gumroad** | Vender ebook | Grátis (taxa por venda) |
| **itch.io** | Vender jogo | Grátis (defina sua %) |
| **OBS Studio** | Gravar tela pra YouTube | Grátis |
| **DaVinci Resolve** | Editar vídeo | Grátis |
| **Canva** | Thumbnails YouTube | Grátis |
| **Twitter/X** | Rede principal | Grátis |
| **YouTube** | Vídeos | Grátis |
| **Discord** | Comunidade | Grátis |
