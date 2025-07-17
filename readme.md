# Python Package Manager (ppm)

[![Rust](https://img.shields.io/badge/Rust-1.85.0%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust-based command-line tool for managing Python packages in your active environment.

## Features

- Python Environment Aware
- Package Management
- Requirements File Support
- **Parallel Installation** - Install packages concurrently for faster performance
- Dependency Tracking
- Version Control
- Listing Support
- Progress Indicators
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
# Install packages (sequential)
ppm install <package1> [package2...]

# Install packages (parallel)
ppm install -p <package1> [package2...]
ppm install --parallel <package1> [package2...]

# Install from requirements file
ppm install -r=requirements.txt

# Install from requirements file (parallel)
ppm install -p -r=requirements.txt

# Update package
ppm update <package-name> <version>

# Remove package
ppm delete <package-name>

# List packages
ppm list

## Examples

Install specific version:
ppm install requests==2.28.1

Install multiple (sequential):
ppm install numpy pandas matplotlib

Install multiple (parallel):
ppm install -p numpy pandas matplotlib

From requirements file:
ppm install -r=requirements.txt

From requirements file (parallel):
ppm install -p -r=requirements.txt

Update package:
ppm update numpy 1.24.0

Remove package:
ppm delete requests

## Performance Benefits

The parallel installation feature provides significant performance improvements:

- **Concurrent Downloads**: Multiple packages download simultaneously
- **Progress Tracking**: Visual progress bar with real-time updates
- **Individual Error Handling**: Failed packages don't block successful installations
- **Resource Optimization**: Better utilization of network and system resources

### Performance Comparison
```
Sequential Installation:
  Package 1 → Package 2 → Package 3 → Package 4
  Total time: ~30 seconds for 4 packages

Parallel Installation:
  Package 1 ↓
  Package 2 ↓  (concurrent)
  Package 3 ↓
  Package 4 ↓
  Total time: ~8 seconds for 4 packages
```

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