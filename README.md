# fungus
[![license-badge](https://img.shields.io/crates/l/fungus.svg)](https://opensource.org/licenses/MIT)
[![build](https://github.com/phR0ze/fungus/workflows/build/badge.svg?branch=main)](https://github.com/phR0ze/fungus/actions)
[![codecov](https://codecov.io/gh/phR0ze/fungus/branch/master/graph/badge.svg?token=2Q81XD9WU1)](https://codecov.io/gh/phR0ze/fungus)
[![crates.io](https://img.shields.io/crates/v/fungus.svg)](https://crates.io/crates/fungus)
[![rust-version](https://img.shields.io/badge/rust-latest%20stable-blue.svg)](https://github.com/rust-lang/rust/releases)

***Rust utilities to reduce code verbosity***

The intent of `fungus` is to reduce code verbosity for common system functions to keep things DRY.

TBD

### Quick links
* [Usage](#usage)
* [Contribute](#contribute)
  * [Git-Hook](#git-hook)
* [License](#license)
  * [Contribution](#contribution)
* [Backlog](#backlog)

# Usage <a name="usage"/></a>
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

## Completed <a name="completed"/></a>
* 12/26/2020
  * implemented a better `defer` function
  * port system utilities from witcher
  * use fastrand rather than rand
