# Install instructions

## Prequisites

- ### Rust
  - The best way to install Rust is with `rustup`
    - On [macOS or Linux](https://www.rust-lang.org/tools/install), run the following command in your terminal:
        ```bash
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        ```
    - On [Windows](https://forge.rust-lang.org/infra/other-installation-methods.html#other-ways-to-install-rustup):
      - Please download the [rustup-init.exe](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)

## Installing `notera`

1. ### Using the `cargo` package manager (recommended, comes with rustup):
    The fastest and easiest way to install `notera` is with `cargo`, using the `install <BINARY>` subcommand. 
    `cargo install` uses [crates.io](https://crates.io/), the Rust community's 'crate' registry, to install packages.
    
    To install `notera` with crates.io, run the following command in your terminal. 
    ```bash
    cargo install notera@0.1.0.alpha.0
    ```
   
    Installing with `cargo` will automatically add the app to your path, allowing you to just run `$ notera` to get started

2. ### Clone the GitHub repository:
    If you are interested in seeing the code, you can clone this repository and build the application with cargo yourself, and optionally add the `notera`'s executable to your path.

    To clone the repository and build the project, run the following commands in your terminal.
    ```bash
    git clone https://github.com/hijknight/notera.git
    cd notera
    ```

    Optional (but recommended): Add the `notera` executable to your path with the following command.
    ```bash
    sudo mv target/release/notera /usr/local/bin
    ```
    



