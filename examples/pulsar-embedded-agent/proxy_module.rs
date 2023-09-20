use std::sync::{Arc, Mutex};

use anyhow::Context;
use pulsar_core::pdk::{Module, ModuleContext, PulsarModule, Version};
use tokio::sync::oneshot;

const MODULE_NAME: &str = "proxy-module";

pub struct ProxyModule;

impl Module for ProxyModule {
    fn start() -> PulsarModule {
        PulsarModule::new(
            MODULE_NAME,
            Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            |_ctx: &ModuleContext| Ok(ProxyModule {}),
        )
    }
}

/// Fake module used to extract the ModuleContext out of pulsar.
pub fn module(tx_ctx: oneshot::Sender<ModuleContext>) -> PulsarModule {
    // This code supports starting the module only once. A smarter solution
    // needs to be architected if restarts are required.
    let tx_ctx = Arc::new(Mutex::new(Some(tx_ctx)));
    PulsarModule::new(
        MODULE_NAME,
        Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
        move |ctx: &ModuleContext| {
            let tx_ctx = tx_ctx.clone();
            let tx_ctx = tx_ctx
                .lock()
                .ok()
                .context("Getting mutex failed")?
                .take()
                .context("Module can be started only once")?;
            tx_ctx
                .send(ctx.clone())
                .ok()
                .context("Sending context failed")?;
            Ok(ProxyModule {})
        },
    )
}
