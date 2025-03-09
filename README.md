# cavernborn

A 2D mining game where every pixel is simulated.

## Setup

Run `tools/setup.sh`

## Particle Spritesheet

The code expects the pixel at 0,0 to be a value of rgba(0,0,0,0) as we use it for air.
Check `to_spritesheet_indices` in `Chunk.rs`.

## To use cargo flamegraph (windows)

First set these variable in shell env: 
$env:RUSTFLAGS="-C force-frame-pointers=y"
$env:CARGO_PROFILE_RELEASE_DEBUG="true"

Then run: cargo flamegraph -c "record -g"

# Notes:

## Core Idea
What if we combine elements of *Valheim* with a falling sand simulation, similar to *Terraria*?  

Maps could be composed of a series of **dungeons** made of **unbreakable particles**, with the gaps filled in by our simulated particles.  

The player starts with a **basic pickaxe** in an upper biome and progresses deeper as they upgrade their gear.

---

## Progression
1. You **can't go deeper** without a better pickaxe. *(Locked progression)*
2. You **can't craft better equipment** without materials from deeper biomes.
3. **Dungeons drive progression**—essential materials and upgrades are locked behind them.
4. **Boss fights** must be defeated to unlock the next biome.

---

## Gameplay
1. **Particle simulation should only be ~50% of the game.**
   - Players **cannot dig** into/around dungeons.
   - Combat should focus on **weapons**, with limited physics-based attacks (e.g., fire grenades).
   
2. **Game length & exploration.**
   - *Valheim* encourages exploration through sailing; in *Cavernborn*, **dungeons** serve that purpose.
   - Likely a **shorter experience than Valheim**, but still persistent and exploration-driven.
   
3. **Building & base management.**
   - Players **must be able to build bases**.
   - How do we prevent **particles from destroying** player bases?
   - Can we still allow for **raids** or environmental threats?

---

## What's Different from Terraria?
While *Cavernborn* has some similarities to *Terraria*, key differences include:
- **True falling sand physics** instead of static terrain.
- **Unbreakable dungeons** that force structured exploration.
- **Persistent physics-based world changes** (cave-ins, flooding, erosion).

---

## Graphics & Optimization
1. Convert **particle chunks into proper meshes** for smoother rendering and more interesting visuals.

---

## Dungeon Exploration as the Main Driver
If dungeons replace *Valheim’s* sailing, they need to be **varied and rewarding**.  
Each dungeon could introduce unique mechanics:

- **Sand-Filled Labyrinth** → Removing too much sand collapses paths.
- **Lava Cavern** → Digging too much allows magma to flood areas.
- **Crystal Cavern** → Certain materials affect physics (e.g., floating sand).
- **Boss fights should tie into terrain physics** (e.g., burrowing enemies that shift the battlefield).

---

This setup keeps the game focused on **exploration, survival, and physics-based interactions**, while ensuring a structured, rewarding progression system.
