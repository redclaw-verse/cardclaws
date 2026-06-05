# CardClaws — Design Handoff Brief

**Internal — Confidential (RedClaw Systems LLC).** Not for distribution outside RedClaw.

This brief describes what CardClaws is, every screen, the end‑to‑end flows, the current
visual language, and where we explicitly want the designer to explore alternatives. The goal:
hand this to our AI designer to produce mockups we can compare and iterate on.

---

## 1. Product in one line

**CardClaws is a mobile‑first digital business card.** You build a beautiful, living card
(photo, AI‑generated scene, or AI‑generated video), flip it to reveal a QR code, and anyone who
scans it lands on your public profile with all your links and a one‑tap "Save Contact." It
replaces paper cards with something animated, shareable, and updatable.

**Tone:** premium, modern, a little futuristic. Think Apple Wallet × Linktree × a luxury
business card. Tactile, dark, high‑contrast, motion‑forward.

---

## 2. Who it's for

- **Founders, creators, developers, consultants** who network and want to stand out.
- **Teams** (Pro/Enterprise) who want consistent, branded cards across employees.
- Primary device: **phone, portrait**. The card is meant to be shown screen‑to‑face or scanned.

---

## 3. The core object: "the card"

A card has **two sides** the user flips between with a 3D rotation:

- **Front** = the hero. A full‑bleed **photo, AI image, or looping AI video**, with the person's
  **name + title** overlaid. This is what you show someone.
- **Back** = the **utility**. A single **QR code** ("Scan to view everything") plus a tidy list
  of labeled links (website, GitHub, publications, social, etc.).

The card is **full‑screen / full‑bleed** when presented — it fills the entire device so it reads
as an object, not a UI panel. Flipping is a signature interaction (see §9).

---

## 4. Current visual language (starting point — open to reinvention)

We built a working dark theme. Treat this as **v0 to react to**, not a constraint. We want to
see this look refined *and* 2–3 distinct alternative directions.

| Token | Value | Use |
|---|---|---|
| Background | `#0a0a0c` / `#0e0e12` | App canvas, card back |
| Surface | `#15151a`, `#16161c` | Inputs, tiles, cards |
| Surface raised | `#222228`, `#26262e` | Buttons, chips, icon wells |
| **Accent / CTA** | **`#ff3b30`** (red) | Primary buttons, active chips, selected state |
| Destructive | `#ff453a` | Delete/remove |
| Text primary | `#f5f5f7` | Headings, values |
| Text secondary | `#9a9aa0` | Labels, captions |
| Text tertiary | `#6b6b70` | Placeholders, hints |
| Radius | 12–20 px (tiles/inputs), **0 px** (full‑bleed card) | — |
| Type | System (SF/Roboto), weights 600–800 for emphasis | — |

**Motion is part of the brand.** Flip animation, ambient idle drift, entry fades, animated
gallery tiles. Designs should account for movement, not just static frames.

**Exploration we want:** alternate accent palettes (e.g., electric violet, neon cyan, warm gold),
light‑mode option, glassmorphism vs. flat vs. neumorphic card surfaces, and 2–3 card‑back layouts.

---

## 5. Screen‑by‑screen

> Screens marked **[built]** exist in the working app; **[roadmap]** are in the product vision
> (PRD) and should be mocked for the full experience.

### 5.1 Gallery — "Your cards" **[built]** (home screen)
The landing screen: a grid of the user's cards.

- Header: title "Your cards" + a "Sign in" affordance (top‑right).
- **Responsive grid that shrinks as it grows:** 2 columns for a few cards, 3 for more, 4 for many
  — tiles get smaller so more fit per screen.
- **Tiles:** portrait (2:3) card thumbnails. Photo cards show the still; **video cards animate**
  (a short looping flipbook of frames) and carry a small ▶ badge so they're distinguishable.
- Tile label: name + title (hidden on the smallest tiles).
- Floating **"+ New card"** button (bottom center).
- Empty state: "No cards yet — create your first."
- Tap a tile → opens that card (view/edit). 

*Explore:* grid vs. carousel vs. stacked‑wallet metaphor; how a "live" video tile should feel
without being noisy.

### 5.2 Card editor — "Make your card" / "Edit card" **[built]**
Where a card is composed. Vertical form.

- **Live preview** at top (the chosen photo / AI image / looping video, 2:3).
- **Front media row 1:** `Take photo` · `Choose photo`.
- **Front media row 2 (AI):** `✨ AI scene` (magic‑wand icon) · `🎬 AI video` (movie icon).
- **Fields:** Name, Title, QR link (the profile URL).
- **"Back of card · scannable links":** a repeatable list editor — each entry has a **kind**
  (Domain / GitHub / Paper / Link — sets its icon), a **label**, and a **URL**. Add / remove rows.
