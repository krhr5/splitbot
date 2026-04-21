use crate::app::{App, DestRow, MultiField, Screen};
use crate::events::AppEvent;
use crate::splitnow::data::{ASSETS, NETWORKS, asset_label, network_label};
use crate::ui::components::{
    accent_color, centered_rect, chrome, field_label_style, focus_border, input_block,
};
use crate::ui::status::{progress_line, wallets_line};
use crate::wallet;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use splitnow::WalletDistribution;
use tui_input::backend::crossterm::EventHandler;

pub fn render(frame: &mut Frame, app: &App) {
    let hints = "`Tab/Shift+Tab` field  ·  `←→` row field  ·  `↑↓` row  ·  `a` add row  ·  `g` gen new wallet  ·  `Del` remove  ·  `Enter` review  ·  `Esc` back";
    let subtitle = format!("{} destinations", app.multi.rows.len());
    let body = chrome(frame, app, "Multi-Swap", &subtitle, hints);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(6),
            Constraint::Length(6),
        ])
        .split(body);

    let header = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(outer[0]);

    input_block(
        frame,
        header[0],
        "amount",
        app.multi.amount.value(),
        app.multi.focus == MultiField::Amount,
        false,
    );
    single_picker(
        frame,
        header[1],
        "from asset",
        asset_label(ASSETS[app.multi.from_asset_idx]),
        app.multi.from_asset_idx,
        ASSETS.len(),
        app.multi.focus == MultiField::FromAsset,
    );
    single_picker(
        frame,
        header[2],
        "from network",
        network_label(NETWORKS[app.multi.from_network_idx]),
        app.multi.from_network_idx,
        NETWORKS.len(),
        app.multi.focus == MultiField::FromNetwork,
    );

    render_rows(frame, outer[1], app);
    render_footer(frame, outer[2], app);

    if app.multi.submit_confirm {
        render_submit_confirm(frame, app);
    }
}

