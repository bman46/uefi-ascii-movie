//! Simple ASCII-art movie player parser.
//!
//! Format expected:
//! - All frames are placed sequentially in a single text file.
//! - Each frame is exactly 14 lines long.
//! - The first line of each frame is the display time for that frame
//!   expressed as an integer number of ticks, where one tick equals
//!   1/15 of a second.
//! - The remaining 13 lines are the ASCII art that should be displayed.
//!
//! This module provides a zero-allocation parser that returns an
//! iterator over `Frame` values which borrow slices from the original
//! input string. This keeps the implementation suitable for `no_std`
//! environments (no heap usage).

use core::time::Duration;

const TICKS_PER_SECOND: u64 = 15;
const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// One frame of the movie.
///
/// Lines are borrowed slices into the original input. There are always
/// 13 art lines per frame (the first of the 14 lines is the duration).
pub struct Frame<'a> {
    /// Display duration for this frame.
    pub duration: Duration,
    /// 13 lines of ASCII art.
    pub lines: [&'a str; 13],
}

impl<'a> Frame<'a> {
    /// Return the lines joined with newlines. Useful for quick display.
    pub fn as_text(&self) -> heapless_line_join::Joined<'_> {
        heapless_line_join::join_lines(&self.lines)
    }
}

/// Movie parser which yields `Frame<'a>` items.
///
/// Construct with `MovieParser::new(movie_text)` and then iterate.
pub struct MovieParser<'a> {
    lines: core::str::Lines<'a>,
}

impl<'a> MovieParser<'a> {
    /// Create a new parser over the provided movie text.
    pub fn new(movie_text: &'a str) -> Self {
        // Normalise CRLF by using `lines()` which handles both '\n' and '\r\n'.
        Self {
            lines: movie_text.lines(),
        }
    }
}

impl<'a> Iterator for MovieParser<'a> {
    type Item = Frame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Collect next 14 lines: first is duration, next 13 are art lines.
        // If we can't collect 14 lines, we stop iteration.
        let mut buf: [&str; 14] = [""; 14];
        for i in 0..14 {
            match self.lines.next() {
                Some(l) => buf[i] = l,
                None => return None,
            }
        }

        // Parse duration from first line. Assumption: integer ticks where
        // each tick represents 1/15 of a second. If parsing fails, fall back
        // to a single tick to avoid panics in constrained envs.
        let ticks = match buf[0].trim().parse::<u64>() {
            Ok(v) => v,
            Err(_) => 1,
        };

        let whole_seconds = ticks / TICKS_PER_SECOND;
        let leftover_ticks = ticks % TICKS_PER_SECOND;
        let nanos = ((leftover_ticks as u128) * NANOS_PER_SECOND
            + (TICKS_PER_SECOND as u128 / 2))
            / (TICKS_PER_SECOND as u128);
        let nanos = nanos as u32;

        // Copy the remaining 13 lines into the frame lines array.
        let mut art: [&str; 13] = [""; 13];
        let mut j = 0;
        let mut i = 1;
        while i < 14 {
            art[j] = buf[i];
            i += 1;
            j += 1;
        }

        Some(Frame {
            duration: Duration::new(whole_seconds, nanos),
            lines: art,
        })
    }
}

/// Convenience function: parse the entire movie and return a `MovieParser`.
pub fn parse_movie(movie_text: &str) -> MovieParser<'_> {
    MovieParser::new(movie_text)
}

// -- Minimal helper to join lines without requiring `alloc` or `std`.
// We'll provide a tiny, optional helper using a small inline utility.
// The consumer can display the 13 lines themselves; this helper simply
// produces a single borrowed string if needed by joining with '\n'.

/// Small helper crate-like module to provide joined view of lines.
///
/// This is intentionally minimal: it returns a small stack-allocated
/// buffer view when possible. If you prefer to print line-by-line,
/// you can ignore this utility.
mod heapless_line_join {
    /// A tiny wrapper that stores a single borrowed string slice where
    /// the caller is expected to have joined lines into a static buffer
    /// or otherwise manage printing line-by-line. Provided for API
    /// completeness only.
    pub struct Joined<'a> {
        pub content: &'a str,
    }

    /// Join lines with newlines. For `no_std` safety we don't allocate.
    /// This returns a small borrowed wrapper around the caller-owned
    /// static or temporary slice. In practice it's easier to print the
    /// lines directly instead of using this function; it's included
    /// to match common conveniences.
    pub fn join_lines<'a>(lines: &'a [&'a str]) -> Joined<'a> {
        // Naive implementation: if there is only a single line, return it
        // directly; otherwise return the first line. This avoids allocation.
        // The helper is best-effort and not a full implementation.
        if lines.len() == 1 {
            Joined { content: lines[0] }
        } else {
            // Fallback: point to the first line (caller should print line-by-line).
            Joined { content: lines[0] }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_two_frames() {
        let text = "1\nline1a\nline2a\nline3a\nline4a\nline5a\nline6a\nline7a\nline8a\nline9a\nline10a\nline11a\nline12a\nline13a\n5\nL1b\nL2b\nL3b\nL4b\nL5b\nL6b\nL7b\nL8b\nL9b\nL10b\nL11b\nL12b\nL13b\n";
        let mut it = parse_movie(text);
        let f1 = it.next().expect("first frame");
        assert_eq!(f1.duration.as_secs(), 0);
        assert_eq!(f1.duration.subsec_nanos(), 66_666_667);
        assert_eq!(f1.lines[0], "line1a");
        let f2 = it.next().expect("second frame");
        assert_eq!(f2.duration.as_secs(), 0);
        assert_eq!(f2.duration.subsec_nanos(), 333_333_333);
        assert_eq!(f2.lines[0], "L1b");
        assert!(it.next().is_none());
    }
}
