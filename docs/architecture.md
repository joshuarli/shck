# Architecture

Shuck is a Cargo workspace. The CLI composes a shell parser, a positional indexer, a semantic model, a lint engine, a formatter, and a managed interpreter runner.

## Crates

| Crate | Role |
| --- | --- |
| `shuck-cli` | Binary entry point. Discovers inputs, resolves config, coordinates caching, runs check/format/run, applies fixes, and renders reports. |
| `shuck-config` | `.shuck.toml` shapes, config loading, settings resolution, project-root discovery, and source-path resolvers. |
| `shuck-discover` | Input discovery: extension and shebang detection, default ignored directories, gitignore and `.ignore` handling. |
| `shuck-parser` | Source-backed lexer and recursive-descent parser producing the AST and recovery diagnostics. |
| `shuck-ast` | AST node types, tokens, names, spans, and arena primitives. |
| `shuck-indexer` | Line, comment, region, continuation, and optional editor-range indexes over a parsed file. |
| `shuck-semantic` | Scopes, bindings, references, declarations, source references and closure, call graph, CFG, dataflow, and value flow. |
| `shuck-linter` | Rule registry and checker dispatch, lint facts, diagnostics, suppressions, and fixes. |
| `shuck-formatter` | Shell formatting over the shared AST and parser. |
| `shuck-cache` | File-level result caching keyed by path, mtime, permissions, and effective settings. |
| `shuck-run` | Managed shell interpreter registry, resolution, download, verification, and installation. |
| `shuck-benchmark` | Vendored fixtures, rustybench micro-benchmarks, and macro-benchmark support. |

## Check pipeline

1. **Discover** (`shuck-discover`). Walk inputs; accept shell extensions and shebangs; skip default ignored directories and ignored files. Explicitly named non-shell files are skipped.
2. **Resolve settings** (`shuck-config`). Find the nearest `.shuck.toml` or `shuck.toml` above each input, merge it with CLI overrides, and build `ResolvedCheckSettings` plus the effective cache key.
3. **Cache lookup** (`shuck-cache`). Reuse a cached result when the file and its dependency fingerprints still match.
4. **Parse and index** (`shuck-parser`, `shuck-indexer`). Infer the dialect, parse into a `File` with recovery diagnostics, then build positional indexes.
5. **Analyze** (`shuck-semantic`, `shuck-linter`). Build the semantic model, apply ambient contracts and source closure, build lint facts, dispatch enabled rules, and filter suppressions and per-file ignores.
6. **Fix, report, cache**. Apply requested fixes, render the selected output format, write the cache entry, and return the exit status.

Files connected by `source` statements or `# shuck: source=` directives participate through source closure; `lint=true` targets are added as extra inputs. Watch mode tracks direct dependencies and re-runs on changes.

## Parser

`shuck-parser` tokenizes with `Lexer` and builds the AST with a recursive-descent `Parser`. Both live under `crates/shuck-parser/src/parser/`; the parser is split by construct under `commands/`, `words/`, `word/`, and the lexer under `lexer/`.

- `Parser::parse` is the single parse entry and returns `ParseResult { file, diagnostics, status, terminal_error, syntax_facts }`.
- `ParseStatus` is `Clean`, `Recovered`, or `Fatal`. Recovery records a diagnostic and skips to the next command boundary, so rules can still analyze the surrounding script.
- Dialects are `Posix`, `Mksh`, and `Bash`. `ShellProfile` and `DialectFeatures` gate constructs such as `[[ ]]`, arithmetic commands, `function`, `select`, and `coproc`.
- Work is bounded by a shared `ParseBudget`: fuel of `max(100_000, input length)`, a default AST depth of 100, a 1536 KiB native-stack budget, an operator-chain limit of 1000, and a substitution-nesting limit of 50. Oversized input and exhausted limits produce a fatal result rather than uncontrolled recursion.
- Alias expansion, heredoc decoding, and source-text re-parsing run through nested parser state with the same budget.

AST invariants:

- Shape is `File -> StmtSeq -> Stmt -> Command`. `and`, `or`, `|`, and `|&` chains are `Command::Binary` nodes; there is no separate pipeline or command-list type.
- Redirects, `!` negation, terminators, and inline comments belong to `Stmt`, not to the inner command.
- Declaration builtins are `Command::Decl(DeclClause)` with typed operands, not simple commands matched by name.
- Comments are AST-owned (`Comment { range }`) and sliced from the source, so no side channel is needed.
- Words are a sequence of span-carrying parts. Variable-bearing parts hold `VarRef` values with typed subscripts, and arithmetic fragments carry both source text and a typed expression tree.
- Heredoc bodies are decoded into parts at parse time. Redirect targets distinguish words from heredocs.
- Spans are source offsets plus 1-based line and column positions. `Position`, `Span`, `TextRange`, and `TextSize` are defined in `shuck-ast`.

