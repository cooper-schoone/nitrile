# Nitrile

A Rust-based CLI tool for managing LaTeX templates and projects.

**Note:** This project is in active development and some features are not yet implemented; these are marked with *(planned)* below.

## Features

- Simple project compilation via the command line
- Conditional compilation and CLI-injected values via a straightforward interface provided by `nitrile.sty`
- Quick initialization of new Git-based LaTeX project repositories from a default preset or a template
- _(planned)_ Reference management with a dialogue-based interface for entering reference information
- _(planned)_ Per-project TOML-based configuration files

## Requirements

- [Rust](https://www.rust-lang.org/tools/install), if installing from source
- A LaTeX distribution providing `pdflatex`
- The [`etoolbox`](https://ctan.org/pkg/etoolbox) package
  - Usually installed by default but absent from minimal TeX installations such as Debian/Ubuntu's `texlive-latex-base`; install `texlive-latex-recommended` or `etoolbox` via your TeX distribution's package manager if missing

## Installation

### From crates.io

```sh
cargo install nitrile
```

### Prebuilt binaries

```sh
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/cooper-schoone/nitrile/releases/latest/download/nitrile-installer.sh | sh
```

```powershell
# Windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/cooper-schoone/nitrile/releases/latest/download/nitrile-installer.ps1 | iex"
```

Archives also be downloaded directly from the [releases page](https://github.com/cooper-schoone/nitrile/releases).

### From source

```sh
git clone https://github.com/cooper-schoone/nitrile.git
cd nitrile
cargo install --path .
```

## Usage

### `compile`

Compile a LaTeX project.

```sh
nitrile compile [-t <FILE>] [-o <FILE>] [-f <FLAG>]... [-v]
```

| Option | Description |
| --- | --- |
| `-t`, `--target <FILE>` | Target `.tex` file to compile (defaults to `main.tex` or `Main.tex` in the current directory). |
| `-o`, `--output <FILE>` | Path to which the compiled PDF is written (defaults to `build/<target>.pdf`). Must end in `.pdf`. |
| `-f`, `--flag <FLAG>` | Flag passed to the compiler for conditional compilation or value injection. May be repeated. |
| `-v`, `--verbose` | Show the LaTeX engine's output during compilation. |

Example:

```sh
nitrile compile -o output.pdf -f name="John Doe" -f no-show-summary
```

### Flags

Flags come in two forms:

- **String flags** — `key=value` (e.g. `-f name="John Doe"`).
- **Boolean flags** — `key` sets the flag to `true`; the `no-` prefix (`no-key`) sets it to `false`.

Flag keys may contain only ASCII letters, digits, and hyphens.

To use flags, add the bundled [`nitrile.sty`](assets/nitrile.sty) package to your document and declare each flag in the preamble:

```latex
\usepackage{nitrile}

\nitrileflag{name}{Anonymous}
\nitrilebool{show-summary}{false}

\author{\flag{name}}

\ifflag{show-summary}{%
  \section{Summary}\ldots
}{}
```

| Command | Description |
| --- | --- |
| `\nitrileflag{<name>}{<default>}` | Declare a string flag. |
| `\nitrilebool{<name>}{<default>}` | Declare a boolean flag. `<default>` must be `true` or `false`. |
| `\flag{<name>}` | Inject a string flag's value. |
| `\ifflag{<name>}{<then>}{<else>}` | Branch on a boolean flag. |
| `\ifflagset{<name>}{<then>}{<else>}` | True if the flag was supplied on the command line, regardless of its value. |

- Repositories initialized with `nitrile init` come automatically bundled with the `nitrile.sty` package.

### `init`

Initialize a new LaTeX project.

```sh
nitrile init [DIR] [-T <DIR>] [--no-git]
```

| Option | Description |
| --- | --- |
| `DIR` | Directory to initialize the project in (defaults to the current directory). Created if it does not exist. |
| `-T`, `--template <DIR>` | Template project to use as a base. Accepts a local directory path or a remote Git URL (`https://…` or scp-like `user@host:path`). |
| `--no-git` | Skip initializing the project as a Git repository. |

With no template, the project is populated from the default preset containing a basic `main.tex` starter document, the bundled `nitrile.sty` package, and a `.gitignore` preconfigured for LaTeX projects:

```sh
nitrile init my-project
```

When a template is provided, the source is cloned into the specified directory and its Git history is stripped, leaving a clean tree:

```sh
nitrile init -T ~/Documents/Templates/project-template/ my-project
nitrile init -T git@github.com:user/latex-template.git my-project
```

Unless `--no-git` is passed, the resulting directory is initialized as a fresh Git repository. If `git` is not found on `PATH`, this step is skipped — except for remote templates, which require `git` for the cloning process.

### `reference` _(planned)_

```sh
nitrile reference add --format article --author "John Doe" --title "My Article" my-article
nitrile reference remove -F bibliography.bib my-article
```

## Configuration _(planned)_

## License

[MIT](LICENSE.md)