fn single_picker(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    idx: usize,
    total: usize,
    focused: bool,
) {
    let text = Line::from(vec![
        Span::styled(
            if focused { "◂ " } else { "  " },
            Style::default().fg(accent_color()),
        ),
        Span::styled(
            value.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ({}/{})", idx + 1, total.max(1)),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(focused))
        .title(Line::from(Span::styled(
            format!(" {label} "),
            field_label_style(focused),
        )));
    frame.render_widget(Paragraph::new(text).block(block), area);
}

fn render_rows(frame: &mut Frame, area: Rect, app: &App) {
    let total_bips: u32 = app
        .multi
        .rows
        .iter()
        .filter_map(|r| parse_percent_to_bips(r.pct.value()).ok())
        .sum();
    let title = format!(
        " destinations  (Σ = {}%) ",
        format_bips_as_percent(total_bips)
    );

    let lines: Vec<Line> = app
        .multi
        .rows
        .iter()
        .enumerate()
        .map(|(i, r)| format_row(app, i, r))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(true))
        .title(title);
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(block),
        area,
    );
}

fn format_row(app: &App, i: usize, r: &DestRow) -> Line<'static> {
    let sel = i == app.multi.selected_row;
    let marker = if sel { "›" } else { " " };
    let asset = asset_label(ASSETS[r.asset_idx]);
    let network = network_label(NETWORKS[r.network_idx]);
    let exch = app
        .exchangers
        .get(r.exchanger_idx)
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "-".into());
    let addr = r.address.value();
    let addr_short = if addr.len() > 20 {
        format!("{}…{}", &addr[..10], &addr[addr.len() - 6..])
    } else if addr.is_empty() {
        "<unset>".into()
    } else {
        addr.to_string()
    };

    let field_marker = |f: MultiField| -> &str {
        if sel && app.multi.focus == f {
            "■"
        } else {
            " "
        }
    };

    let text = format!(
        "{marker} #{idx:02}  {ab}{addr:<22} {pb}{pct:>7}  {xb}{asset:<6}/{net:<12}  {eb}{ex}",
        marker = marker,
        idx = i,
        ab = field_marker(MultiField::RowAddress),
        addr = addr_short,
        pb = field_marker(MultiField::RowPercent),
        pct = format!("{}%", r.pct.value()),
        xb = field_marker(MultiField::RowAsset),
        asset = asset,
        net = network,
        eb = field_marker(MultiField::RowExchanger),
        ex = exch,
    );
    let style = if sel {
        Style::default()
            .fg(accent_color())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Line::from(Span::styled(text, style))
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    if app.multi.submitting {
        lines.push(Line::from(Span::styled(
            "submitting…",
            Style::default().fg(Color::Yellow),
        )));
    }
    if let Some(o) = &app.multi.order {
        lines.push(Line::from(Span::raw(format!("order: {}", o.order_id))));
        lines.push(Line::from(Span::raw(format!(
            "deposit: {} {} to {}",
            o.deposit_amount,
            asset_label(ASSETS[app.multi.from_asset_idx]),
            o.deposit_address
        ))));
        if let Some(status_order) = app
            .status
            .latest
            .as_ref()
            .filter(|status_order| status_order.short_id == o.order_id)
        {
            lines.push(progress_line(status_order, area.width.saturating_sub(2)));
            lines.push(wallets_line(status_order));
        }
    }
    if let Some(err) = &app.multi.error {
        lines.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(false))
        .title(" status ");
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(block),
        area,
    );
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.multi.submit_confirm {
        match key.code {
            KeyCode::Esc => app.multi.submit_confirm = false,
            KeyCode::Char('y') | KeyCode::Char('Y') => submit(app),
            _ => {}
        }
        return;
    }
    if key.code == KeyCode::Esc {
        app.screen = Screen::Home;
        return;
    }
    if key.code == KeyCode::Enter {
        app.multi.submit_confirm = true;
        return;
    }
    if key.code == KeyCode::Tab {
        app.multi.focus = next_field(app.multi.focus, !app.multi.rows.is_empty());
        return;
    }
    if key.code == KeyCode::BackTab {
        app.multi.focus = prev_field(app.multi.focus, !app.multi.rows.is_empty());
        return;
    }

    // row-level navigation regardless of focus
    if matches!(
        app.multi.focus,
        MultiField::RowAddress
            | MultiField::RowPercent
            | MultiField::RowAsset
            | MultiField::RowNetwork
            | MultiField::RowExchanger
    ) {
        match key.code {
            KeyCode::Up => {
                if app.multi.selected_row > 0 {
                    app.multi.selected_row -= 1;
                }
                return;
            }
            KeyCode::Down => {
                if app.multi.selected_row + 1 < app.multi.rows.len() {
                    app.multi.selected_row += 1;
                }
                return;
            }
            KeyCode::Char('a') if !is_text_field(app.multi.focus) => {
                app.multi.rows.push(DestRow::default());
                app.multi.selected_row = app.multi.rows.len() - 1;
                return;
            }
            KeyCode::Delete | KeyCode::Backspace if !is_text_field(app.multi.focus) => {
                if app.multi.rows.len() > 1 {
                    let idx = app.multi.selected_row;
                    app.multi.rows.remove(idx);
                    if app.multi.selected_row >= app.multi.rows.len() {
                        app.multi.selected_row = app.multi.rows.len() - 1;
                    }
                }
                return;
            }
            KeyCode::Char('g') if !is_text_field(app.multi.focus) => {
                gen_clean_wallet(app);
                return;
            }
            _ => {}
        }
    }

    dispatch(app, key);
}

fn is_text_field(f: MultiField) -> bool {
    matches!(
        f,
        MultiField::RowAddress | MultiField::RowPercent | MultiField::Amount
    )
}

