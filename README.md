# bevy_trickfilm

[![crates.io](https://img.shields.io/crates/v/bevy_trickfilm)](https://crates.io/crates/bevy_trickfilm)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![docs.rs](https://docs.rs/bevy_trickfilm/badge.svg)](https://docs.rs/bevy_trickfilm)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/bevyengine/bevy#license)

| bevy | bevy_trickfilm |
|------|----------------|
| 0.12 | 0.4.0+         |
| 0.11 | 0.3.0          |
| 0.10 | 0.2.0          |
| 0.9  | 0.1.0          |

## What is bevy_trickfilm?

Simple plugin to load spritesheet animations from manifest files written in ron. The animations are not directly tied to a certain sprite sheet.
You can combine this with plugins that add the ability to load a texture atlas from a manifest file. For example: [bevy_titan](https://github.com/KirmesBude/bevy_titan) or [bevy_heterogeneous_texture_atlas_loader](https://github.com/ickshonpe/bevy_heterogeneous_texture_atlas_loader).

## How to use?

Check the examples for usage.

## License

bevy_trickfilm is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!

Some of the code was adapted from other sources.
The [assets](assets) included in this repository fall under different open licenses.
See [CREDITS.md](CREDITS.md) for the details of the origin of the adapted code and licenses of those files.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