- Primary CTA: **Save card** (red). For existing cards, a **Delete card** link.

*Explore:* progressive disclosure (collapse AI + links until needed), a segmented "Front /
Back / Details" layout, inline validation, a clearer "this becomes your card front" framing.

### 5.3 AI scene generator **[built]** (single screen)
Reached from `AI scene`. Generates the card‑front **image** with Google's nano‑banana model.

- "Describe your scene" — multiline text (e.g., *"a neon‑lit Tokyo street at night"*).
- "Style" chips: Cinematic · Minimal · Neon · Studio portrait · Nature · Abstract.
- "Mood" chips: Bold · Calm · Luxe · Playful · Dark.
- CTA: **Generate image** (shows a spinner; refines the brief with AI, then generates). On
  success it returns to the editor with the image on the card front.

*Explore:* show example thumbnails per style; a "regenerate / variations" row; a before/after.
*(Note: a voice‑to‑text mic for the description is planned but not yet shipped.)*

### 5.4 AI video generator **[built]** (single screen)
Reached from `AI video`. Generates a looping card‑front **video** with Google Veo.

- "Describe the motion" — multiline (e.g., *"slow drift across a city skyline as lights flicker on"*).
- "Motion style" chips: Parallax · Particles · Slow zoom · Liquid · Aurora · Glitch.
- "Length" chips: 3s · 5s · 8s.
- CTA: **Generate video.** This is **async (~1 minute)** — needs a clear, reassuring progress
  state ("Generating with Veo — this takes about a minute…"). On success the looping clip lands
  on the card front.

*Explore:* a richer waiting state (progress %, animated placeholder, "we'll notify you"), and how
to preview/scrub the result before accepting.

### 5.5 Card viewer (full‑screen) **[built]** — the signature moment
Tapping a saved card (or "Save") presents it **full‑bleed**, status bar hidden.

- **Front:** the photo/AI‑image/looping video fills the screen; **name + title stacked top‑right**.
  A subtle hint: "Swipe or tap the card to flip →".
- **Flip:** swipe or tap → 3D `rotateY` flip (perspective, mid‑flip face swap, haptic tick).
- **Back (the template):**
  - Identity header (name + title).
  - **One hero QR** on a white tile — caption "Scan to view everything" + the profile URL.
  - A divider, then the **link list**: each row = kind icon + label + URL.
  - Controls (Gallery · Edit) appear **only on the back**, so the front stays clean.
- Ambient idle: a gentle drift after a few seconds of no interaction (premium "alive" feel).

*Explore:* multiple **card‑back templates/themes** (this is explicitly requested — give us 3–4:
e.g., "minimal", "grid of links", "social‑first", "portfolio"), QR styling (rounded modules,
logo in center, color), and the flip's easing/feel.

### 5.6 Share sheet **[roadmap]**
How others receive the card without scanning in person:

- **QR** (the back), **share link** (`cardclaws.com/handle`), **NFC tag** write (tap a physical
  sticker/card), **Add to Apple Wallet / Google Wallet** (the card as a wallet pass).

### 5.7 Auth **[roadmap, exists in backend]**
Sign up / sign in: **email + password, magic link, Sign in with Apple.** Minimal, trust‑building.

### 5.8 Public web profile **[roadmap]** — `cardclaws.com/[handle]`
The page a scanner lands on (responsive web, not the app):

- Hero recreation of the card (with the flip), name/title/company.
- Sticky **contact‑actions bar**: Save Contact (.vcf), Call, Email, Map.
- The same links as the card back.
- Should feel like a continuation of the card, on the web.

### 5.9 Analytics **[roadmap]**
A simple dashboard per card: profile visits, contact saves, scans over time, rough geo. Clean,
glanceable, not enterprise‑heavy.

### 5.10 Teams / settings / billing **[roadmap]**
- Tiers: **Free** (1 card), **Pro** (multiple cards, AI, video, analytics), **Team/Enterprise**
  (seats, roles, central branded templates, aggregate analytics).
- Paywall / upgrade screens; team roster; brand template management.

---

## 6. Primary end‑to‑end flows (storyboard these)

1. **First card (photo):** Gallery (empty) → New card → take/choose photo → fill name/title →
   add a couple links → Save → full‑screen card → flip to QR.
2. **AI image card:** New card → AI scene → describe + style + mood → Generate → image on front →
   Save → present.
3. **AI video card:** New card → AI video → describe + motion + length → Generate (wait ~1 min) →
   looping video front → Save → present.