Locations: `crates/shuck-parser/src/parser/{lexer,commands,words,word,heredocs,redirects,arithmetic,recovery,limits}.rs`, AST types in `crates/shuck-ast/src/{ast,span,tokens,name}.rs`.

## Indexer

`Indexer` pre-computes positional and structural facts so rules and the formatter do not re-walk the AST for basic context. It borrows nothing from the parse result after construction.

- `LineIndex` maps offsets to 1-based lines, line ranges, and column offsets; it also records line-ending style and raw continuation candidates.
- `CommentIndex` holds `IndexedComment { range, line, is_own_line }` and answers comment lookups by line and offset.
- `RegionIndex` answers innermost-region queries for single quotes, double quotes, heredocs, command substitutions, arithmetic, and conditionals. Ranges are half-open and include delimiters. It also tracks quoted coverage, backtick substitutions, dollar-brace pairs, and heredoc closing markers.
- Continuation lines are backslash-newline candidates that are not inside a comment, heredoc, or quoted region.
- `IndexerOptions` gates close-delimiter, folding-range, and selection-range indexes; all are off by default and are available for editor-oriented range queries.

Locations: `crates/shuck-indexer/src/{lib,line_index,comment_index,region_index,close_delimiter_index,folding_range_index,selection_range_index}.rs`.

## Semantic model

`SemanticModel::build(&File, source, &Indexer)` performs one evaluation-order traversal. Function bodies are deferred until the enclosing scope has executed, so declarations see the environment the shell would see. The model itself is immutable; expensive analyses sit behind `OnceLock` on `SemanticAnalysis`.

Scopes:

- `ScopeKind` is `File`, `Function`, `Subshell`, `CommandSubstitution`, or `Pipeline` (one scope per pipeline segment). Brace groups do not create a scope.
- Bindings and references are per scope; resolution walks ancestor scopes for the latest same-name binding at or before the reference.

Model contents:

- Bindings cover assignments, declarations, loop variables, read/mapfile/printf/getopts targets, arithmetic assignments, namerefs, function definitions, and imported bindings. Attributes are bitflags such as exported, local, readonly, array, nameref, imported, and externally consumed.
- References record their kind and span; unresolved references are retained for undefined-variable rules.
- Declarations preserve declaration builtins and their operands, including flags and assignment metadata.
- Source references classify literal paths, `# shuck: source=` directives, `/dev/null`, dynamic paths, and single-variable static tails, along with resolution state and whether the reference is conditionally executed.
- The call graph records call sites, reachable functions, overwritten functions, and unreached functions with reasons. It is built lazily and invalidated when contracts add bindings.
- The control-flow graph is a single graph per model. Function bodies are located through `scope_entries`, `scope_exits`, and recorded function-body scopes; subshell, pipeline, and command-substitution bodies are isolated regions joined with `NestedRegion` edges.
- Dataflow uses dense bitsets over the CFG for reaching definitions, uninitialized reads, unused assignments, and dead code. `SemanticAnalysis` also exposes scope-read plans and value-flow queries used by quoting and helper rules.
- `ShellBehaviorAt` is threaded through facts for behavior-sensitive decisions. It currently reports fixed default behavior regardless of shell or option state; option-state tracking (`setopt`, zsh options, `emulate`) is not implemented.

Construction order: build the file scope, parse source directives, traverse statements and words, complete scopes, visit deferred function bodies, seed heuristic unused-assignment facts, then apply the file-entry contract, collect source closure, and apply source contracts. Cross-file effects flow through `FileContract` and `SourceClosureContracts`.

Locations: `crates/shuck-semantic/src/{lib,scope,binding,reference,declaration,source_ref,source_resolve,call_graph,cfg,dataflow,value_flow,glob,runtime}.rs`, builder in `crates/shuck-semantic/src/builder/`.

## Arena primitives

`shuck-ast::arena` provides `Idx<T>`, `IdRange<T>`, and `ListArena<T>` with `u32` indexes; overflow panics at construction instead of truncating. Linter facts use `ListArena` and `IdRange` for packed command child lists while still borrowing AST nodes. The parsed AST itself remains a recursive `Vec`/`Box` tree.
