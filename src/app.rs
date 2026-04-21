use crate::events::{AppEvent, Sender};
use crate::persist;
use crate::splitnow::SplitnowClient;
use crate::splitnow::client::AvailableExchanger;
use crate::vault::Vault;
use splitnow::{AssetId, ExchangerId, NetworkId, Order, OrderData, OrderStatus, QuoteData};
use std::collections::VecDeque;
use std::time::Instant;
use tui_input::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Unlock,
    Home,
    Account,
    Wallets,
    SingleSwap,
    MultiSwap,
    OrderStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlockStage {
    EnterPassphrase,
    NewPassphrase,
    ConfirmPassphrase,
    NewApiKey,
}

pub struct UnlockState {
    pub first_run: bool,
    pub stage: UnlockStage,
    pub passphrase: Input,
    pub confirm: Input,
    pub api_key: Input,
    pub error: Option<String>,
}

impl UnlockState {
    pub fn new(first_run: bool) -> Self {
        Self {
            first_run,
            stage: if first_run {
                UnlockStage::NewPassphrase
            } else {
                UnlockStage::EnterPassphrase
            },
            passphrase: Input::default(),
            confirm: Input::default(),
            api_key: Input::default(),
            error: None,
        }
    }
}

#[derive(Default)]
pub struct HomeState {
    pub selected: usize,
}

pub const HOME_ITEMS: &[&str] = &[
    "Wallets",
    "Single Swap",
    "Multi-Swap",
    "Order Status",
    "Account",
    "Quit",
];

#[derive(Default)]
pub struct AccountState {
    pub balance_loading: bool,
    pub total_sol: Option<f64>,
    pub total_usd: Option<f64>,
    pub balance_error: Option<String>,
    pub api_key_input: Input,
    pub editing_api_key: bool,
    pub reveal_api_key: bool,
    pub api_key_error: Option<String>,
}

