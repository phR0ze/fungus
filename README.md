# fungus
[![license-badge](https://img.shields.io/crates/l/fungus.svg)](https://opensource.org/licenses/MIT)
[![build](https://github.com/phR0ze/fungus/workflows/build/badge.svg?branch=main)](https://github.com/phR0ze/fungus/actions)
[![codecov](https://codecov.io/gh/phR0ze/fungus/branch/main/graph/badge.svg?token=2Q81XD9WU1)](https://codecov.io/gh/phR0ze/fungus)
[![crates.io](https://img.shields.io/crates/v/fungus.svg)](https://crates.io/crates/fungus)
[![Minimum rustc](https://img.shields.io/badge/rustc-1.30+-lightgray.svg)](https://github.com/phR0ze/fungus#rustc-requirements)

***Rust utilities to reduce code verbosity***

***fungus*** is a collection of convenience functions I built up while working on other projects.
I always seem to write this kind of boiler plate code to make working with a system more ergonomic
and decided to make it reusable.

## intents <a name="intents"/></a>
fungas attempts to follow these intents:

* ***Chaining*** - ensure Rust's functional chaining style isn't impeded by additions
* ***Brevity*** - keep the naming as concise as possible while not infringing on clarity
* ***Clarity*** - keep the naming as unambiguous as possible while not infringing on brevity
* ***Performance*** - keep convenience functions as performant as possible while calling out significant costs
* ***Speed*** - provide ergonomic functions similar to rapid development languages
* ***Comfort*** - use naming and concepts in similar ways to popular languages

### Quick links
* [Usage](#usage)
  * [Rustc requirments](#rustc-requirements)
* [Contribute](#contribute)
  * [Git-Hook](#git-hook)
* [License](#license)
  * [Contribution](#contribution)
* [Backlog](#backlog)
* [Changelog](#changelog)

# Usage <a name="usage"/></a>

#### Requires rustc >= 1.30 <a name="rustc-requirements"/></a>
This minimum rustc requirement is driven by the enhancements made to [Rust's `std::error::Error`
handling improvements](https://doc.rust-lang.org/std/error/trait.Error.html#method.source)

TBD

## Contribute <a name="Contribute"/></a>
Pull requests are always welcome. However understand that they will be evaluated purely on whether
or not the change fits with my goals/ideals for the project.

### Git-Hook <a name="git-hook"/></a>
Enable the git hooks to have automatic version increments
```bash
cd ~/Projects/fungus
git config core.hooksPath .githooks
```

## License <a name="license"/></a>
This project is licensed under either of:
 * MIT license [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT
 * Apache License, Version 2.0 [LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0

### Contribution <a name="contribution"/></a>
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.

---

## Backlog <a name="backlog"/></a>
* Update documentation

## Changelog <a name="changelog"/></a>
* 12/30/2020
  * Updating rustc minimum version explanation
* 12/29/2020
  * Split Arch Linux functionality out to [relic](https://crates.io/crates/relic)
* 12/28/2020
  * Split git2 functionality out to [skellige](https://crates.io/crates/skellige)
* 12/26/2020
  * Get fungus building and reporting with github actions
  * Split out git and arch work into another project
  * Implemented a better `defer` function
  * Port system utilities from witcher
  * Use fastrand rather than rand
