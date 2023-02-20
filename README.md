# Xargo

Xargo is a (La)TeX build system seeking to bring an "it just works" experience to TeX projects. It was inspired by the eminently pleasant `cargo` build system for the Rust programming language.

This project is for my personal use. You're free to build and use it, but understand that there is no accompanying promise of support.

The name `xargo` is a placeholder. There already exist many projects with this name.

## Goals
Xargo tries to do the following things:
+ Eliminate source tree pollution in (La)TeX projects
+ Ease the "mixing in" of packages from local directories, git repositories, and other non-official locations.
+ Provide a standardized feature flag system to enable multiple build channels for a project.
+ Eases bibliography management by allowing a global bibliography. This can be a file, or communicate with Zotero, Mendeley, etc. Talking to a bibliography server over HTTP will not be deterministic, but...
+ Backwards-compatibility via `xargo eject` subcommand that produces a standalone, reproducible, `xargo`-free TeX project.
  - Writes new bibliography file from global bibliography
  - Vendors packages
+ Replace certain packages with an opinionated alternative. These include,
  + `subfiles`

## Similar projects
+ Tectonic