#[derive(Default)]
pub struct WalletsState {
    pub selected: usize,
    pub mode: WalletsMode,
    pub chain_family_idx: usize,
    pub label_input: Input,
    pub secret_input: Input,
    pub reveal_secret: bool,
    pub error: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WalletsMode {
    #[default]
    List,
    Inspect,
    NewLabel,
    RenameLabel,
    ImportLabel,
    ImportSecret,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    #[default]
    Amount,
    FromAsset,
    FromNetwork,
    ToAsset,
    ToNetwork,
    Exchanger,
    Destination,
}

pub struct SwapState {
    pub amount: Input,
    pub from_asset_idx: usize,
    pub from_network_idx: usize,
    pub to_asset_idx: usize,
    pub to_network_idx: usize,
    pub exchanger_idx: usize,
    pub destination: Input,
    pub focus: Field,
    pub quote: Option<QuoteData>,
    pub order: Option<OrderData>,
    pub submitting: bool,
    pub submit_confirm: bool,
    pub error: Option<String>,
}

impl Default for SwapState {
    fn default() -> Self {
        Self {
            amount: Input::default(),
            from_asset_idx: 0,
            from_network_idx: 0,
            to_asset_idx: 1,
            to_network_idx: 1,
            exchanger_idx: 0,
            destination: Input::default(),
            focus: Field::Amount,
            quote: None,
            order: None,
            submitting: false,
            submit_confirm: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DestRow {
    pub address: Input,
    pub pct: Input,
    pub asset_idx: usize,
    pub network_idx: usize,
    pub exchanger_idx: usize,
}

impl Default for DestRow {
    fn default() -> Self {
        Self {
            address: Input::default(),
            pct: Input::from("100"),
            asset_idx: 0,
            network_idx: 0,
            exchanger_idx: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum MultiField {
    #[default]
    Amount,
    FromAsset,
    FromNetwork,
    RowAddress,
    RowPercent,
    RowAsset,
    RowNetwork,
    RowExchanger,
}

pub struct MultiSwapState {
    pub amount: Input,
    pub from_asset_idx: usize,
    pub from_network_idx: usize,
    pub rows: Vec<DestRow>,
    pub selected_row: usize,
    pub focus: MultiField,
    pub submitting: bool,
    pub submit_confirm: bool,
    pub order: Option<OrderData>,
    pub error: Option<String>,
}

impl Default for MultiSwapState {
    fn default() -> Self {
        Self {
            amount: Input::default(),
            from_asset_idx: 0,
            from_network_idx: 0,
            rows: vec![DestRow::default()],
            selected_row: 0,
            focus: MultiField::Amount,
            submitting: false,
            submit_confirm: false,
            order: None,
            error: None,
        }
    }
}

#[derive(Default)]
pub struct StatusState {
    pub order_id: Option<String>,
    pub latest: Option<Order>,
    pub last_poll: Option<Instant>,
    pub error: Option<String>,
    pub deposit_address: Option<String>,
    pub deposit_amount: Option<f64>,
    pub show_raw: bool,
}

pub struct Toast {
    pub message: String,
    pub until: Instant,
    pub level: ToastLevel,
}

struct PendingToast {
    message: String,
    level: ToastLevel,
    duration: std::time::Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Error,
}

pub struct App {
    pub quit: bool,
    pub screen: Screen,
    pub vault: Option<Vault>,
    pub client: Option<SplitnowClient>,
    pub exchangers: Vec<AvailableExchanger>,
    pub toast: Option<Toast>,
    toast_queue: VecDeque<PendingToast>,
    pub tx: Sender,

    pub unlock: UnlockState,
    pub home: HomeState,
    pub account: AccountState,
    pub wallets: WalletsState,
    pub single: SwapState,
    pub multi: MultiSwapState,
    pub status: StatusState,
}

impl App {
    pub fn new(first_run: bool, tx: Sender) -> Self {
        Self {
            quit: false,
            screen: Screen::Unlock,
            vault: None,
            client: None,
            exchangers: Vec::new(),
            toast: None,
            toast_queue: VecDeque::new(),
            tx,
            unlock: UnlockState::new(first_run),
            home: HomeState::default(),
            account: AccountState::default(),
            wallets: WalletsState::default(),
            single: SwapState::default(),
            multi: MultiSwapState::default(),
            status: StatusState::default(),
        }
    }

    pub fn toast_info(&mut self, msg: impl Into<String>) {
        self.set_toast(msg.into(), ToastLevel::Info);
    }

    pub fn toast_info_short(&mut self, msg: impl Into<String>) {
        self.set_toast_with_duration(
            msg.into(),
            ToastLevel::Info,
            std::time::Duration::from_secs(2),
        );
    }

    pub fn toast_ok_short(&mut self, msg: impl Into<String>) {
        self.set_toast_with_duration(
            msg.into(),
            ToastLevel::Success,
            std::time::Duration::from_secs(2),
        );
    }

    pub fn toast_ok(&mut self, msg: impl Into<String>) {
        self.set_toast(msg.into(), ToastLevel::Success);
    }

    pub fn toast_err(&mut self, msg: impl Into<String>) {
        self.set_toast(msg.into(), ToastLevel::Error);
    }

    fn set_toast(&mut self, message: String, level: ToastLevel) {
        let duration = match level {
            ToastLevel::Info => std::time::Duration::from_secs(2),
            ToastLevel::Success | ToastLevel::Error => std::time::Duration::from_secs(6),
        };
        self.set_toast_with_duration(message, level, duration);
    }

    fn set_toast_with_duration(
        &mut self,
        message: String,
        level: ToastLevel,
        duration: std::time::Duration,
    ) {
        let pending = PendingToast {
            message,
            level,
            duration,
        };
        if self.toast.is_none() {
            self.show_toast(pending);
        } else {
            self.toast_queue.push_back(pending);
        }
    }

    pub fn gc_toast(&mut self) {
        if let Some(t) = &self.toast
            && Instant::now() > t.until
        {
            self.toast = None;
            if let Some(next) = self.toast_queue.pop_front() {
                self.show_toast(next);
            }
        }
    }

    fn show_toast(&mut self, pending: PendingToast) {
        self.toast = Some(Toast {
            message: pending.message,
            until: Instant::now() + pending.duration,
            level: pending.level,
        });
    }

    pub fn exchanger_at(&self, idx: usize) -> Option<ExchangerId> {
        self.exchangers.get(idx).map(|e| e.id)
    }

    pub fn single_from_pair(&self) -> (AssetId, NetworkId) {
        (
            crate::splitnow::data::ASSETS[self.single.from_asset_idx],
            crate::splitnow::data::NETWORKS[self.single.from_network_idx],
        )
    }

    pub fn single_to_pair(&self) -> (AssetId, NetworkId) {
        (
            crate::splitnow::data::ASSETS[self.single.to_asset_idx],
            crate::splitnow::data::NETWORKS[self.single.to_network_idx],
        )
    }

    pub fn multi_from_pair(&self) -> (AssetId, NetworkId) {
        (
            crate::splitnow::data::ASSETS[self.multi.from_asset_idx],
            crate::splitnow::data::NETWORKS[self.multi.from_network_idx],
        )
    }

    pub fn restore_last_order(&mut self) {
        match persist::load_last_order() {
            Ok(Some(status)) => {
                self.status = status;
            }
            Ok(None) => {}
            Err(e) => self.toast_err(format!("load last order: {e}")),
        }
    }

    pub fn sync_account_api_key_input(&mut self) {
        let api_key = self
            .vault
            .as_ref()
            .and_then(|vault| vault.api_key())
            .unwrap_or_default();
        self.account.api_key_input = Input::from(api_key);
        self.account.api_key_error = None;
    }

    pub fn open_account_screen(&mut self) {
        self.sync_account_api_key_input();
        self.account.editing_api_key = false;
        self.account.reveal_api_key = false;
        self.screen = Screen::Account;
        crate::account::refresh_balances(self);
    }
}

pub fn handle_quote(app: &mut App, res: Result<QuoteData, String>) {
    match res {
        Ok(q) => {
            app.toast_ok("quote ready");
            app.single.quote = Some(q);
            app.single.submitting = false;
        }
        Err(e) => {
            app.single.error = Some(e.clone());
            app.single.submitting = false;
            app.toast_err(format!("quote error: {e}"));
        }
    }
}

pub fn handle_order(app: &mut App, res: Result<OrderData, String>) {
    match res {
        Ok(o) => {
            let id = o.order_id.clone();
            app.toast_ok(format!("order {} created", &id));
            app.status.order_id = Some(id);
            app.status.latest = None;
            app.status.last_poll = None;
            app.status.error = None;
            app.status.deposit_address = Some(o.deposit_address.clone());
            app.status.deposit_amount = Some(o.deposit_amount);
            app.single.order = Some(o.clone());
            app.multi.order = Some(o);
            if let Err(e) = persist::save_last_order(&app.status) {
                app.toast_err(format!("save last order: {e}"));
            }
            app.single.submitting = false;
            app.multi.submitting = false;
            app.screen = Screen::OrderStatus;
        }
        Err(e) => {
            app.single.error = Some(e.clone());
            app.multi.error = Some(e.clone());
            app.single.submitting = false;
            app.multi.submitting = false;
            app.toast_err(format!("order error: {e}"));
        }
    }
}

pub fn handle_status(app: &mut App, res: Result<Order, String>) {
    match res {
        Ok(order) => {
            let prev = app
                .status
                .latest
                .as_ref()
                .map(|current| current.status_short);
            let next = order.status_short;
            app.status.deposit_address = Some(order.deposit_wallet_address.clone());
            app.status.deposit_amount = Some(order.deposit_amount);
            app.status.latest = Some(order.clone());
            app.status.last_poll = Some(Instant::now());
            app.status.error = None;
            if let Err(e) = persist::save_last_order(&app.status) {
                app.toast_err(format!("save last order: {e}"));
            }
            if prev != Some(next) {
                match next {
                    OrderStatus::Completed => {
                        app.toast_ok(format!("order {} completed", order.short_id));
                    }
                    OrderStatus::Failed | OrderStatus::Halted => {
                        app.toast_err(format!("order {} {:?}", order.short_id, next));
                    }
                    OrderStatus::Refunded | OrderStatus::Expired => {
                        app.toast_info(format!("order {} {:?}", order.short_id, next));
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            app.status.error = Some(e);
        }
    }
}

pub fn spawn_splitnow_bootstrap(app: &App, client: SplitnowClient) {
    let tx = app.tx.clone();
    let c1 = client.clone();
    tokio::spawn(async move {
        let r = c1.health().await.map_err(|e| format!("{e:#}"));
        let _ = tx.send(crate::events::AppEvent::HealthChecked(r));
    });
    let tx = app.tx.clone();
    tokio::spawn(async move {
        let r = client.exchangers().await.map_err(|e| format!("{e:#}"));
        let _ = tx.send(crate::events::AppEvent::ExchangersLoaded(r));
    });
}

pub fn dispatch_app_event(app: &mut App, evt: AppEvent) {
    match evt {
        AppEvent::Tick => {}
        AppEvent::ExchangersLoaded(Ok(list)) => {
            app.exchangers = list;
            app.toast_info_short(format!("loaded {} exchangers", app.exchangers.len()));
        }
        AppEvent::ExchangersLoaded(Err(e)) => {
            app.exchangers = crate::splitnow::data::fallback_exchangers();
            app.toast_err(format!("exchangers unavailable, using fallback list: {e}"));
        }
        AppEvent::HealthChecked(Ok(true)) => app.toast_ok_short("SplitNOW health OK"),
        AppEvent::HealthChecked(Ok(false)) => app.toast_err("SplitNOW health degraded"),
        AppEvent::HealthChecked(Err(e)) => app.toast_err(format!("health: {e}")),
        AppEvent::AccountBalancesLoaded(Ok(snapshot)) => {
            app.account.balance_loading = false;
            app.account.total_sol = Some(snapshot.total_sol);
            app.account.total_usd = snapshot.total_usd;
            app.account.balance_error = None;
        }
        AppEvent::AccountBalancesLoaded(Err(e)) => {
            app.account.balance_loading = false;
            app.account.balance_error = Some(e.clone());
            app.toast_err(format!("balance refresh: {e}"));
        }
        AppEvent::QuoteReady(r) => handle_quote(app, r),
        AppEvent::OrderReady(r) => handle_order(app, r),
        AppEvent::StatusTick(r) => handle_status(app, *r),
    }
}
