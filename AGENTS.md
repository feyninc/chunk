# chunk

Fast text chunking and splitting in Rust, on SIMD byte search (`memchr`) and lookup tables. One crate, published to crates.io as `chunk`.

License is MIT OR Apache-2.0.

Read [CONTRIBUTING.md](CONTRIBUTING.md) and [AI_POLICY.md](AI_POLICY.md). Both apply to agents.

## Scope

chunk does two things: size-based chunking at delimiter boundaries (`chunk`, `Chunker`, `OwnedChunker`) and splitting at every delimiter or pattern (`split`, `split_at_delimiters`, `split_at_patterns`, `PatternSplitter`). Nothing else. Do not add tokenizers, token-aware merging, embeddings, similarity math, or language bindings. Chonkie no longer depends on chunk; don't add code here for it.

Ask before adding a dependency. Today there are two: `memchr` and `daggrs`.

## Layout

| Path | Role |
| --- | --- |
| `src/chunk.rs` | `chunk`, `Chunker`, `OwnedChunker`: chunks of a target size, cut at the last delimiter in the window. |
| `src/split.rs` | `split`, `split_at_delimiters`, `split_at_patterns`, `PatternSplitter`, `IncludeDelim`. |
| `src/delim.rs` | The search kernels: `memchr`/`memchr2`/`memchr3`, the 256-entry table, `memmem`, `MultiPatternSearcher`. |
| `benches/` | Criterion benchmarks against text-splitter and kiru. See `benches/README.md` for the data. |

## Correctness

Every chunking and splitting function must hold these:

1. Chunks and segments cover the input with no gaps and no overlaps, so joining them gives back the input byte for byte (except under `IncludeDelim::None`, which drops delimiters by design).
2. Offsets slice the input: `&text[start..end]` is the segment.
3. Same input and options, same output.
4. No empty chunks or segments.

Test empty input, input with no delimiters, input that is all delimiters, and multi-byte patterns.

## Commands

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

CI runs the same. Benchmarks: `cargo bench` (needs `benches/data/enwik8`).

## Commit messages

Every commit subject uses one of these two prefixes:

```
feat(scope): short imperative summary
bug(scope): short imperative summary
```

`feat` is new behavior, a rewrite, docs, or a refactor. `bug` fixes incorrect behavior. Use no other prefix. That includes `docs`, `chore`, `fix`, and `refactor`.

`scope` is `chunk`, `split`, `delim`, `bench`, `ci`, or `docs`. The summary is lowercase and imperative, with no trailing period. Add a body when the subject does not say why. PR titles use the same format.

```
feat(split): add a reusable pattern splitter

bug(chunk): keep the last byte when the window ends on a delimiter
```

Commit only when the user asks. Never open pull requests or issues, and never post comments on GitHub: a human does that. Never force-push `main`. Put the session's prompts in the commit body or PR description under `AI prompts:` when AI wrote a significant part, as the AI policy requires.
