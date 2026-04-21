use crate::account::AccountBalances;
use crate::splitnow::client::AvailableExchanger;
use splitnow::{Order, OrderData, QuoteData};
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum AppEvent {
    Tick,
    ExchangersLoaded(Result<Vec<AvailableExchanger>, String>),
    HealthChecked(Result<bool, String>),
    AccountBalancesLoaded(Result<AccountBalances, String>),
    QuoteReady(Result<QuoteData, String>),
    OrderReady(Result<OrderData, String>),
    StatusTick(Box<Result<Order, String>>),
}

pub type Sender = mpsc::UnboundedSender<AppEvent>;
pub type Receiver = mpsc::UnboundedReceiver<AppEvent>;

pub fn channel() -> (Sender, Receiver) {
    mpsc::unbounded_channel()
}