fn dispatch(app: &mut App, key: KeyEvent) {
    match app.multi.focus {
        MultiField::Amount => forward(&mut app.multi.amount, key),
        MultiField::FromAsset => pick(&mut app.multi.from_asset_idx, ASSETS.len(), key),
        MultiField::FromNetwork => pick(&mut app.multi.from_network_idx, NETWORKS.len(), key),
        MultiField::RowAddress => {
            if let Some(r) = app.multi.rows.get_mut(app.multi.selected_row) {
                forward(&mut r.address, key);
            }
        }
        MultiField::RowPercent => {
            if let Some(r) = app.multi.rows.get_mut(app.multi.selected_row) {
                forward(&mut r.pct, key);
            }
        }
        MultiField::RowAsset => {
            if let Some(r) = app.multi.rows.get_mut(app.multi.selected_row) {
                pick(&mut r.asset_idx, ASSETS.len(), key);
            }
        }
        MultiField::RowNetwork => {
            if let Some(r) = app.multi.rows.get_mut(app.multi.selected_row) {
                pick(&mut r.network_idx, NETWORKS.len(), key);
            }
        }
        MultiField::RowExchanger => {
            let total = app.exchangers.len().max(1);
            if let Some(r) = app.multi.rows.get_mut(app.multi.selected_row) {
                pick(&mut r.exchanger_idx, total, key);
            }
        }
    }
}

fn forward(input: &mut tui_input::Input, key: KeyEvent) {
    let evt = crossterm::event::Event::Key(key);
    input.handle_event(&evt);
}

fn pick(idx: &mut usize, total: usize, key: KeyEvent) {
    if total == 0 {
        return;
    }
    match key.code {
        KeyCode::Left => {
            if *idx == 0 {
                *idx = total - 1;
            } else {
                *idx -= 1;
            }
        }
        KeyCode::Right => {
            *idx = (*idx + 1) % total;
        }
        _ => {}
    }
}

fn next_field(f: MultiField, has_rows: bool) -> MultiField {
    match f {
        MultiField::Amount => MultiField::FromAsset,
        MultiField::FromAsset => MultiField::FromNetwork,
        MultiField::FromNetwork => {
            if has_rows {
                MultiField::RowAddress
            } else {
                MultiField::Amount
            }
        }
        MultiField::RowAddress => MultiField::RowPercent,
        MultiField::RowPercent => MultiField::RowAsset,
        MultiField::RowAsset => MultiField::RowNetwork,
        MultiField::RowNetwork => MultiField::RowExchanger,
        MultiField::RowExchanger => MultiField::Amount,
    }
}

fn prev_field(f: MultiField, has_rows: bool) -> MultiField {
    match f {
        MultiField::Amount => {
            if has_rows {
                MultiField::RowExchanger
            } else {
                MultiField::FromNetwork
            }
        }
        MultiField::FromAsset => MultiField::Amount,
        MultiField::FromNetwork => MultiField::FromAsset,
        MultiField::RowAddress => MultiField::FromNetwork,
        MultiField::RowPercent => MultiField::RowAddress,
        MultiField::RowAsset => MultiField::RowPercent,
        MultiField::RowNetwork => MultiField::RowAsset,
        MultiField::RowExchanger => MultiField::RowNetwork,
    }
}

fn gen_clean_wallet(app: &mut App) {
    let idx = app.multi.selected_row;
    let Some(row) = app.multi.rows.get(idx) else {
        return;
    };
    let network = NETWORKS[row.network_idx];
    let Some(chain_family) = wallet::chain_family_for_network(network) else {
        app.toast_err(format!(
            "wallet generation is not available for {}",
            network_label(network)
        ));
        return;
    };
    let label = format!("clean-{}", chrono_like_ts());
    let w = match wallet::generate(chain_family, label) {
        Ok(wallet) => wallet,
        Err(e) => {
            app.toast_err(format!("wallet generate: {e}"));
            return;
        }
    };
    let address = w.address.clone();
    if let Some(v) = app.vault.as_mut()
        && let Err(e) = v.add_wallet(w)
    {
        app.toast_err(format!("vault save: {e}"));
        return;
    }
    if let Some(r) = app.multi.rows.get_mut(idx) {
        r.address = tui_input::Input::from(address);
    }
    app.toast_ok("clean wallet generated and stored");
}

