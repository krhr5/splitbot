use crate::app::{App, Field, Screen};
use crate::events::AppEvent;
use crate::splitnow::data::{ASSETS, NETWORKS, asset_label, exchanger_label, network_label};
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
    let hints = "`Tab/Shift+Tab`: next field  ·  `←/→`: pick  ·  `g`: gen new wallet  ·  `Enter`: review  ·  `Esc`: back";
    let body = chrome(frame, app, "Single Swap", "", hints);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(21), Constraint::Min(3)])
        .split(body);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(outer[0]);

    input_block(
        frame,
        rows[0],
        "amount",
        app.single.amount.value(),
        app.single.focus == Field::Amount,
        false,
    );
    picker(
        frame,
        rows[1],
        "from asset",
        asset_label(ASSETS[app.single.from_asset_idx]),
        app.single.from_asset_idx,
        ASSETS.len(),
        app.single.focus == Field::FromAsset,
    );
    picker(
        frame,
        rows[2],
        "from network",
        network_label(NETWORKS[app.single.from_network_idx]),
        app.single.from_network_idx,
        NETWORKS.len(),
        app.single.focus == Field::FromNetwork,
    );
    picker(
        frame,
        rows[3],
        "to asset",
        asset_label(ASSETS[app.single.to_asset_idx]),
        app.single.to_asset_idx,
        ASSETS.len(),
        app.single.focus == Field::ToAsset,
    );
    picker(
        frame,
        rows[4],
        "to network",
        network_label(NETWORKS[app.single.to_network_idx]),
        app.single.to_network_idx,
        NETWORKS.len(),
        app.single.focus == Field::ToNetwork,
    );

    let exchanger_text = app
        .exchangers
        .get(app.single.exchanger_idx)
        .map(|e| format!("{} ({})", e.name, exchanger_label(e.id)))
        .unwrap_or_else(|| "(no exchangers loaded)".into());
    picker(
        frame,
        rows[5],
        "exchanger",
        &exchanger_text,
        app.single.exchanger_idx,
        app.exchangers.len().max(1),
        app.single.focus == Field::Exchanger,
    );

    input_block(
        frame,
        rows[6],
        "destination address",
        app.single.destination.value(),
        app.single.focus == Field::Destination,
        false,
    );

    render_summary(frame, outer[1], app);

    if app.single.submit_confirm {
        render_submit_confirm(frame, app);
    }
}

fn picker(
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
        Span::styled(
            if focused { "  ▸" } else { "" },
            Style::default().fg(accent_color()),
        ),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(focused))
        .title(Line::from(Span::styled(
            format!(" {label} "),
            field_label_style(focused),
        )));
    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, area);
}

