use std::{collections::HashMap, sync::Arc, time::Duration};

use cfd_engine_sb_contracts::TradeLogSbModel;
use my_service_bus_abstractions::publisher::MyServiceBusPublisher;
use my_service_bus_tcp_client::MyServiceBusClient;
use rust_extensions::{date_time::DateTimeAsMicroseconds, IntoStringOrStr, Logger};
use tokio::sync::Mutex;

use crate::TradeLogInner;

pub struct TradeLog {
    inner: Arc<Mutex<TradeLogInner>>,
}

impl TradeLog {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TradeLogInner::new())),
        }
    }

    pub async fn start(&self, sb_client: &MyServiceBusClient) {
        let mut write_access = self.inner.lock().await;

        if write_access.is_started() {
            panic!("TradeLog is already started");
        }

        let publisher = sb_client.get_publisher(true).await;
        write_access.sb_publisher = Some(Arc::new(publisher));

        let inner = self.inner.clone();

        tokio::spawn(write_to_trade_log(inner));
    }

    pub async fn write<'s>(
        &self,
        trader_id: impl IntoStringOrStr<'s>,
        account_id: impl IntoStringOrStr<'s>,
        process_id: Option<impl IntoStringOrStr<'s>>,
        message: impl IntoStringOrStr<'s>,
        data: Option<impl serde::Serialize>,
    ) {
        let item = TradeLogSbModel {
            trader_id: trader_id.into_string_or_str().to_string(),
            account_id: account_id.into_string_or_str().to_string(),
            process_id: if let Some(process_id) = process_id {
                process_id.into_string_or_str().to_string()
            } else {
                "".to_string()
            },
            message: message.into_string_or_str().to_string(),
            data: if let Some(data) = &data {
                serde_json::to_string(data).unwrap()
            } else {
                "".to_string()
            },
            date_time_unix_micros: DateTimeAsMicroseconds::now().unix_microseconds,
        };

        let mut write_access = self.inner.lock().await;
        if !write_access.is_started() {
            panic!("TradeLog is not started");
        }

        write_access.add(item);
    }

    pub async fn stop(&self) {
        loop {
            let (items_in_queue, items_on_delivery) = {
                let mut write_access = self.inner.lock().await;

                write_access.stopping = true;

                (
                    write_access.get_elements_in_queue(),
                    write_access.get_items_on_delivery(),
                )
            };

            if items_in_queue == 0 && items_on_delivery == 0 {
                return;
            }

            if items_in_queue > 0 {
                println!("TradeLog: {} items in queue. Waiting", items_in_queue);
            }

            if items_on_delivery > 0 {
                println!("TradeLog: {} items on delivery. Waiting", items_on_delivery);
            }

            tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        }
    }
}

async fn write_to_trade_log(inner: Arc<Mutex<TradeLogInner>>) {
    loop {
        let to_write = {
            let mut write_access = inner.lock().await;
            write_access.get_elements_to_deliver()
        };

        match to_write {
            Some((to_write, publisher)) => {
                deliver_it(inner.clone(), to_write, publisher).await;
            }
            None => {
                {
                    let write_access = inner.lock().await;

                    if write_access.stopping
                        && write_access.get_elements_in_queue() == 0
                        && write_access.get_items_on_delivery() == 0
                    {
                        return;
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
}

async fn deliver_it(
    inner: Arc<Mutex<TradeLogInner>>,
    to_write: Vec<TradeLogSbModel>,
    publisher: Arc<MyServiceBusPublisher<TradeLogSbModel>>,
) {
    loop {
        match publisher.publish_messages(&to_write).await {
            Ok(_) => {
                let mut write_access = inner.lock().await;
                write_access.delivered();
                break;
            }
            Err(err) => {
                let mut account_ids = String::new();
                let mut i = 0;
                for itm in &to_write {
                    account_ids.push_str(&itm.account_id);
                    account_ids.push(';');
                    i += 1;
                    if i >= 10 {
                        break;
                    }
                }

                let mut ctx = HashMap::new();

                ctx.insert("accountIds".to_string(), account_ids);

                my_logger::LOGGER.write_error(
                    "Publish TradeLog to Sb".to_string(),
                    format!("{:?}", err),
                    Some(ctx),
                );

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
