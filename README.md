Dumping league metaclasses


Build instructions:
** Install Ubuntu 22.04 under WSL or docker or w/e **
```
# Install llvm implementation of C++ standard library
sudo apt install libc++1

# Build
cargo build --release
```

Usage Instructions:
```
# Download (or copy) league executable
rmand-dl -p '.+/LeagueofLegends' manifest.manifest ./Game

# Dump
cargo run --release -- ./Game/LeagueofLegends.app/Contents/MacOS/LeagueofLegends
```
