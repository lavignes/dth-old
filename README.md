# Down The Hatch

A rogue-like game that I will probably never finish.

## Interesting Code

* Hand-rolled math in [dth::math](./src/math):
    - [Vectors](./src/math/vector.rs)
    - [Matrices](./src/math/matrix.rs)
    - [Quaternions](./src/math/quaternion.rs)
    
* Fast frustum culler: [dth::gfx::Frustum](./src/gfx/frustum.rs)

* Exotic / fun data structures in [dth::collections](./src/collections):
    - Packed integer arrays: [dth::collections::PackedIntVec](./src/collections/packed_int_vec.rs)
    - Bitmaps: [dth::collections::bitmap](src/collections/bitmap.rs)
    - Indexed bitmaps: [dth::collections::PaletteVec](./src/collections/palette_vec.rs)
    - Indexed cube-maps: [dth::collections::CubeMap16](./src/collections/cube_map.rs)
    - Object pools with dumb hashes: [dth::collections::HashPool](./src/collections/hash_pool.rs)