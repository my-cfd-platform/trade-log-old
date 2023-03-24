mod my_sb_entity;
#[cfg(not(feature = "trade-log-writer"))]
mod trade_log;
#[cfg(not(feature = "trade-log-writer"))]
mod trade_log_inner;
#[cfg(not(feature = "trade-log-writer"))]
pub use my_sb_entity::*;
#[cfg(not(feature = "trade-log-writer"))]
use std::sync::Arc;
#[cfg(not(feature = "trade-log-writer"))]
pub use trade_log::*;
#[cfg(not(feature = "trade-log-writer"))]
pub use trade_log_inner::*;

#[cfg(not(feature = "trade-log-writer"))]
lazy_static::lazy_static! {
    pub static ref TRADE_LOG: Arc<TradeLog> = {
        Arc::new(TradeLog::new())
    };

}
