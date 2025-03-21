# Install Instructions

## Prequisites

- ### Rust
  - The best way to install Rust is with `rustup`
    - On [macOS or Linux](https://www.rust-lang.org/tools/install), run the following command in your terminal:
        ```bash
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        ```
    - On [Windows](https://forge.rust-lang.org/infra/other-installation-methods.html#other-ways-to-install-rustup):
      - Please download the [rustup-init.exe](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)

- ### OpenAI API Key (Optional, but highly recommended)
    - During the current beta, `notera` depends on the user to use their own api key (as of right now), however it is easy to set up.
    
1. See this link to set up: [Open AI api key creation](https://platform.openai.com/api-keys)
2. Add the following code to you `.zshrc` or `.bashrc` file. 
```shell
export OPENAI_API_KEY=your_api_key_here
```
3. Run the following command in your terminal:
- ZSH:
    ```shell
    source ~/.zshrc
    ```
- BASH:
    ```shell
    source ~/.bashrc
    ```
4. Now, you should be able to use the AI features of `notera`

## Installing the `notera` AI alpha

1. ### Using the `cargo` package manager (recommended, comes with rustup):
    The fastest and easiest way to install `notera` is with `cargo`, using the `install <BINARY>` subcommand. 
    `cargo install` uses [crates.io](https://crates.io/), the Rust community's 'crate' registry, to install packages.
    
    To install `notera` with crates.io, run the following command in your terminal. 
    ```bash
    cargo install notera@1.0.0-alpha.0
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
    cargo build --release
    sudo mv target/release/notera /usr/local/bin
    ```
    



