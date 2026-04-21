<pre>
███████╗██████╗ ██╗     ██╗████████╗██████╗  ██████╗ ████████╗
██╔════╝██╔══██╗██║     ██║╚══██╔══╝██╔══██╗██╔═══██╗╚══██╔══╝
███████╗██████╔╝██║     ██║   ██║   ██████╔╝██║   ██║   ██║   
╚════██║██╔═══╝ ██║     ██║   ██║   ██╔══██╗██║   ██║   ██║   
███████║██║     ███████╗██║   ██║   ██████╔╝╚██████╔╝   ██║   
╚══════╝╚═╝     ╚══════╝╚═╝   ╚═╝   ╚═════╝  ╚═════╝    ╚═╝  
</pre>

# 🤖 splitbot

`splitbot` is a terminal UI for quick multichain bridging, swaps and local hot-wallet management. It is powered by the [SplitNOW](https://splitnow.io/) API, with access to their deep network of CEXs, OTC desks, and instant exchangers.
It can create or import wallets, prepare single-destination swaps, prepare
multi-destination split swaps across chains, and poll the latest order status. Currently supporting Solana, EVM, Bitcoin and Monero ecosystems.

## ✨ Features

- 🖥️ Terminal UI built with Ratatui.
- 🔐 Local encrypted vault for the SplitNOW API key and wallet secrets.
- 👛 Wallet generation, import, labeling, export, and swap-destination reuse.
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

## 📦 Install from crates.io

`splitbot` is published on [crates.io](https://crates.io/crates/splitbot).
Install it with Cargo:

```sh
cargo install splitbot
```

Then run:

```sh
splitbot
```

## 🖥️ Terminal Support

`splitbot` runs in modern interactive terminals on macOS, Linux, and Windows.
The terminal must support raw-mode keyboard input and Unicode rendering for the
best TUI experience.

On macOS, Ghostty is the recommended terminal. It provides the modern terminal
behavior, rendering, and keyboard handling expected by the TUI.

Install Ghostty from the [official Ghostty site](https://ghostty.org/).

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

## 👛 Wallet Management

`splitbot` includes a local hot-wallet manager for addresses you want to use
with swaps and split payouts. Wallets are stored in the encrypted vault with
your SplitNOW API key and can be managed directly from the terminal UI.

Users can:

- Generate new Solana, EVM, Bitcoin, and Monero wallets without installing the
  chain CLIs.
- Import existing wallets using Solana base58 secrets, EVM private key hex,
  Bitcoin WIF, or Monero private spend/view keys.
- Label, rename, inspect, and delete saved wallets.
- Copy wallet addresses for deposits, withdrawals, or external verification.
- Reveal and copy private wallet secrets when needed for manual backup or
  migration.
- Export one wallet as JSON or export all wallets as a Markdown backup file.
- Add a saved wallet directly to a multi-swap destination row.
- Generate a fresh destination wallet from the single-swap or multi-swap flow
  and save it back into the vault.

The Account screen can also summarize native SOL balances across Solana wallets
stored in the vault.

## 🕹️ How To Use

Start the TUI with `cargo run`, unlock or create your vault, and confirm your
SplitNOW API key is configured. From Home, use `↑/↓` to choose Wallets, Single
Swap, Multi-Swap, Order Status, or Account, then press `Enter` to open the
screen. Press `Esc` from feature screens to return Home.

### Execute a Single Swap

1. Open **Single Swap** from Home.
2. Enter the amount, then choose the source asset/network, destination
   asset/network, and exchanger.
3. Enter a destination address, or focus the destination field and press `g` to
   generate a fresh wallet for the selected destination network.
4. Press `Enter` to review the order, then press `y` to submit.
5. Send the shown deposit amount to the shown deposit address and monitor the
   order status.

### Execute a Multi-Swap

1. Open **Multi-Swap** from Home.
2. Enter the source amount, source asset, and source network.
3. Configure each destination row with an address, percent, destination
   asset/network, and exchanger.
4. Use `a` to add rows, `Del` to remove rows, and make sure the destination
   percentages total `100.00%`.
5. Press `Enter` to review, then press `y` to submit.
6. Send the shown deposit amount to the shown deposit address and monitor the
   order status.

Check destination addresses, order details, and deposit addresses before sending
funds. Private keys copied or exported from the Wallets screen are hot secrets.

### Hotkeys

| Screen | Keys | Action |
| --- | --- | --- |
| Home | `↑/↓` | Move through menu items. |
| Home | `Enter` | Open the selected screen. |
| Home | `q` or `Esc` | Quit. |
| Feature screens | `Esc` | Return Home or cancel the current prompt. |
| Wallets | `g` | Generate a wallet. |
| Wallets | `i` | Import a wallet. |
| Wallets | `Enter` or `v` | View the selected wallet. |
| Wallets | `c` or `y` | Copy the selected wallet address. |
| Wallets | `r` | Rename in list view, or reveal/hide the secret in detail view. |
| Wallets | `m` | Add the selected wallet to a multi-swap destination row. |
| Wallets | `x` | Export all wallets. |
| Wallets | `Del` | Delete the selected wallet. |
| Wallet details | `p` | Copy the private secret. |
| Wallet details | `e` | Export the selected wallet. |
| Single Swap | `Tab` / `Shift+Tab` | Move between fields. |
| Single Swap | `←/→` | Change the focused picker value. |
| Single Swap | `g` | Generate a destination wallet when the destination field is focused. |
| Single Swap | `Enter` | Review the swap. |
| Single Swap | `y` | Submit from the review prompt. |
| Multi-Swap | `Tab` / `Shift+Tab` | Move between fields. |
| Multi-Swap | `←/→` | Change the focused picker value. |
| Multi-Swap | `↑/↓` | Move between destination rows while focused on row fields. |
| Multi-Swap | `a` | Add a destination row. |
| Multi-Swap | `g` | Generate a clean wallet for the selected row. |
| Multi-Swap | `Del` | Remove a destination row. |
| Multi-Swap | `Enter` | Review the swap. |
| Multi-Swap | `y` | Submit from the review prompt. |
| Order Status | `r` | Refresh the latest order status. |
| Order Status | `d` | Show or hide raw status details. |
| Account | `r` | Refresh balances. |
| Account | `e` | Edit the SplitNOW API key. |
| Account | `v` | Reveal or hide the API key. |
| Account | `Enter` | Save while editing the API key. |


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

See the official [Bitcoin Core downloads and verification
instructions](https://bitcoincore.org/en/download).

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

See the official [Monero downloads and verification
instructions](https://www.getmonero.org/downloads/).

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

See [LICENSE.txt](LICENSE.txt).


<pre>
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
</pre>
