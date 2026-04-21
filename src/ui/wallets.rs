use crate::app::{App, Screen, WalletsMode};
use crate::config;
use crate::ui::components::{
    accent_fill_color, chrome, field_label_style, focus_border, input_block,
};
use crate::vault::{ChainFamily, StoredWallet, WalletSecret};
use crate::wallet::{self, bitcoin, evm, monero, solana};
use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tui_input::backend::crossterm::EventHandler;

pub fn render(frame: &mut Frame, app: &App) {
    let hints = match app.wallets.mode {
        WalletsMode::List => {
            "`Enter`/`v` view  ·  `g` generate wallet ·  `c` copy address  ·  `m` add to multi-swap  ·  `r` rename  ·  `x` export all  ·  `i` import  ·  `Del` delete  ·  `Esc` back"
        }
        WalletsMode::Inspect => {
            "`y` copy address  ·  `m` add to multi-swap  ·  `p` copy secret  ·  `e` export  ·  `r` reveal  ·  `Esc` back"
        }
        WalletsMode::NewLabel => "`←/→`: wallet type  ·  `Enter`: create  ·  `Esc`: cancel",
        WalletsMode::RenameLabel => "`Enter`: save  ·  `Esc`: cancel",
        WalletsMode::ImportLabel | WalletsMode::ImportSecret => {
            "`←/→`: wallet type  ·  `Enter`: next  ·  `Esc`: cancel"
        }
    };
    let subtitle = app
        .vault
        .as_ref()
        .map(|v| format!("{} wallets", v.wallets().len()))
        .unwrap_or_default();
    let body = chrome(frame, app, "Wallets", &subtitle, hints);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(6), Constraint::Length(6)])
        .split(body);

    let wallets = app
        .vault
        .as_ref()
        .map(|v| v.wallets().to_vec())
        .unwrap_or_default();
    render_list(frame, chunks[0], &wallets, app.wallets.selected);

    match app.wallets.mode {
        WalletsMode::List => {
            let help = Paragraph::new(Line::from(Span::styled(
                "generate with g, choose type with arrows while entering label, copy address with c, import with i",
                Style::default().fg(Color::DarkGray),
            )))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(focus_border(false))
                    .title(Line::from(Span::styled(
                        " hint ",
                        field_label_style(false),
                    ))),
            );
            frame.render_widget(help, chunks[1]);
        }
        WalletsMode::Inspect => render_inspect(frame, chunks[1], app, &wallets),
        WalletsMode::NewLabel => input_block(
            frame,
            chunks[1],
            &format!(
                "label for new {} wallet",
                selected_chain_family(app).label()
            ),
            app.wallets.label_input.value(),
            true,
            false,
        ),
        WalletsMode::RenameLabel => input_block(
            frame,
            chunks[1],
            "new wallet label",
            app.wallets.label_input.value(),
            true,
            false,
        ),
        WalletsMode::ImportLabel => input_block(
            frame,
            chunks[1],
            &format!(
                "label for imported {} wallet",
                selected_chain_family(app).label()
            ),
            app.wallets.label_input.value(),
            true,
            false,
        ),
        WalletsMode::ImportSecret => input_block(
            frame,
            chunks[1],
            import_secret_label(selected_chain_family(app)),
            app.wallets.secret_input.value(),
            true,
            true,
        ),
    }

    if let Some(err) = &app.wallets.error {
        let err_rect = ratatui::layout::Rect {
            x: body.x,
            y: body.y + body.height.saturating_sub(2),
            width: body.width,
            height: 1,
        };
        let p = Paragraph::new(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
        frame.render_widget(p, err_rect);
    }
}