fn chrono_like_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn submit(app: &mut App) {
    app.multi.submit_confirm = false;
    let amount: f64 = match app.multi.amount.value().trim().parse() {
        Ok(v) if v > 0.0 => v,
        _ => {
            app.multi.error = Some("amount must be a positive number".into());
            return;
        }
    };
    if app.multi.rows.is_empty() {
        app.multi.error = Some("at least one destination row required".into());
        return;
    }
    let mut sum: u32 = 0;
    let mut dists: Vec<WalletDistribution> = Vec::with_capacity(app.multi.rows.len());
    for (i, r) in app.multi.rows.iter().enumerate() {
        let bips: u32 = match parse_percent_to_bips(r.pct.value()) {
            Ok(v) => v,
            Err(e) => {
                app.multi.error = Some(format!("row {i}: {e}"));
                return;
            }
        };
        sum = sum.saturating_add(bips);
        let addr = r.address.value().trim().to_string();
        if addr.is_empty() {
            app.multi.error = Some(format!("row {i}: address required"));
            return;
        }
        let Some(exchanger_id) = app.exchanger_at(r.exchanger_idx) else {
            app.multi.error = Some("no exchanger loaded".into());
            return;
        };
        dists.push(WalletDistribution {
            to_address: addr,
            to_pct_bips: bips,
            to_asset_id: ASSETS[r.asset_idx],
            to_network_id: NETWORKS[r.network_idx],
            to_exchanger_id: exchanger_id,
        });
    }
    if sum != 10000 {
        app.multi.error = Some(format!(
            "percent total must equal 100.00% (got {}%)",
            format_bips_as_percent(sum)
        ));
        return;
    }
    let Some(client) = app.client.clone() else {
        app.multi.error = Some("SplitNOW client not initialized".into());
        return;
    };
    let from = app.multi_from_pair();
    app.multi.error = None;
    app.multi.submitting = true;
    let tx = app.tx.clone();
    tokio::spawn(async move {
        let quote_res = client
            .quote(amount, from, (dists[0].to_asset_id, dists[0].to_network_id))
            .await;
        match quote_res {
            Ok(q) => {
                let quote_id = q.quote_id.clone();
                let _ = tx.send(AppEvent::QuoteReady(Ok(q)));
                let order_res = client
                    .order(quote_id, amount, from, dists)
                    .await
                    .map_err(|e| e.to_string());
                let _ = tx.send(AppEvent::OrderReady(order_res));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::QuoteReady(Err(e.to_string())));
            }
        }
    });
}

fn parse_percent_to_bips(value: &str) -> Result<u32, &'static str> {
    let value = value.trim();
    if value.is_empty() {
        return Err("percent required");
    }
    if value.starts_with('-') {
        return Err("percent cannot be negative");
    }

    let (whole, frac) = match value.split_once('.') {
        Some((whole, frac)) => (whole, Some(frac)),
        None => (value, None),
    };

    if whole.is_empty() || !whole.chars().all(|ch| ch.is_ascii_digit()) {
        return Err("percent must be a number");
    }

    let whole: u32 = whole.parse().map_err(|_| "percent must be a number")?;
    let frac_bips = match frac {
        Some(frac) => {
            if frac.is_empty() || frac.len() > 2 || !frac.chars().all(|ch| ch.is_ascii_digit()) {
                return Err("percent can have at most 2 decimals");
            }
            match frac.len() {
                1 => {
                    frac.parse::<u32>()
                        .map_err(|_| "percent must be a number")?
                        * 10
                }
                2 => frac
                    .parse::<u32>()
                    .map_err(|_| "percent must be a number")?,
                _ => 0,
            }
        }
        None => 0,
    };

    whole
        .checked_mul(100)
        .and_then(|base| base.checked_add(frac_bips))
        .ok_or("percent is too large")
}

fn format_bips_as_percent(bips: u32) -> String {
    format!("{}.{:02}", bips / 100, bips % 100)
}

fn render_submit_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 34, frame.area());
    let lines = vec![
        Line::from(Span::styled(
            "Submit this multi-swap order?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "from: {} {} on {}",
            app.multi.amount.value().trim(),
            asset_label(ASSETS[app.multi.from_asset_idx]),
            network_label(NETWORKS[app.multi.from_network_idx]),
        )),
        Line::from(format!("destinations: {}", app.multi.rows.len())),
        Line::from("review the rows before confirming"),
        Line::from(""),
        Line::from(Span::styled(
            "`y` submit  ·  `Esc` cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let popup = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" confirm submit "),
    );
    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}
