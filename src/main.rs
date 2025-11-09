#![no_main]
#![no_std]

use core::fmt::Write;
use uefi::prelude::*;
use uefi::{proto::console::text::{Color, Output}, system, Error};

mod player;

const MOVIE_TEXT: &str = include_str!("../movies/sw1.txt");
const DEFAULT_COLUMNS: usize = 80;
const DEFAULT_ROWS: usize = 25;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    let result = system::with_stdout(|stdout| -> uefi::Result {
        // Use the first text mode (commonly 80x25) and ensure a clean start.
        let (columns, rows) = if let Some(mode) = stdout.modes().next() {
            let columns = mode.columns() as usize;
            let rows = mode.rows() as usize;
            stdout.set_mode(mode)?;
            (columns, rows)
        } else {
            (DEFAULT_COLUMNS, DEFAULT_ROWS)
        };

        stdout.set_color(Color::White, Color::Black)?;
        stdout.clear()?;

        for frame in player::parse_movie(MOVIE_TEXT) {
            stdout.set_cursor_position(0, 0)?;
            stdout.set_color(Color::White, Color::Black)?;

            for row in 0..rows {
                stdout.set_cursor_position(0, row as usize)?;
                if row < frame.lines.len() {
                    write_line(stdout, frame.lines[row], columns)?;
                } else {
                    write_blank_line(stdout, columns)?;
                }
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

fn write_line(stdout: &mut Output, line: &str, width: usize) -> uefi::Result {
    let display = if line.len() > width {
        &line[..width]
    } else {
        line
    };

    stdout
        .write_str(display)
        .map_err(|_| Error::from(Status::DEVICE_ERROR))?;

    pad_spaces(stdout, width.saturating_sub(display.len()))
}

fn write_blank_line(stdout: &mut Output, width: usize) -> uefi::Result {
    pad_spaces(stdout, width)
}

fn pad_spaces(stdout: &mut Output, count: usize) -> uefi::Result {
    for _ in 0..count {
        stdout
            .write_str(" ")
            .map_err(|_| Error::from(Status::DEVICE_ERROR))?;
    }
    Ok(())
}