fn render_list(frame: &mut Frame, area: Rect, wallets: &[StoredWallet], selected: usize) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(true))
        .title(" Wallets ");
    if wallets.is_empty() {
        let p = Paragraph::new(Line::from(Span::styled(
            "no wallets yet — press g to generate one",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        frame.render_widget(p, area);
        return;
    }
    let lines: Vec<Line> = wallets
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let sel = i == selected;
            let style = if sel {
                Style::default()
                    .fg(Color::Black)
                    .bg(accent_fill_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let marker = if sel { "› " } else { "  " };
            Line::from(Span::styled(
                format!(
                    "{marker}{:<18} {:<8} {}",
                    w.label,
                    w.chain_family.label(),
                    w.address
                ),
                style,
            ))
        })
        .collect();
    let p = Paragraph::new(lines).block(block);
    frame.render_widget(p, area);
}

fn render_inspect(frame: &mut Frame, area: Rect, app: &App, wallets: &[StoredWallet]) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(false))
        .title(Line::from(Span::styled(
            " wallet details ",
            field_label_style(false),
        )));

    let Some(wallet) = wallets.get(app.wallets.selected) else {
        frame.render_widget(Paragraph::new("no wallet selected").block(block), area);
        return;
    };

    let secret = wallet_secret(wallet).unwrap_or_else(|e| format!("secret unavailable: {e}"));
    let secret_display = if app.wallets.reveal_secret {
        secret
    } else {
        format!(
            "{}…{}",
            "*".repeat(12),
            &wallet.address[wallet.address.len().saturating_sub(6)..]
        )
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("label     ", Style::default().fg(Color::DarkGray)),
            Span::raw(wallet.label.clone()),
        ]),
        Line::from(vec![
            Span::styled("type      ", Style::default().fg(Color::DarkGray)),
            Span::raw(wallet.chain_family.label()),
        ]),
        Line::from(vec![
            Span::styled("address   ", Style::default().fg(Color::DarkGray)),
            Span::raw(wallet.address.clone()),
        ]),
        Line::from(vec![
            Span::styled("secret    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                secret_display,
                if app.wallets.reveal_secret {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            export_hint(wallet.chain_family),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.wallets.mode {
        WalletsMode::List => handle_list(app, key),
        WalletsMode::Inspect => handle_inspect(app, key),
        WalletsMode::NewLabel => handle_new_label(app, key),
        WalletsMode::RenameLabel => handle_rename_label(app, key),
        WalletsMode::ImportLabel => handle_import_label(app, key),
        WalletsMode::ImportSecret => handle_import_secret(app, key),
    }
}

fn handle_list(app: &mut App, key: KeyEvent) {
    let len = app.vault.as_ref().map(|v| v.wallets().len()).unwrap_or(0);
    match key.code {
        KeyCode::Esc => app.screen = Screen::Home,
        KeyCode::Enter | KeyCode::Char('v') if len > 0 => {
            app.wallets.mode = WalletsMode::Inspect;
            app.wallets.reveal_secret = false;
            app.wallets.error = None;
        }
        KeyCode::Char('g') => {
            app.wallets.mode = WalletsMode::NewLabel;
            app.wallets.label_input.reset();
            app.wallets.error = None;
        }
        KeyCode::Char('r') => {
            if let Some(wallet) = app
                .vault
                .as_ref()
                .and_then(|v| v.wallets().get(app.wallets.selected))
            {
                app.wallets.mode = WalletsMode::RenameLabel;
                app.wallets.label_input = tui_input::Input::from(wallet.label.clone());
                app.wallets.error = None;
            }
        }
        KeyCode::Char('x') => {
            let wallets = app
                .vault
                .as_ref()
                .map(|v| v.wallets().to_vec())
                .unwrap_or_default();
            match export_all_wallets_markdown(&wallets) {
                Ok(path) => {
                    app.wallets.error = None;
                    app.toast_ok(format!("exported all wallets to {}", path.display()));
                }
                Err(e) => app.wallets.error = Some(e.to_string()),
            }
        }
        KeyCode::Char('c') => {
            if let Some(wallet) = app
                .vault
                .as_ref()
                .and_then(|v| v.wallets().get(app.wallets.selected))
            {
                match copy_to_clipboard(&wallet.address) {
                    Ok(()) => {
                        app.wallets.error = None;
                        app.toast_ok("address copied");
                    }
                    Err(e) => app.wallets.error = Some(e.to_string()),
                }
            }
        }
        KeyCode::Char('m') => {
            if let Some(wallet) = app
                .vault
                .as_ref()
                .and_then(|v| v.wallets().get(app.wallets.selected))
                .cloned()
            {
                add_wallet_to_multiswap(app, &wallet);
            }
        }
        KeyCode::Char('i') => {
            app.wallets.mode = WalletsMode::ImportLabel;
            app.wallets.label_input.reset();
            app.wallets.secret_input.reset();
            app.wallets.error = None;
        }
        KeyCode::Delete if len > 0 => {
            let idx = app.wallets.selected.min(len - 1);
            if let Some(v) = app.vault.as_mut() {
                if let Err(e) = v.remove_wallet(idx) {
                    app.toast_err(format!("delete failed: {e}"));
                } else {
                    app.toast_ok("wallet deleted");
                }
            }
            let new_len = app.vault.as_ref().map(|v| v.wallets().len()).unwrap_or(0);
            if app.wallets.selected >= new_len && new_len > 0 {
                app.wallets.selected = new_len - 1;
            } else if new_len == 0 {
                app.wallets.selected = 0;
            }
        }
        KeyCode::Up if app.wallets.selected > 0 => {
            app.wallets.selected -= 1;
        }
        KeyCode::Down if app.wallets.selected + 1 < len => {
            app.wallets.selected += 1;
        }
        _ => {}
    }
}

fn handle_inspect(app: &mut App, key: KeyEvent) {
    let wallet = app
        .vault
        .as_ref()
        .and_then(|v| v.wallets().get(app.wallets.selected))
        .cloned();
    let Some(wallet) = wallet else {
        app.wallets.mode = WalletsMode::List;
        return;
    };
    let len = app.vault.as_ref().map(|v| v.wallets().len()).unwrap_or(0);

    match key.code {
        KeyCode::Esc => {
            app.wallets.mode = WalletsMode::List;
            app.wallets.reveal_secret = false;
            app.wallets.error = None;
        }
        KeyCode::Up if app.wallets.selected > 0 => {
            app.wallets.selected -= 1;
            app.wallets.reveal_secret = false;
        }
        KeyCode::Down if app.wallets.selected + 1 < len => {
            app.wallets.selected += 1;
            app.wallets.reveal_secret = false;
        }
        KeyCode::Char('r') => {
            app.wallets.reveal_secret = !app.wallets.reveal_secret;
            app.wallets.error = None;
        }
        KeyCode::Char('y') => match copy_to_clipboard(&wallet.address) {
            Ok(()) => {
                app.wallets.error = None;
                app.toast_ok("address copied");
            }
            Err(e) => app.wallets.error = Some(e.to_string()),
        },
        KeyCode::Char('p') => match wallet_secret(&wallet).and_then(|s| copy_to_clipboard(&s)) {
            Ok(()) => {
                app.wallets.error = None;
                app.toast_ok("secret copied");
            }
            Err(e) => app.wallets.error = Some(e.to_string()),
        },
        KeyCode::Char('e') => match export_wallet(&wallet) {
            Ok(path) => {
                app.wallets.error = None;
                app.toast_ok(format!("exported to {}", path.display()));
            }
            Err(e) => app.wallets.error = Some(e.to_string()),
        },
        KeyCode::Char('m') => add_wallet_to_multiswap(app, &wallet),
        _ => {}
    }
}

fn handle_new_label(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.wallets.mode = WalletsMode::List;
            app.wallets.reveal_secret = false;
            app.wallets.error = None;
        }
        KeyCode::Enter => {
            let label = app.wallets.label_input.value().trim().to_string();
            if label.is_empty() {
                app.wallets.error = Some("label required".into());
                return;
            }
            let chain_family = selected_chain_family(app);
            let w = match wallet::generate(chain_family, label) {
                Ok(w) => w,
                Err(e) => {
                    app.wallets.error = Some(e.to_string());
                    return;
                }
            };
            if let Some(v) = app.vault.as_mut()
                && let Err(e) = v.add_wallet(w)
            {
                app.wallets.error = Some(e.to_string());
                return;
            }
            let len = app.vault.as_ref().map(|v| v.wallets().len()).unwrap_or(0);
            if len > 0 {
                app.wallets.selected = len - 1;
            }
            app.wallets.mode = WalletsMode::List;
            app.wallets.reveal_secret = false;
            app.wallets.label_input.reset();
            app.wallets.error = None;
            app.toast_ok(format!("{} wallet generated", chain_family.label()));
        }
        KeyCode::Left | KeyCode::Right => pick_wallet_family(app, key),
        _ => {
            let evt = crossterm::event::Event::Key(key);
            app.wallets.label_input.handle_event(&evt);
        }
    }
}

fn handle_rename_label(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.wallets.mode = WalletsMode::List;
            app.wallets.error = None;
        }
        KeyCode::Enter => {
            let label = app.wallets.label_input.value().trim().to_string();
            if label.is_empty() {
                app.wallets.error = Some("label required".into());
                return;
            }
            let idx = app.wallets.selected;
            if let Some(v) = app.vault.as_mut()
                && let Err(e) = v.rename_wallet(idx, label)
            {
                app.wallets.error = Some(e.to_string());
                return;
            }
            app.wallets.mode = WalletsMode::List;
            app.wallets.label_input.reset();
            app.wallets.error = None;
            app.toast_ok("wallet renamed");
        }
        _ => {
            let evt = crossterm::event::Event::Key(key);
            app.wallets.label_input.handle_event(&evt);
        }
    }
}

