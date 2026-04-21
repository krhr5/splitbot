███████╗██████╗ ██╗     ██╗████████╗██████╗  ██████╗ ████████╗
██╔════╝██╔══██╗██║     ██║╚══██╔══╝██╔══██╗██╔═══██╗╚══██╔══╝
███████╗██████╔╝██║     ██║   ██║   ██████╔╝██║   ██║   ██║   
╚════██║██╔═══╝ ██║     ██║   ██║   ██╔══██╗██║   ██║   ██║   
███████║██║     ███████╗██║   ██║   ██████╔╝╚██████╔╝   ██║   
╚══════╝╚═╝     ╚══════╝╚═╝   ╚═╝   ╚═════╝  ╚═════╝    ╚═╝  

# 🤖 splitbot

`splitbot` is a terminal UI for quick multichain bridging, swaps and local hot-wallet management. We utilize the SplitNOW API, with access to their deep network of CEXs, OTC desks, and instant exchangers.
It can create or import wallets, prepare single-destination swaps, prepare
multi-destination split swaps across chains, and poll the latest order status. Currently supporting Solana, EVM, Bitcoin and Monero ecosystems.

## ✨ Features

- 🖥️ Terminal UI built with Ratatui.
- 🔐 Local encrypted vault for the SplitNOW API key and wallet secrets.
- 👛 Wallet generation/import for Solana, EVM, Bitcoin, and Monero.
- 🔁 Single swap flow for one destination address.
- 🧮 Multi-swap flow for splitting one deposit across multiple destination rows.
- 📡 Order status polling for the most recent order.
- 📦 Plaintext wallet export for manual backup/import workflows.

## ⚡ Quick Start

```sh
cargo build
cargo run
```

Run the app in a real terminal, not an automation shell. A TUI needs an
interactive terminal so it can enter raw mode and handle keyboard input
correctly.

On first launch, create a vault passphrase and enter a SplitNOW API key from
your SplitNOW account. Later launches ask only for the vault passphrase.

## 🧰 Toolchain Setup

`splitbot` creates wallets internally with Rust libraries, so the chain CLIs
below are not required just to generate addresses in the app. They are useful
for verifying addresses, inspecting balances, running nodes, or spending funds
outside `splitbot`.

### 🦀 Rust

Install Rust with `rustup`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

On macOS, install Apple's command-line tools first if you do not already have a
compiler toolchain:

```sh
xcode-select --install
```

### 🟣 Solana

Install the Solana CLI:

```sh
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version
```

### 🔷 EVM / Ethereum

Install Foundry for EVM tooling (`cast`, `forge`, `anvil`, `chisel`):

```sh
curl -L https://foundry.paradigm.xyz | bash
source "$HOME/.zshrc"
foundryup
cast --version
```

If you use Bash instead of Zsh, replace the `source` line with:

```sh
source "$HOME/.bashrc"
```

### 🟠 Bitcoin

`splitbot` can generate/import Bitcoin wallet keys internally. For a full
Bitcoin client, install Bitcoin Core from the official downloads page and verify
the release files before running it.

macOS/Homebrew convenience command:

```sh
brew install --cask bitcoin-core
```

Official downloads and verification instructions:

```text
https://bitcoincore.org/en/download
```

### ⚫ Monero

`splitbot` can generate/import Monero wallet keys internally. For Monero CLI
tools (`monero-wallet-cli`, `monerod`), install the official Monero CLI package
or use Homebrew on macOS/Linux.

Homebrew convenience command:

```sh
brew install monero
monero-wallet-cli --version
monerod --version
```

Official downloads and verification instructions:

```text
https://www.getmonero.org/downloads/
```

## 📁 Local Data

The app stores local data in the OS-native per-user data directory:

- macOS: `~/Library/Application Support/splitbot/`
- Linux: `~/.local/share/splitbot/`
- Windows: `%LOCALAPPDATA%\splitbot\`

Typical files include:

```text
vault.bin        # encrypted API key + wallet vault
last-order.json  # latest order status cache
splitbot.log     # local application log
exports/         # plaintext wallet exports, if you create them
```

## 🧪 Development

Run the full local check suite before opening a pull request:

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```

`Cargo.lock` is committed because `splitbot` is an application binary.

## 🔒 Security Notes

`splitbot` manages hot wallets. Any wallet generated or imported into the app
should be treated as online key material.

- 🔑 The vault is encrypted locally with a passphrase. There is no recovery path
  if the passphrase is lost.
- 📄 Wallet exports are plaintext files containing private keys. Store and
  delete them carefully.
- 📋 Copying secrets or addresses uses the system clipboard, which other local
  software may be able to read.
- 🌐 The app calls third-party services including SplitNOW, Solana RPC, and
  CoinGecko. Requests can reveal addresses, order identifiers, and usage
  metadata to those services.
- 🚫 Do not put real API keys, private keys, seed phrases, vault files, exports,
  or logs into GitHub issues, pull requests, commits, screenshots, or support
  messages.

See [SECURITY.md](SECURITY.md) for vulnerability reporting guidance.

