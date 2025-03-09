# Python Package Manager (ppm)

[![Rust](https://img.shields.io/badge/Rust-1.85.0%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust-based command-line tool for managing Python packages in your active environment.

## Features

- Python Environment Aware
- Package Management
- Requirements File Support
- Dependency Tracking
- Version Control
- Listing Support
- Rust-Powered

## Installation

### Prerequisites
- Rust 1.85.0+
- Cargo
- Python 3.6+

### Steps
1. Clone the repository:
git clone https://github.com/yourusername/python-package-manager.git
cd python-package-manager

2. Build:
cargo build --release

3. (Optional) Add to PATH:
sudo cp target/release/python-package-manager /usr/local/bin/ppm

## Usage

### Basic Commands
# Install packages
ppm install <package1> [package2...]
ppm install -r=requirements.txt

# Update package
ppm update <package-name> <version>

# Remove package
ppm delete <package-name>

# List packages
ppm list

## Examples

Install specific version:
ppm install requests==2.28.1

Install multiple:
ppm install numpy pandas matplotlib

From requirements:
ppm install -r=requirements.txt

Update package:
ppm update numpy 1.24.0

Remove package:
ppm delete requests

## Development

Build:
cargo build

Test:
cargo test

## License
MIT - See LICENSE

## Support
- Linux
- macOS
- Windows (WSL recommended)