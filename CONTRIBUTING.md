# Contributing

chunk is the fast, boring part of chunking: find the delimiters, cut the bytes, get out of the way. Read the [AI policy](AI_POLICY.md) before you open anything.

## What belongs here

Two things: size-based chunking at delimiter boundaries (`chunk`, `Chunker`, `OwnedChunker`) and splitting at every delimiter or pattern (`split`, `split_at_delimiters`, `split_at_patterns`, `PatternSplitter`). Speed comes from SIMD byte search (`memchr`, `memchr2`, `memchr3`, `memmem`) and lookup tables, so changes to the hot paths come with benchmark numbers.

Don't add tokenizers, token-aware merging, embeddings, similarity math, or language bindings. They used to live here for Chonkie v1 and were removed in 0.11.

## Correctness

These are not optional:

1. Chunks and segments cover the input with no gaps and no overlaps, so joining them gives back the input byte for byte. The one exception is `IncludeDelim::None`, which drops delimiters by design.
2. Offsets slice the input: `&text[start..end]` is the segment.
3. Same input and options, same output.
4. No empty chunks or segments.

If a change breaks any of these, it doesn't land.

## Dev setup

You need Rust stable, via [rustup](https://rustup.rs), with `rustfmt` and `clippy`.

```bash
git clone https://github.com/feyninc/chunk
cd chunk
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Run all three before you push. CI runs the same.

Benchmarks use enwik8. See [benches/README.md](benches/README.md) for the download, then run `cargo bench`.

## Commit messages

Every commit subject uses one of two prefixes:

```
feat(scope): short imperative summary
bug(scope): short imperative summary
```

`feat` is new behavior, a rewrite, docs, or a refactor. `bug` fixes incorrect behavior. No other prefixes: not `fix`, `docs`, `chore`, or `refactor`.

`scope` is one of `chunk`, `split`, `delim`, `bench`, `ci`, `docs`. The summary is lowercase and imperative, with no trailing period. Add a body when the subject doesn't say why.

```
feat(split): add a reusable pattern splitter

bug(chunk): keep the last byte when the window ends on a delimiter
```

Pull request titles follow the same format.

## Pull requests

- One change per pull request. Keep it small enough to read in one sitting.
- A bug fix comes with a test that fails without it.
- A change to a search or chunking path comes with before and after `cargo bench` numbers.
- Open it yourself and explain what changed and why in your own words. AI must not open pull requests or write comments.
- If AI wrote a significant part of the change, include the prompts you used under an **AI prompts** heading. See the [AI policy](AI_POLICY.md).

## Issues

If something is broken, [open an issue](https://github.com/feyninc/chunk/issues). Include the input, the options, what you expected, and what chunk did.
