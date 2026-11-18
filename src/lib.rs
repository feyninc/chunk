//! The fastest semantic text chunking library — up to 1TB/s chunking throughput.
//!
//! This crate provides two main functionalities:
//!
//! 1. **Size-based chunking** ([`chunk`] module): Split text into chunks of a target size,
//!    preferring to break at delimiter boundaries.
//!
//! 2. **Delimiter splitting** ([`split`] module): Split text at every delimiter occurrence,
//!    keeping every byte.
//!
//! # Examples
//!
//! ## Size-based chunking
//!
//! ```
//! use chunk::chunk;
//!
//! let text = b"Hello world. How are you? I'm fine.\nThanks for asking.";
//!
//! // With defaults (4KB chunks, split at \n . ?)
//! let chunks: Vec<&[u8]> = chunk(text).collect();
//!
//! // With custom size and delimiters
//! let chunks: Vec<&[u8]> = chunk(text).size(1024).delimiters(b"\n.?!").collect();
//!
//! // With multi-byte pattern (e.g., metaspace for SentencePiece tokenizers)
//! let metaspace = "▁".as_bytes(); // [0xE2, 0x96, 0x81]
//! let chunks: Vec<&[u8]> = chunk(b"Hello\xE2\x96\x81World").pattern(metaspace).collect();
//! ```
//!
//! ## Delimiter splitting
//!
//! ```
//! use chunk::{split, split_at_delimiters, IncludeDelim};
//!
//! let text = b"Hello. World. Test.";
//!
//! // Using the builder API
//! let slices = split(text).delimiters(b".").include_prev().collect_slices();
//! assert_eq!(slices, vec![b"Hello.".as_slice(), b" World.".as_slice(), b" Test.".as_slice()]);
//!
//! // Using the function directly
//! let offsets = split_at_delimiters(text, b".", IncludeDelim::Prev, 0);
//! assert_eq!(&text[offsets[0].0..offsets[0].1], b"Hello.");
//! ```

mod chunk;
mod delim;
mod split;

// Re-export from chunk module
pub use crate::chunk::{Chunker, OwnedChunker, chunk};

// Re-export from split module
pub use crate::split::{
    IncludeDelim, PatternSplitter, Splitter, split, split_at_delimiters, split_at_patterns,
};

// Re-export constants and types from delim module
pub use crate::delim::{DEFAULT_DELIMITERS, DEFAULT_TARGET_SIZE, MultiPatternSearcher};

