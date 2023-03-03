# Largo

Largo is a (La)TeX build tool I'm writing to bring an "it just works" experience to my (*my*, not your) TeX projects, namely my papers, talks, and thesis. It was inspired by the eminently pleasant `cargo` build tool for the Rust programming language, and is also written in Rust.

This project is for my personal use. If you have access to this repository, you're free to use it, too, but understand that there is no implied promise of support, or that I'll ever accept pull requests.

## Why a new build tool?
TeX is a staggering achievement of 1970s software engineering that I love to hate. Its tool ecosystem is dominated by janky 1990s Perl scripts, and I'm tired of dealing with all of it. Here are some things I don't want to deal with any more:
* Figure out a sensible project directory structure, add a `.gitignore` file with all the right `.aux`, `.out`, and so on files in it. Write a `Makefile` to clean out all those files when my build gets irrecoverably corrupted for who-knows-what-reason.
* What if I'm making a package, or a Beamer template? What should the project directory look like then? Another 10 minutes of searching.
* Figure out how to use personal TeX packages located at `/some/path` on my machine by setting who-knows-what environment variable.
* What if I'm using *someone else's* package that's not included in my TeX distribution? I download the files, put them in my source directory, and check them into version control? What if a new version comes out?
* What about a CTAN package that *is* included in my TeX distribution, but I have the wrong version? Is there no easy way to override it for a single project?
* More `Makefile` jankery to create separate "`debug`" and "`release`" builds of a document (or "`review`" and "`camera-ready`").
* Building and distributing a bibliography. I keep a big global reference database in Zotero that routinely exports to `/some/local/path/biblio.bib`. Usually, I just want to use _this_ bibliography, and end up symlinking it into my project directory. This is so annoying; I just want `biber` to know about my global bibliography.
* Re-figuring-out almost *all* of the above when I realize I want to switch from `pdflatex` to `xelatex` or whatever.
* Making _other_ tools, like my editor, aware of the cli flags I want for `pdflatex` (or `xelatex` or whatever).
* Having to learn about yet another legacy system from the 80s---that exists to support some platform no one has used in 30 years---every time I want to solve one of these problems.
* Reading errors from `pdflatex` is a total pain!

There are other solutions to a lot of these problems, but they feel complicated to me, and make things harder in the long run.
* [Overleaf](https://www.overleaf.com) is pretty good for sharing, and does solve some of these things, but I want to develop locally. I like my editor, I like version control, I like not needing internet to write, and I like doing things "my way."
* The menagerie of TeX IDEs. Don't even get me started.
* Various scripts like `latexmk` are supposed to be solutions in the same space, more or less, I think. But I've never found one that made my life easier.

## So, what does it do?
Largo solves these problems for me, in my way (not yours):
+ Eliminate source tree pollution in (La)TeX projects
+ Ease the "mixing in" of packages from local directories, git repositories, and so on. Proper dependency management with versions, lockfiles, optional vendoring, etc.
+ Provide a feature flag system to enable multiple build channels for a project.
+ Ease bibliography management with a global bibliography. At some point in the future, I'd like if this could be a file **or** a protocol: communicate with Zotero, Mendeley, etc. This wouldn't be entirely deterministic, but...
+ Backwards-compatibility via `largo eject` subcommand that produces a standalone, reasonably reproducible, `largo`-free TeX project.
+ Support a bunch of TeX systems, meaning different TeX formats/distributions as well as a variety of TeX engines.
+ Do all of this without trying to change how TeX works, or be a new Tex engine, or anything too drastic. Go with the flow.

Largo is basically a wrapper around the existing command-line tools like `pdftex` and `biber`. It (approximately) accomplishes some (soon, all) of these things without trying to change how TeX works, or be a new TeX engine, or anything too drastic. Go with the flow.

## Using Largo

You can create a new TeX project by running `largo new myproject`. This will create a directory `./myproject` that looks like this:
```
myproject
├── largo.toml    // project configuration
├── src           // source directory
│   └── main.tex  // TeX entry point
├── build         // build directory
└── ...           // `git` files, etc.
```
To build the project, you can run

``` shell
cd myproject
largo build
```

Now the build subdirectory will look like,

``` shell
build
└── debug         // default build profile
    ├── main.aux  // 
    ├── ...
    └── main.pdf  // finished artifact
```

where `debug` is the default _build profile_ selected by Largo.

### Largo macros
Largo passes some information about the build to the TeX engine. This information is exposed through a set of Largo user macros:

* `\LargoProfile`: the build profile, _e.g._ `debug` in the example above. This is particularly useful for conditional compilation.
* `\LargoOutputDirectory`: the build directory, _e.g._ `./build/debug/` in the example above.
* `\LargoBibliography`: the global bibliography, if it is configured in `.largo/config.toml`.

## Settings and configuration
### Project settings
`largo.toml`
### Largo configuration
`$HOME/.largo/config.toml`

## Installation
### Cargo
As long as you have `cargo` installed, you can build and install Largo via
``` shell
git clone git@github.com:mcncm/largo
cargo install --path .
```
As long as `$HOME/.cargo/bin` is your `PATH`, you can try running the `largo` command now.

### Nix flake
Alternatively, you may be a masochist. This repository includes a `flake.nix`, so you can include Largo in your flakes-based Nix system by adding

``` nix
largo.url = "git+ssh://git@github.com/mcncm/largo";
```

to the `inputs` of your configuration. Then the Largo binary package is in the attribute `largo.packages.${system}.default`, where `${system}` is _e.g._ `aarch64-darwin`.

## Packages that require care
Some packages don't work "out of the box" with Largo, or need a little massaging.
+ `minted` takes a special option, `outputdir`, that can set as 
  ``` tex
  \usepackage[outputdir=/\LargoOutputDirectory]{}`
  ```
+ `natbib` seems to disagree with `outputdir`. Some important classes---like
  `revtex4`, used for all APS journals---preload `natbib`, so we can't just use
  `biber` for them. This should be solvable *somehow*, but for now it's an incompatibility.

## Similar projects
+ [tectonic](https://tectonic-typesetting.github.io/en-US/) has many of the same goals. It looks pretty neat. But it also tries to reimagine more of how TeX works. It tries to implement fancy things like HTML output. It's very opinionated and cuts against the grain. I found it harder to integrate into the "rest of the world". I just wanted a tool that does what I want as simply as possible.