fn handle_import_label(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.wallets.mode = WalletsMode::List;
            app.wallets.reveal_secret = false;
            app.wallets.error = None;
        }
        KeyCode::Enter => {
            if app.wallets.label_input.value().trim().is_empty() {
                app.wallets.error = Some("label required".into());
                return;
            }
            app.wallets.mode = WalletsMode::ImportSecret;
            app.wallets.error = None;
        }
        KeyCode::Left | KeyCode::Right => pick_wallet_family(app, key),
        _ => {
            let evt = crossterm::event::Event::Key(key);
            app.wallets.label_input.handle_event(&evt);
        }
    }
}

fn handle_import_secret(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.wallets.mode = WalletsMode::List;
            app.wallets.reveal_secret = false;
            app.wallets.error = None;
        }
        KeyCode::Enter => {
            let label = app.wallets.label_input.value().trim().to_string();
            let secret = app.wallets.secret_input.value().trim().to_string();
            let chain_family = selected_chain_family(app);
            match import_wallet(chain_family, label, &secret) {
                Ok(w) => {
                    if let Some(v) = app.vault.as_mut()
                        && let Err(e) = v.add_wallet(w)
                    {
                        app.wallets.error = Some(e.to_string());
                        return;
                    }
                    let len = app.vault.as_ref().map(|v| v.wallets().len()).unwrap_or(0);
                    if len > 0 {
                        app.wallets.selected = len - 1;
                    }
                    app.wallets.mode = WalletsMode::List;
                    app.wallets.reveal_secret = false;
                    app.wallets.label_input.reset();
                    app.wallets.secret_input.reset();
                    app.wallets.error = None;
                    app.toast_ok(format!("{} wallet imported", chain_family.label()));
                }
                Err(e) => {
                    app.wallets.error = Some(e.to_string());
                }
            }
        }
        KeyCode::Left | KeyCode::Right => pick_wallet_family(app, key),
        _ => {
            let evt = crossterm::event::Event::Key(key);
            app.wallets.secret_input.handle_event(&evt);
        }
    }
}

