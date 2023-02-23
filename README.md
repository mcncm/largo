# Largo

Largo is a (La)TeX build system seeking to bring an "it just works" experience to TeX projects. It was inspired by the eminently pleasant `cargo` build system for the Rust programming language.

This project is for my personal use. You're free to build and use it, but understand that there is no accompanying promise of support.

The name `largo` is a placeholder. There already exist many projects with this name.

## Goals
Largo tries to do the following things:
+ Eliminate source tree pollution in (La)TeX projects
+ Ease the "mixing in" of packages from local directories, git repositories, and other non-official locations.
+ Provide a standardized feature flag system to enable multiple build channels for a project.
+ Built-in dependency management: has your LaTeX distribution ever had a broken version of a package? How did you solve it? What if you could just edit a single line in a config file?
+ Eases bibliography management by allowing a global bibliography. This can be a file, or communicate with Zotero, Mendeley, etc. Talking to a bibliography server over HTTP will not be deterministic, but...
+ Backwards-compatibility via `largo eject` subcommand that produces a standalone, reproducible, `largo`-free TeX project.
  - Writes new bibliography file from global bibliography
  - Vendors packages
+ Possibly replace certain packages with an opinionated alternative. These include,
  + `subfiles`
+ Support a bunch of TeX systems, meaning different TeX formats/distributions as well as a variety of TeX engines.

## Similar projects
+ Tectonic
