# OpenRTX-Companion

GUI application to install and manage OpenRTX on ham radios.

## Building

To build you need glib-2.0 development headers.

```bash
cargo build
```

## Cross-Compiling Linux -> Windows

Install mingw-w64.

Open a shell and install dependencies:

```bash
pacman -S git base-devel gcc
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Type Y, 2, x86_64-pc-windows-gnu, enter, enter, enter, 1
echo 'export PATH="$PATH:/c/Users/nizzo/.cargo/bin"' >> ~/.bashrc
. ~/.bashrc
```

Clone and build this project:

```bash
git clone https://github.com/OpenRTX/OpenRTX-Companion; cd OpenRTX-Companion
cargo build
```