fn selected_chain_family(app: &App) -> ChainFamily {
    ChainFamily::ALL[app.wallets.chain_family_idx % ChainFamily::ALL.len()]
}

fn pick_wallet_family(app: &mut App, key: KeyEvent) {
    let total = ChainFamily::ALL.len();
    match key.code {
        KeyCode::Left => {
            if app.wallets.chain_family_idx == 0 {
                app.wallets.chain_family_idx = total - 1;
            } else {
                app.wallets.chain_family_idx -= 1;
            }
        }
        KeyCode::Right => {
            app.wallets.chain_family_idx = (app.wallets.chain_family_idx + 1) % total;
        }
        _ => {}
    }
    app.wallets.error = None;
}

fn import_wallet(chain_family: ChainFamily, label: String, secret: &str) -> Result<StoredWallet> {
    match chain_family {
        ChainFamily::Solana => solana::import_base58(label, secret),
        ChainFamily::Evm => evm::import_private_key_hex(label, secret),
        ChainFamily::Bitcoin => bitcoin::import_wif(label, secret),
        ChainFamily::Monero => {
            let (spend, view) = parse_monero_import(secret)?;
            monero::import_keys_hex(label, spend, view)
        }
    }
}

fn parse_monero_import(secret: &str) -> Result<(&str, &str)> {
    let parts: Vec<&str> = secret
        .split(|ch: char| ch == ':' || ch == ',' || ch.is_ascii_whitespace())
        .filter(|part| !part.trim().is_empty())
        .collect();
    if parts.len() != 2 {
        anyhow::bail!("expected Monero spend and view key as spend_hex:view_hex");
    }
    Ok((parts[0], parts[1]))
}