fn render_summary(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines: Vec<Line> = Vec::new();
    if app.single.submitting {
        lines.push(Line::from(Span::styled(
            "submitting…",
            Style::default().fg(Color::Yellow),
        )));
    }
    if let Some(q) = &app.single.quote {
        lines.push(Line::from(Span::styled(
            format!("quote: {}  rates: {}", q.quote_id, q.rates.len()),
            Style::default().fg(Color::Green),
        )));
    }
    if let Some(o) = &app.single.order {
        lines.push(Line::from(Span::raw(format!("order: {}", o.order_id))));
        lines.push(Line::from(Span::raw(format!(
            "deposit: {} {} to {}",
            o.deposit_amount,
            asset_label(ASSETS[app.single.from_asset_idx]),
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
    if let Some(err) = &app.single.error {
        lines.push(Line::from(Span::styled(
            err.clone(),
            Style::default().fg(Color::Red),
        )));
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(focus_border(false))
        .title(" status ");
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.single.submit_confirm {
        match key.code {
            KeyCode::Esc => app.single.submit_confirm = false,
            KeyCode::Char('y') | KeyCode::Char('Y') => submit(app),
            _ => {}
        }
        return;
    }
    if key.code == KeyCode::Esc {
        app.screen = Screen::Home;
        return;
    }
    if key.code == KeyCode::Tab {
        app.single.focus = next_field(app.single.focus);
        return;
    }
    if key.code == KeyCode::BackTab {
        app.single.focus = prev_field(app.single.focus);
        return;
    }
    if key.code == KeyCode::Enter {
        app.single.submit_confirm = true;
        return;
    }
    if key.code == KeyCode::Char('g') || key.code == KeyCode::Char('G') {
        gen_destination_wallet(app);
        return;
    }
    match app.single.focus {
        Field::Amount => forward(&mut app.single.amount, key),
        Field::Destination => forward(&mut app.single.destination, key),
        Field::FromAsset => pick(&mut app.single.from_asset_idx, ASSETS.len(), key),
        Field::FromNetwork => pick(&mut app.single.from_network_idx, NETWORKS.len(), key),
        Field::ToAsset => pick(&mut app.single.to_asset_idx, ASSETS.len(), key),
        Field::ToNetwork => pick(&mut app.single.to_network_idx, NETWORKS.len(), key),
        Field::Exchanger => pick(
            &mut app.single.exchanger_idx,
            app.exchangers.len().max(1),
            key,
        ),
    }
}

fn gen_destination_wallet(app: &mut App) {
    if app.single.focus != Field::Destination {
        app.single.error = Some("focus destination address to generate a wallet".into());
        return;
    }
    let to_network = NETWORKS[app.single.to_network_idx];
    let Some(chain_family) = wallet::chain_family_for_network(to_network) else {
        app.single.error = Some(format!(
            "wallet generation is not available for {}",
            network_label(to_network)
        ));
        return;
    };

    let label = format!("clean-{}", chrono_like_ts());
    let wallet = match wallet::generate(chain_family, label) {
        Ok(wallet) => wallet,
        Err(e) => {
            app.single.error = Some(format!("wallet generate: {e}"));
            return;
        }
    };
    let address = wallet.address.clone();
    if let Some(vault) = app.vault.as_mut()
        && let Err(e) = vault.add_wallet(wallet)
    {
        app.single.error = Some(format!("vault save: {e}"));
        return;
    }
    let len = app.vault.as_ref().map(|v| v.wallets().len()).unwrap_or(0);
    if len > 0 {
        app.wallets.selected = len - 1;
    }
    app.single.destination = tui_input::Input::from(address);
    app.single.error = None;
    app.toast_ok("destination wallet generated and stored");
}

fn chrono_like_ts() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
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
        KeyCode::Left | KeyCode::Up => {
            if *idx == 0 {
                *idx = total - 1;
            } else {
                *idx -= 1;
            }
        }
        KeyCode::Right | KeyCode::Down => {
            *idx = (*idx + 1) % total;
        }
        _ => {}
    }
}

fn next_field(f: Field) -> Field {
    match f {
        Field::Amount => Field::FromAsset,
        Field::FromAsset => Field::FromNetwork,
        Field::FromNetwork => Field::ToAsset,
        Field::ToAsset => Field::ToNetwork,
        Field::ToNetwork => Field::Exchanger,
        Field::Exchanger => Field::Destination,
        Field::Destination => Field::Amount,
    }
}

fn prev_field(f: Field) -> Field {
    match f {
        Field::Amount => Field::Destination,
        Field::FromAsset => Field::Amount,
        Field::FromNetwork => Field::FromAsset,
        Field::ToAsset => Field::FromNetwork,
        Field::ToNetwork => Field::ToAsset,
        Field::Exchanger => Field::ToNetwork,
        Field::Destination => Field::Exchanger,
    }
}

fn submit(app: &mut App) {
    app.single.submit_confirm = false;
    let amount: f64 = match app.single.amount.value().trim().parse() {
        Ok(v) if v > 0.0 => v,
        _ => {
            app.single.error = Some("amount must be a positive number".into());
            return;
        }
    };
    let dest = app.single.destination.value().trim().to_string();
    if dest.is_empty() {
        app.single.error = Some("destination address required".into());
        return;
    }
    let Some(client) = app.client.clone() else {
        app.single.error = Some("SplitNOW client not initialized".into());
        return;
    };
    let Some(exchanger_id) = app.exchanger_at(app.single.exchanger_idx) else {
        app.single.error = Some("no exchanger available".into());
        return;
    };
    let from = app.single_from_pair();
    let to = app.single_to_pair();
    app.single.error = None;
    app.single.submitting = true;
    let tx = app.tx.clone();
    tokio::spawn(async move {
        let quote_res = client.quote(amount, from, to).await;
        match quote_res {
            Ok(q) => {
                let quote_id = q.quote_id.clone();
                let _ = tx.send(AppEvent::QuoteReady(Ok(q)));
                let dist = WalletDistribution {
                    to_address: dest,
                    to_pct_bips: 10000,
                    to_asset_id: to.0,
                    to_network_id: to.1,
                    to_exchanger_id: exchanger_id,
                };
                let order_res = client
                    .order(quote_id, amount, from, vec![dist])
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

fn render_submit_confirm(frame: &mut Frame, app: &App) {
    let area = centered_rect(64, 30, frame.area());
    let exchanger = app
        .exchangers
        .get(app.single.exchanger_idx)
        .map(|e| e.name.as_str())
        .unwrap_or("unavailable");
    let lines = vec![
        Line::from(Span::styled(
            "Submit this order?",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "from: {} {} on {}",
            app.single.amount.value().trim(),
            asset_label(ASSETS[app.single.from_asset_idx]),
            network_label(NETWORKS[app.single.from_network_idx]),
        )),
        Line::from(format!(
            "to: {} on {}",
            asset_label(ASSETS[app.single.to_asset_idx]),
            network_label(NETWORKS[app.single.to_network_idx]),
        )),
        Line::from(format!(
            "destination: {}",
            app.single.destination.value().trim()
        )),
        Line::from(format!("exchanger: {exchanger}")),
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
