# Formatter

`shuck-formatter` formats shell source from the shared AST and parser. The CLI's `shuck format` and embedders use the same entry points.

## Entry points

- `format_source(source, path, options) -> Result<FormattedSource>` parses, then formats.
- `format_file_ast(source, file, path, options) -> Result<FormattedSource>` formats an already-parsed `File` and never re-parses.
- `format_range(source, range, path, options) -> Result<Option<FormattedRange>>` expands a range to the smallest enclosing complete statement.
- `source_is_formatted(source, path, options) -> Result<bool>` backs `--check`.

`FormattedSource` is `Unchanged` or `Formatted(String)`. The formatter compares rendered output to the input; equal bytes mean `Unchanged`.

## Pipeline

`format_source` resolves the dialect and parses. `format_file_ast` receives the AST and resolves formatting options. Both then:

1. reject files beyond the statement-nesting depth guard
2. run the simplify pass if requested
3. build `FormatterFacts` once for the file
4. resolve the line ending from the indexed source
5. render through a `RenderContext` that exposes only the source, options, and facts

Rendering has two modes. `BufferedRender` writes to a string. `CompareRender` streams against the source and never allocates the formatted document. `--check` uses `CompareRender`, so a formatted file costs no output allocation.

`FormatterFacts` owns layout decisions shared across statements, branches, case items, comments, heredocs, and compound closes, including indentation, padding, line breaks, and verbatim regions. The word and command layers consume facts rather than re-deriving context. Nested `$()` bodies render through the same statement-sequence formatter; backticks and heredoc bodies fall back to verbatim paths where needed. The depth guard is the only structural refusal; malformed input is a parse error, and unsupported layouts use a verbatim fallback.

Comments come from `shuck_indexer::CommentIndex` and are attached by `SourceMap` and the comment-attachment model before rendering.

## Options

`ShellFormatOptions` carries the dialect, indent style and width, binary-operator placement, case-body indentation, redirect spacing, padding, function-brace placement, never-split, simplify, and minify. `FormatSettingsPatch` applies per-field overrides with the precedence CLI > `[format]` > defaults.

`ShellDialect` is `Auto`, `Bash`, `Posix`, or `Mksh`. `sh` and `dash` resolve to POSIX, `bash` to Bash, and `mksh` to Mksh; `ksh` has no formatter dialect. Inference is shebang, then extension, then Bash.

Config keys under `[format]`: `exclude`, `indent-style`, `indent-width`, `binary-next-line`, `switch-case-indent`, `space-redirects`, `keep-padding`, `function-next-line`, and `never-split`. `simplify` and `minify` are CLI-only. `[format].dialect` is rejected; use `[per-file-shell]` or `shuck format --dialect`.

`format.exclude` applies to directory discovery and explicitly named files, so generated files stay untouched.

## Simplify and minify

`simplify_file(&mut File, source)` applies fixed-order rewrites: parenthesis cleanup, arithmetic variable simplification, conditional cleanup, nested-subshell flattening, and quote tightening. The report lists applied rewrite names. Rewrites mutate the owned tree before facts are built, so facts always describe the simplified tree. `minify` implies simplify and drops comments and unnecessary layout.

## CLI and cache

`shuck format` supports in-place, `--check`, `--diff`, stdin with `--stdin-filename`, `--dialect`, the layout flags, `--simplify`, and `--minify`. `--check` and `--diff` exit 1 when files would change and leave files untouched; formatter parse errors exit 2.

Format results are cached per file in the shared cache root. The cache key includes the resolved format settings and a project-format tag, so changing options invalidates entries. `--no-cache` bypasses reads and writes.

## Performance state

The direct-write renderer, unified nested command-substitution formatting, and the `FormatterFacts` sidecar have landed. Parser and CLI reuse across files has not landed: each `format_source` builds a parser and the CLI batches files through the generic project runner rather than a formatter session. Formatter benchmarks live in `crates/shuck-benchmark/benches/formatter.rs` and the formatter macro scripts under `scripts/benchmarks/`.