// Additional tests that span modules
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_chunk_and_split_consistency() {
        // Both should preserve all bytes
        let text = b"Hello. World. Test.";

        let chunk_total: usize = chunk(text).size(10).delimiters(b".").map(|c| c.len()).sum();
        let split_total: usize = split_at_delimiters(text, b".", IncludeDelim::Prev, 0)
            .iter()
            .map(|(s, e)| e - s)
            .sum();

        assert_eq!(chunk_total, text.len());
        assert_eq!(split_total, text.len());
    }

    #[test]
    fn test_consecutive_delimiters_chunk() {
        let text = b"Hello\n\nWorld";
        let chunks: Vec<_> = chunk(text).size(8).delimiters(b"\n").collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_prefix_mode_chunk() {
        let text = b"Hello World Test";
        let chunks: Vec<_> = chunk(text).size(8).delimiters(b" ").prefix().collect();
        assert_eq!(chunks[0], b"Hello");
        assert_eq!(chunks[1], b" World");
        assert_eq!(chunks[2], b" Test");
    }

    #[test]
    fn test_prefix_preserves_total_bytes() {
        let text = b"Hello World Test More Words Here";
        let chunks: Vec<_> = chunk(text).size(10).delimiters(b" ").prefix().collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_prefix_mode_delimiter_at_window_start() {
        let text = b"Hello world";
        let chunks: Vec<_> = chunk(text).size(5).delimiters(b" ").prefix().collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        assert_eq!(chunks[0], b"Hello");
    }

    #[test]
    fn test_prefix_mode_small_chunks() {
        let text = b"a b c d e";
        let chunks: Vec<_> = chunk(text).size(2).delimiters(b" ").prefix().collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        for c in &chunks {
            assert!(!c.is_empty(), "Found empty chunk!");
        }
    }

    // ============ Multi-byte pattern tests ============

    #[test]
    fn test_pattern_metaspace_suffix() {
        let metaspace = "▁".as_bytes();
        let text = "Hello▁World▁Test".as_bytes();
        let chunks: Vec<_> = chunk(text).size(15).pattern(metaspace).collect();
        assert_eq!(chunks[0], "Hello▁".as_bytes());
        assert_eq!(chunks[1], "World▁Test".as_bytes());
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_pattern_metaspace_prefix() {
        let metaspace = "▁".as_bytes();
        let text = "Hello▁World▁Test".as_bytes();
        let chunks: Vec<_> = chunk(text).size(15).pattern(metaspace).prefix().collect();
        assert_eq!(chunks[0], "Hello".as_bytes());
        assert_eq!(chunks[1], "▁World▁Test".as_bytes());
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_pattern_preserves_bytes() {
        let metaspace = "▁".as_bytes();
        let text = "The▁quick▁brown▁fox▁jumps▁over▁the▁lazy▁dog".as_bytes();
        let chunks: Vec<_> = chunk(text).size(20).pattern(metaspace).collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_pattern_no_match_hard_split() {
        let pattern = b"XYZ";
        let text = b"abcdefghijklmnop";
        let chunks: Vec<_> = chunk(text).size(5).pattern(pattern).collect();
        assert_eq!(chunks[0], b"abcde");
        assert_eq!(chunks[1], b"fghij");
    }

    #[test]
    fn test_pattern_single_byte_optimization() {
        let text = b"Hello World Test";
        let chunks: Vec<_> = chunk(text).size(8).pattern(b" ").prefix().collect();
        assert_eq!(chunks[0], b"Hello");
        assert_eq!(chunks[1], b" World");
    }

    // ============ Consecutive and Forward Fallback Tests ============

    #[test]
    fn test_consecutive_pattern_basic() {
        let metaspace = b"\xE2\x96\x81";
        let text = b"word\xE2\x96\x81\xE2\x96\x81\xE2\x96\x81next";
        let chunks: Vec<_> = chunk(text)
            .pattern(metaspace)
            .size(10)
            .prefix()
            .consecutive()
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        assert_eq!(chunks[0], b"word");
        assert!(chunks[1].starts_with(metaspace));
    }

    #[test]
    fn test_forward_fallback_basic() {
        let metaspace = b"\xE2\x96\x81";
        let text = b"verylongword\xE2\x96\x81short";
        let chunks: Vec<_> = chunk(text)
            .pattern(metaspace)
            .size(6)
            .prefix()
            .forward_fallback()
            .collect();
        assert_eq!(chunks[0], b"verylongword");
        assert!(chunks[1].starts_with(metaspace));
    }

    #[test]
    fn test_delimiter_consecutive_basic() {
        let text = b"Hello\n\n\nWorld";
        let chunks: Vec<_> = chunk(text)
            .delimiters(b"\n")
            .size(8)
            .prefix()
            .consecutive()
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        assert_eq!(chunks[0], b"Hello");
        assert_eq!(chunks[1], b"\n\n\nWorld");
    }

    #[test]
    fn test_delimiter_forward_fallback_basic() {
        let text = b"verylongword next";
        let chunks: Vec<_> = chunk(text)
            .delimiters(b" ")
            .size(6)
            .prefix()
            .forward_fallback()
            .collect();
        assert_eq!(chunks[0], b"verylongword");
        assert_eq!(chunks[1], b" next");
    }

    #[test]
    fn test_owned_chunker_pattern() {
        let metaspace = "▁".as_bytes();
        let text = "Hello▁World▁Test".as_bytes().to_vec();
        let mut chunker = OwnedChunker::new(text.clone())
            .size(15)
            .pattern(metaspace.to_vec())
            .prefix();
        let mut chunks = Vec::new();
        while let Some(c) = chunker.next_chunk() {
            chunks.push(c);
        }
        assert_eq!(chunks[0], "Hello".as_bytes());
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_owned_chunker_collect_offsets() {
        let metaspace = "▁".as_bytes();
        let text = "Hello▁World▁Test".as_bytes().to_vec();
        let mut chunker = OwnedChunker::new(text.clone())
            .size(15)
            .pattern(metaspace.to_vec())
            .prefix();
        let offsets = chunker.collect_offsets();
        assert_eq!(offsets[0], (0, 5));
        assert_eq!(&text[offsets[0].0..offsets[0].1], "Hello".as_bytes());
    }

    // ============ Multi-pattern (.patterns()) tests ============

    #[test]
    fn test_patterns_cjk_basic() {
        // The exact use case from issue #2
        let text = "Hello。World，Test！Done".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(20)
            .delimiters(b"\n.?!")
            .patterns(&["。", "，", "！"])
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        // Should split at CJK punctuation, not mid-character
        for c in &chunks {
            assert!(
                std::str::from_utf8(c).is_ok(),
                "Chunk is not valid UTF-8: {:?}",
                c
            );
        }
    }

    #[test]
    fn test_patterns_preserves_all_bytes() {
        let text = "First sentence。Second part，Third section！Final".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(25)
            .delimiters(b".")
            .patterns(&["。", "，", "！"])
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_patterns_mixed_ascii_and_cjk() {
        // ASCII delimiter should also work alongside CJK patterns
        let text = "Hello. World。Test".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(12)
            .delimiters(b".")
            .patterns(&["。"])
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        // Both '.' and '。' should be valid split points
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_patterns_prefix_mode() {
        let text = "Hello。World。Test".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(15)
            .delimiters(b"")
            .patterns(&["。"])
            .prefix()
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        // First chunk should not start with 。
        assert!(chunks[0].starts_with(b"Hello"));
    }

    #[test]
    fn test_patterns_suffix_mode() {
        let text = "Hello。World。Test".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(15)
            .delimiters(b"")
            .patterns(&["。"])
            .collect(); // suffix is default
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        // First chunk should end with 。
        assert!(chunks[0].ends_with("。".as_bytes()));
    }

    #[test]
    fn test_patterns_with_forward_fallback() {
        let text = "verylongwordwithnodelimiters。short".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(10)
            .delimiters(b"")
            .patterns(&["。"])
            .prefix()
            .forward_fallback()
            .collect();
        // forward_fallback should find 。 past the window
        assert_eq!(chunks[0], "verylongwordwithnodelimiters".as_bytes());
        assert!(chunks[1].starts_with("。".as_bytes()));
    }

    #[test]
    fn test_patterns_no_match_hard_split() {
        let text = b"abcdefghijklmnop";
        let chunks: Vec<_> = chunk(text)
            .size(5)
            .delimiters(b"")
            .patterns(&["。"])
            .collect();
        // No matches — should hard split at target_size
        assert_eq!(chunks[0], b"abcde");
        assert_eq!(chunks[1], b"fghij");
    }

    #[test]
    fn test_patterns_utf8_boundary_safety() {
        // Regression test for issue #2: right single quote U+2019 = 0xE2 0x80 0x99
        // With forward_fallback, the chunker finds 。 past the window instead of hard-splitting
        let text = "It\u{2019}s a test。Done".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(15)
            .delimiters(b".")
            .patterns(&["。"])
            .forward_fallback()
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        for c in &chunks {
            assert!(
                std::str::from_utf8(c).is_ok(),
                "Chunk is not valid UTF-8: {:?}",
                c
            );
        }
    }

    #[test]
    fn test_patterns_many_triggers_aho_corasick() {
        // 5+ patterns should use Aho-Corasick internally
        let text = "A。B，C！D？E；F".as_bytes();
        let chunks: Vec<_> = chunk(text)
            .size(8)
            .patterns(&["。", "，", "！", "？", "；"])
            .collect();
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
        for c in &chunks {
            assert!(
                std::str::from_utf8(c).is_ok(),
                "Chunk is not valid UTF-8: {:?}",
                c
            );
        }
    }

    #[test]
    fn test_patterns_owned_chunker() {
        let text = "Hello。World，Test".as_bytes().to_vec();
        let mut chunker = OwnedChunker::new(text.clone())
            .size(15)
            .delimiters(b".".to_vec())
            .patterns(&["。", "，"]);
        let mut chunks = Vec::new();
        while let Some(c) = chunker.next_chunk() {
            chunks.push(c);
        }
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, text.len());
    }

    #[test]
    fn test_patterns_owned_chunker_collect_offsets() {
        let text = "Hello。World，Test".as_bytes().to_vec();
        let mut chunker = OwnedChunker::new(text.clone())
            .size(15)
            .delimiters(b".".to_vec())
            .patterns(&["。", "，"]);
        let offsets = chunker.collect_offsets();
        let total: usize = offsets.iter().map(|(s, e)| e - s).sum();
        assert_eq!(total, text.len());
        // Verify offsets are contiguous
        for i in 1..offsets.len() {
            assert_eq!(offsets[i - 1].1, offsets[i].0);
        }
    }
}
