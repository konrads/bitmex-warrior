use std::sync::mpsc;
use reqwest;
use crate::model::*;
use crate::model::{ExchangeCmd, OrchestratorEvent::*, PriceType::*};

#[tokio::main]
pub async fn issue_order(cmd: &ExchangeCmd, tx: &mpsc::Sender<OrchestratorEvent>) -> Result<(), reqwest::Error> {
    let res = reqwest::get("https://hyper.rs").await?;
    println!("Status: {}", res.status());
    let body = res.text().await?;
    println!("Body:\n\n{}", body);
    tx.send(NewStatus(body[..50].to_string()));
    Ok(())
}