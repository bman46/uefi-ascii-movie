#![no_main]
#![no_std]

use core::fmt::Write;
use uefi::prelude::*;
use uefi::{proto::console::text::{Color, Output}, system, Error};

mod player;

const MOVIE_TEXT: &str = include_str!("../movies/sw1.txt");

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
            stdout.clear()?;

            for &line in frame.lines.iter() {
                write_line(stdout, line)?;
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