## 📜 License

No license has been selected yet. Choose and add a license before announcing the
project as open source.


                                                       ▁▂▃▄▄▄▃▂▁                                    
                    ▁▄▅▆▆▄▂                        ▂▃▅▇█████████▆▄▁                                 
                    ▄█▅▄▇██▆▁                ▁▃▄▆▇███████▅▄▃▂▃▄▆██▇▁                                
                    ▁▃▁ ▁███▅            ▂▄▅▇████████▇▅▃▁       ▂▆█▆▄▁                              
                        ▁▇██▇         ▁▄▇███████▇▇▆▄▂▁            ▂▄▃▁     ▂▄▃▃▁                    
                        ▅███▅       ▁▅████████▆▅▃▁  ▂▄▆▇▇▇▇▆▅▂▁           ▄█▆▇██▆▂                  
                       ▄████▂      ▃███████▇▇▅▂   ▂▆██████████▆▁          ▃▆▂ ▁▇██▂                 
                      ▃████▇▁     ▃███████▇▇▃▁   ▂████▇▇▅▂▂▃▆██▃               ▃██▆                 
                     ▁▇████▅     ▂███████▆▆▅▁    ▇███▇▆▅   ▁▄██▄              ▁▇██▄                 
                     ▁█████▇     ▅█████▇█▆▇▅▁    █████▇▄▁  ▃██▅▁            ▁▁▆██▇▁                 
             ▁▄▆▃    ▁██████▃    ▆███████▆▇▆▃▁   ▆████▇▇▃▁    ▁▁▁▂▂   ▁▂▄▅▆▇█████▇▄▁                
             ▆█▄▁     ▆██████▃   ▅████████▆▇▆▅▂  ▂█████▅▆▆▂▁▄▆██████▆▆██████████████▆▃▁             
             ▆█▇▂     ▂███████▅▁ ▂████████▇▅█▇▅▃▁ ███████▆▅▆██████████████████▇█▆▇████▆▁            
             ▁▇██▇▅▄▃▂ ▃████████▅▂▅█████████▅▆▆▆▅▂▆█████████▅▆▆▇███████████████▅  ▁▃▇██▅            
              ▁▅▇██████▆▄▇███████████████████▇▇▇▇▇▅▇██████████▇▇████████▇█████▆▁    ▂███            
                 ▂▃▅▇▇███▇▅▇████████████████████▆▅█▇▆███████████████████▇███▆▃▁     ▁▇██            
                   ▁▃▇▆████▄▅██████████████████████████████████████████▇█▇▅▂      ▃▆███▃            
                     ▃▆▆▇███▅▆██████████▇▆████████████▇▇██████████████▇▇▇▂        ▂▄▄▃▁             
▆▆▆▆▆▅▄▄▃▁▁ ▁▁▁ ▁▂▂▁▂▃▆█▇████████████████▇████████████████████████████▇▅▁ ▁▁     ▁▂▂▂▂▂▃▃▃▃▂▃▃▄▅▅▅▆▅
██████████████████▅▃▃▆▇▇▆▇█████████████████████████████▅██████████████▆▅▂▁▁▅▇████▆█▇█████████▇▇█████
████████████▇███▇██▆▁ ▃▁▄▃▃▄▃▇█▅▃▄▄▇█████████████████▇▃▆█████▄▇█▆▆▇██▅▃▁  ▂▇███▆▅▇▇█████████████████
██████████████▇▅▇▇▂▄▁    ▁▁ ▁▅█▆▂▁▂▁▅▄▅█▅▆██▇███████▅▂ ▂▇▆▃▆▂▅▇▃▄▅▇█▅▁    ▄▆█▇▃▁▅▄▄▆████████████████
███████████████▄ ▃▁▂▁▄▁▃ ▅▅▂▁▁▂▁▄▁       ▄█▃▆████▆▇▇▂     ▁▄▃▄▁▂▁▄▅▂▁ ▁ ▁ ▁▃▁▂ ▁▁▂▁▅████████████████
██████████████▇▇██▆▆▇▆▂▄▆█▆▄▇▆▅▁ ▁▂▂▃▁▃▂▄▃▁  ▄▃▄▂ ▁▁▁▄▁▁▄▁▁▂▄▂▃▅▄  ▂▄▂▆▄▂▃▂▂▁▄▆▆▇█▆▆▇▇▇▇▇▇██████████
███████████████▆▆▆██▆▆▇▇▆▆▇▆▆▆▅▆▇█▇▆▄▅▇▆▇▇▇▆▆▃▃▄▅▅▅▆▇▆▅▅█▇▆▆▅▃▃▂▃▅▆██▇▆█▇▆▆▆█▇▇▇████▇▇██████████████
███████████████████████████████▇▇▇▇█▇█████████████████▇▇▇██▇██████████████▇▇▇██▇▇▇▇▇████████████████
███████████████▇▇▇▇▇████████████▇█████▇▇▇▇▆▇█▇████████████████▇▇▇▇▆▆▇▇▇▇███▇████████████████████████