fn wallet_secret(wallet: &StoredWallet) -> Result<String> {
    match &wallet.secret {
        WalletSecret::Solana { .. } => solana::secret_base58(wallet),
        WalletSecret::Evm { .. } => evm::private_key_hex(wallet),
        WalletSecret::Bitcoin { .. } => bitcoin::wif(wallet),
        WalletSecret::Monero { .. } => {
            let (spend, view) = monero::private_keys_hex(wallet)?;
            Ok(format!("spend: {spend}\nview:  {view}"))
        }
    }
}

fn import_secret_label(chain_family: ChainFamily) -> &'static str {
    match chain_family {
        ChainFamily::Solana => "Solana base58 secret (64 bytes)",
        ChainFamily::Evm => "EVM private key hex",
        ChainFamily::Bitcoin => "Bitcoin WIF private key",
        ChainFamily::Monero => "Monero spend_hex:view_hex",
    }
}

fn export_hint(chain_family: ChainFamily) -> &'static str {
    match chain_family {
        ChainFamily::Solana => "secret is base58, compatible with Solana wallet imports",
        ChainFamily::Evm => "secret is an EVM private key hex string for MetaMask/Rabby imports",
        ChainFamily::Bitcoin => "secret is WIF for compatible Bitcoin wallet imports",
        ChainFamily::Monero => {
            "secret contains private spend and view keys for Monero wallet restore"
        }
    }
}

fn copy_to_clipboard(value: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .context("launch pbcopy")?;
    let Some(stdin) = child.stdin.as_mut() else {
        return Err(anyhow::anyhow!("pbcopy stdin unavailable"));
    };
    stdin
        .write_all(value.as_bytes())
        .context("write clipboard contents")?;
    let status = child.wait().context("wait for pbcopy")?;
    if !status.success() {
        return Err(anyhow::anyhow!("pbcopy failed"));
    }
    Ok(())
}

