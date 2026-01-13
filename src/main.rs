#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

use core::fmt::Write;
use uefi::prelude::*;
use uefi::{proto::console::text::{Color, Output}, system, Error};

mod player;

const MOVIE_TEXT: &str = include_str!("../movies/sw1.txt");
const MAX_LINE_WIDTH: usize = 256; // Maximum buffer size for line width

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let result = system::with_stdout(|stdout| -> uefi::Result {
        // Use the first text mode (commonly 80x25) and ensure a clean start.
        if let Some(mode) = stdout.modes().next() {
            stdout.set_mode(mode)?;
        }

        // Query current mode to get console dimensions
        let mode_info = stdout.current_mode()?.unwrap();
        let console_cols = mode_info.columns();
        let _console_rows = mode_info.rows();
        
        // Use console width for line width, capped at MAX_LINE_WIDTH
        let line_width = console_cols.min(MAX_LINE_WIDTH);
        
        stdout.set_color(Color::White, Color::Black)?;
        stdout.clear()?;

        // Allocate line buffer once and reuse it for all lines
        let mut line_buf = [0u8; MAX_LINE_WIDTH];

        for frame in player::parse_movie(MOVIE_TEXT) {
            stdout.set_cursor_position(0, 0)?;
            stdout.set_color(Color::White, Color::Black)?;

            for &line in frame.lines.iter() {
                let normalized = normalize_line(line, &mut line_buf[..line_width]);
                write_line(stdout, normalized)?;
            }

            boot::stall(frame.duration.as_micros() as usize);
        }

        Ok(())
    });

    match result {
        Ok(()) => Status::SUCCESS,
        Err(err) => err.status(),
    }
}

fn write_line(stdout: &mut Output, line: &str) -> uefi::Result {
    stdout
        .write_str(line)
        .map_err(|_| Error::from(Status::DEVICE_ERROR))?;
    stdout
        .write_str("\r\n")
        .map_err(|_| Error::from(Status::DEVICE_ERROR))?;
    Ok(())
}

fn normalize_line<'a>(line: &str, buf: &'a mut [u8]) -> &'a str {
    let line_width = buf.len();
    buf.fill(b' ');

    let mut filled = 0;
    for ch in line.chars() {
        let mut tmp = [0u8; 4];
        let encoded = ch.encode_utf8(&mut tmp);
        let encoded_bytes = encoded.as_bytes();

        if filled + encoded_bytes.len() > line_width {
            break;
        }

        buf[filled..filled + encoded_bytes.len()].copy_from_slice(encoded_bytes);
        filled += encoded_bytes.len();

        if filled == line_width {
            break;
        }
    }

    core::str::from_utf8(buf).unwrap()
}