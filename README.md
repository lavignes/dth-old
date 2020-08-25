# Down The Hatch

A rogue-like game that I will probably never finish.

## Interesting Code

* Hand-rolled math in [dth::math](./src/math):
    - [Vectors](./src/math/vector.rs)
    - [Matrices](./src/math/matrix.rs)
    - [Quaternions](./src/math/quaternion.rs)

* Exotic data structures in [dth::collections](./src/collections):
    - Packed integer arrays: [dth::collections::PackedIntVec](./src/collections/packed_int_vec.rs)
    - Indexed bitmaps: [dth::collections::PaletteVec](./src/collections/palette_vec.rs)
    - Indexed cube-maps: [dth::collections::cube_map](./src/collections/cube_map.rs)
    - Object pools with dumb hashes: [dth::collections::HashPool](./src/collections/hash_pool.rs)