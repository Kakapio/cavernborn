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

## Gifs

![NVIDIA_Overlay_QVmhZrc9eN](https://github.com/user-attachments/assets/ec518194-817a-4ec0-ab04-b04204762bd1)
![NVIDIA_Overlay_0HxQGyfGus](https://github.com/user-attachments/assets/1da34782-cb20-41d7-bd6a-fe0a0675bfa3)