fn export_wallet(wallet: &StoredWallet) -> Result<std::path::PathBuf> {
    let dir = config::exports_dir()?;
    let filename = format!(
        "{}-{}.json",
        slugify(&wallet.label),
        &wallet.address[..wallet.address.len().min(8)]
    );
    let path = dir.join(filename);
    let payload = wallet_export_json(wallet)?;
    let body = serde_json::to_vec_pretty(&payload).context("serialize export json")?;
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn wallet_export_json(wallet: &StoredWallet) -> Result<serde_json::Value> {
    let mut payload = json!({
        "label": wallet.label,
        "chain_family": wallet.chain_family.label(),
        "address": wallet.address,
    });
    let obj = payload
        .as_object_mut()
        .context("wallet export payload must be an object")?;
    match &wallet.secret {
        WalletSecret::Solana { .. } => {
            obj.insert(
                "private_key_base58".into(),
                json!(solana::secret_base58(wallet)?),
            );
        }
        WalletSecret::Evm { .. } => {
            obj.insert(
                "private_key_hex".into(),
                json!(evm::private_key_hex(wallet)?),
            );
        }
        WalletSecret::Bitcoin { .. } => {
            obj.insert("private_key_wif".into(), json!(bitcoin::wif(wallet)?));
        }
        WalletSecret::Monero { .. } => {
            let (spend, view) = monero::private_keys_hex(wallet)?;
            obj.insert("private_spend_key_hex".into(), json!(spend));
            obj.insert("private_view_key_hex".into(), json!(view));
        }
    }
    Ok(payload)
}

fn add_wallet_to_multiswap(app: &mut App, wallet: &StoredWallet) {
    let target_idx = app
        .multi
        .rows
        .iter()
        .position(|row| row.address.value().trim().is_empty());

    let idx = if let Some(idx) = target_idx {
        idx
    } else {
        app.multi.rows.push(crate::app::DestRow::default());
        app.multi.rows.len() - 1
    };

    if let Some(row) = app.multi.rows.get_mut(idx) {
        row.address = tui_input::Input::from(wallet.address.clone());
        row.asset_idx = default_asset_idx(wallet.chain_family).unwrap_or(row.asset_idx);
        row.network_idx = crate::splitnow::data::NETWORKS
            .iter()
            .position(|network| {
                wallet::chain_family_for_network(*network) == Some(wallet.chain_family)
            })
            .unwrap_or(row.network_idx);
    }

    app.multi.selected_row = idx;
    app.wallets.error = None;
    app.toast_ok(format!("{} added to multi-swap", wallet.label));
}

fn default_asset_idx(chain_family: ChainFamily) -> Option<usize> {
    let asset = match chain_family {
        ChainFamily::Solana => splitnow::AssetId::Sol,
        ChainFamily::Evm => splitnow::AssetId::Eth,
        ChainFamily::Bitcoin => splitnow::AssetId::Btc,
        ChainFamily::Monero => splitnow::AssetId::Xmr,
    };
    crate::splitnow::data::ASSETS
        .iter()
        .position(|candidate| *candidate == asset)
}

fn export_all_wallets_markdown(wallets: &[StoredWallet]) -> Result<std::path::PathBuf> {
    if wallets.is_empty() {
        anyhow::bail!("no wallets to export");
    }

    let dir = config::exports_dir()?;
    let path = dir.join(format!("all-wallets-{}.md", export_timestamp()));

    let mut body = String::new();
    body.push_str("# SplitNOW TUI wallet export\n\n");
    body.push_str("Sensitive file. Treat every private key below as a hot secret.\n\n");

    for wallet in wallets {
        body.push_str(&wallet_markdown(wallet)?);
    }

    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

fn wallet_markdown(wallet: &StoredWallet) -> Result<String> {
    Ok(format!(
        "## {}\n\n- Type: `{}`\n- Address: `{}`\n\n```text\n{}\n```\n\n",
        wallet.label,
        wallet.chain_family.label(),
        wallet.address,
        wallet_secret(wallet)?
    ))
}

fn slugify(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-');
    if trimmed.is_empty() {
        "wallet".to_string()
    } else {
        trimmed.to_string()
    }
}

fn export_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::WalletSecret;

    #[test]
    fn wallet_markdown_omits_secret_label() {
        let wallet = StoredWallet {
            label: "evm-test".into(),
            chain_family: ChainFamily::Evm,
            address: "0xabc".into(),
            secret: WalletSecret::Evm {
                private_key_hex: "0x1234".into(),
            },
        };

        let markdown = wallet_markdown(&wallet).unwrap();
        assert!(!markdown.contains("Secret:"));
        assert!(markdown.contains("\n```text\n0x1234\n```"));
    }
}
