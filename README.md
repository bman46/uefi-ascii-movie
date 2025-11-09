# UEFI ASCII Movie Bootloader
Stream an ASCII movie from a UEFI bootloader.

Inspired by [asciimation](https://asciimation.co.nz/), [ascii-movie](https://github.com/gabe565/ascii-movie), [uefi_nyan_80x25](https://github.com/diekmann/uefi_nyan_80x25/) and the iconic [towel.blinkenlights.nl](https://web.archive.org/web/20021205144143/http://www.blinkenlights.nl/thereg/).

## Build Process
Clone this repo, then run the following Cargo commands:

For x86 (Intel/AMD):
```bash
cargo build --target x86_64-unknown-uefi --release
```

For ARM:
```bash
cargo build --target aarch64-unknown-uefi --release
```

## Making a bootable
Use the following command to make a bootable flashdrive with ascii bootloader:
```bash
sudo efibootmgr --create --disk /dev/sdX --part 1 --label "ascii" --loader \\EFI\\ascii\\bootx64.efi 
```
Replace `/dev/sdX` with the device that corresponds to your flash drive.