#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

use core::fmt::Write;
use uefi::prelude::*;
use uefi::{proto::console::text::{Color, Output}, system, Error};

mod player;

const MOVIE_TEXT: &str = include_str!("../movies/sw1.txt");
const LINE_WIDTH: usize = 67;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let result = system::with_stdout(|stdout| -> uefi::Result {
        // Use the first text mode (commonly 80x25) and ensure a clean start.
        if let Some(mode) = stdout.modes().next() {
            stdout.set_mode(mode)?;
        }

        stdout.set_color(Color::White, Color::Black)?;
        stdout.clear()?;

        for frame in player::parse_movie(MOVIE_TEXT) {
            stdout.set_cursor_position(0, 0)?;
            stdout.set_color(Color::White, Color::Black)?;

            for &line in frame.lines.iter() {
                let mut line_buf = [0u8; LINE_WIDTH];
                let normalized = normalize_line(line, &mut line_buf);
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

fn normalize_line<'a>(line: &str, buf: &'a mut [u8; LINE_WIDTH]) -> &'a str {
    buf.fill(b' ');

    let mut filled = 0;
    for ch in line.chars() {
        let mut tmp = [0u8; 4];
        let encoded = ch.encode_utf8(&mut tmp);
        let encoded_bytes = encoded.as_bytes();

        if filled + encoded_bytes.len() > LINE_WIDTH {
            break;
        }

        buf[filled..filled + encoded_bytes.len()].copy_from_slice(encoded_bytes);
        filled += encoded_bytes.len();

        if filled == LINE_WIDTH {
            break;
        }
    }

    core::str::from_utf8(buf).unwrap()
}