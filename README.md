## Installation and Setup

### Setup procedure for Windows
 - Download Rust toolchain
```
curl https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -o rustup-init.exe
```

 - Install Rust
```
./rustup-init.exe -y
rustup install nightly
```

 - Clone and build vcpkg
```
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
```

 - Integrate vcpkg with the build environment
```
.\vcpkg integrate install
```

 - Install libusb
```
.\vcpkg install libusb:x64-windows-static-md
cd ..
```

### Setup procedure for Linux

 - Update package lists
```
sudo apt update
```

 - Install development tools & libraries
```
sudo apt install -y gcc curl libusb-1.0-0-dev pkg-config
```

 - Download Rust
```
curl -o rust.sh --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs
```

 - Install Rust toolchain
```
sh ./rust.sh -y
source "$HOME/.cargo/env"
rustup install nightly
```

## Build Procedure

### Can Dumptool for Windows

#### Steps
 - Complete the setup procedure for Windows given above
 - Open a terminal and run the below command
```
cargo build --release
```
 - The executable is created in the path `./target/release/can.exe`

### Build python libs for Linux

#### Dependencies

 - Python >= 3.9

#### Steps

 - Complete the setup procedure for Linux given above
 - Open a terminal and run the below commands

```
pip install -r driver/requirements.txt
pip install build
cd driver
rustup override set nightly
python -m build --wheel --outdir dist
```
 - The wheel file is created in the path `./dist/zscanfd-<cpython-version>.whl`

### Build python libs for Windows

#### Dependencies

 - Python >= 3.9

#### Steps
 - Complete the setup procedure for Windows given above
 - Open a terminal and run the below commands

```
pip install -r driver/requirements.txt
pip install build
cd driver
rustup override set nightly
python -m build --wheel --outdir dist
```
 - The wheel file is created in the path `./dist/zscanfd-<cpython-version>.whl`

### Build zscanfd library for Windows

#### Steps

 - Complete the setup procedure for Windows given above
 - Open a terminal and run the below commands

```
cd driver
cargo clean
cargo build
```
 - The library file is created in the path `./target/debug/zscanfd.dll`
 - The header file is available in the path `./src/c/zs_canfd.h`