4. **Show & share:** open a card → present front → flip → someone scans the QR → lands on the web
   profile → taps Save Contact.
5. **Manage many:** Gallery fills up → tiles shrink to fit → video tiles animate to stand out →
   tap to edit/delete.

---

## 7. Animation & interaction notes (brand‑critical)

- **Flip:** 3D rotateY with perspective, configurable ~250–600 ms, target <450 ms; haptic on flip.
- **Entry:** card fades/scales in on present.
- **Ambient:** slow idle drift after ~5 s of inactivity.
- **Gallery video tiles:** ~2 fps frame flipbook (subtle, not distracting).
- **Generate states:** image = seconds (spinner); video = ~1 min (needs a real, calm progress UX).
- Everything should feel **fast, tactile, and physical** — the card is a real object.

---

## 8. Component inventory (for a design system)

Buttons (primary/secondary/destructive), selectable **chips**, text inputs (single + multiline),
the **card tile** (photo + animated video variants + badge), the **full‑bleed card** (front +
back faces), **QR tile**, **link row** (icon + label + URL), **link editor row** (kind chips +
inputs + remove), floating action button, section headers, empty states, progress/loading states,
nav header. Icon set currently from Material Community Icons (web, github, book, link, microphone,
movie, play, magic‑wand).

---

## 9. Hard constraints (don't break these)

- **Mobile‑first, portrait.** The card front is **full‑bleed** (no chrome) when presented.
- **One QR per card** on the back (links are listed, not individually coded).
- Front carries **name + title**; back carries the **QR + links**.
- The flip is the hero interaction — design around it.
- Dark is the default we've built, but **propose a light mode** too.

## 10. What we want from the designer

1. A **refined version** of the current dark direction (polish what exists).
2. **2–3 alternative visual directions** (different palette/material/personality).
3. **3–4 card‑back templates/themes** to choose from.
4. Mockups for every **[built]** screen + the key **[roadmap]** ones (web profile, share, auth).
5. The **5 flows** in §6 as click‑through storyboards.
6. Treatments for **motion** (flip, ambient, generate/progress, animated gallery tiles).

Deliver as comparable options so we can A/B the looks quickly.

---

## 11. Post‑scan experiences (concepts to mock)

The moment someone **scans the QR** is the product's payoff. Today it just opens the web profile
(§5.8). We want to explore *designed experiences* triggered on scan — each is a **distinct
landing‑page look**, so these double as design directions to compare. Mock **3–4** of these as
alternative post‑scan screens (mobile‑web, portrait, fast‑loading). The default polished profile +
"Save Contact" is the baseline; these are modes on top of it.

> Architecturally these all ride one lever: the scan can carry an *experience + context* selector,
> so one card triggers many experiences. Designers don't need to worry about that — just treat each
> as its own landing screen.

**A · Remember the moment**
1. **"We Met"** — a two‑way, time/place‑stamped meeting ("Met at SXSW · Austin · Jun 5"); both
   people get added to each other's "People I met." Reciprocal share‑back. *Design:* a warm
   confirmation + connection card + a list.
2. **Auto follow‑up** — schedule a nudge or send a "great meeting you" with your links. *Design:* a
   tiny scheduling/confirmation moment.
3. **Memory note** — a one‑line context ("the climber building fintech") attached to the connection.

**B · Adapt to the moment**
4. **One QR, many faces** — work / social / creator modes of the same person. *Design:* show how the
   profile re‑skins per mode.
5. **Event companion** — event‑themed profile + event CTAs ("Catch my 3pm talk, Room B").
6. **Geo / locale‑aware** — localized language/links based on where the scanner is.

**C · Deliver the goods**
7. **"Book me"** — live availability → pick a slot → on both calendars. *Design:* an inline
   booking widget.
8. **Payload drop** — the scan hands over the *thing* (deck, resume, menu, discount, playlist,
   tip/pay link). *Design:* a focused "here's your file/offer" screen.
9. **Live "now" feed** — your latest post/video/status, always fresh. *Design:* a feed/now block.

**D · Delight & spread**
10. **AI welcome** — a personalized AI image/video greeting plays on the landing (powered by our
    nano‑banana + Veo). *Design:* a cinematic hero moment.
11. **AR / collectible** — AR card/avatar in the camera, and/or a collectible deck of people you've
    met (rare/holographic variants). *Design:* AR overlay + a collection/“deck” view.
12. **Surprise & delight** — confetti, a custom line, an event micro‑site, or a tiny quiz that ends
    in your contact. *Design:* a playful, reactive landing.

**Recommended to mock first:** #1 "We Met", #7 "Book me", #10 AI welcome — they cover relationship,
conversion, and our AI differentiator.
