# Xargo

Xargo is a (La)TeX build system seeking to bring an "it just works" experience to TeX projects. It was inspired by the eminently pleasant `cargo` build system for the Rust programming language.

This project is for my personal use. You're free to build and use it, but understand that there is no accompanying promise of support.

The name `xargo` is a placeholder. There already exist many projects with this name.

## Goals
Xargo tries to do the following things:
+ Eliminate source tree pollution in (La)TeX projects
+ Ease the "mixing in" of packages from local directories, git repositories, and other non-official locations.
+ Provide a standardized feature flag system to enable multiple build channels for a project.
+ Replace certain packages with an opinionated alternative. These include,
  + `subfiles`

## Similar projects
+ Tectonic
