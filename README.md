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