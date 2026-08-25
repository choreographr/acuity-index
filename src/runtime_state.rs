use std::collections::HashMap;
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};

use subxt::{
    OnlineClient, PolkadotConfig, config::RpcConfigFor, rpcs::methods::legacy::LegacyRpcMethods,
};
use tokio::sync::mpsc;
use tracing::error;

use crate::metrics::Metrics;
use crate::protocol::{JsonRpcNotification, Key};

pub fn lock_or_recover<'a, T>(mutex: &'a Mutex<T>, name: &str) -> MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("Recovering poisoned mutex: {name}");
            poisoned.into_inner()
        }
    }
}

#[derive(Clone)]
pub struct SubscriptionEntry {
    pub tx: mpsc::Sender<JsonRpcNotification>,
    pub kind: SubscriptionKind,
}

#[derive(Clone)]
pub enum SubscriptionKind {
    Status,
    Events { key: Key },
}

pub struct RuntimeState {
    pub(crate) subscriptions: Mutex<HashMap<String, SubscriptionEntry>>,
    pub(crate) metrics: Arc<Metrics>,
    api: Mutex<Option<OnlineClient<PolkadotConfig>>>,
    rpc: Mutex<Option<LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>>>,
    finalized_mode: AtomicBool,
}

impl RuntimeState {
    #[cfg(test)]
    pub fn new(max_total_subscriptions: usize) -> Self {
        Self::with_metrics(max_total_subscriptions, Arc::new(Metrics::new()))
    }

    pub fn with_metrics(_max_total_subscriptions: usize, metrics: Arc<Metrics>) -> Self {
        Self {
            subscriptions: Mutex::new(HashMap::new()),
            metrics,
            api: Mutex::new(None),
            rpc: Mutex::new(None),
            finalized_mode: AtomicBool::new(false),
        }
    }

    pub fn set_api(&self, api: Option<OnlineClient<PolkadotConfig>>) {
        *lock_or_recover(&self.api, "runtime_api") = api;
    }

    pub fn api(&self) -> Option<OnlineClient<PolkadotConfig>> {
        lock_or_recover(&self.api, "runtime_api").clone()
    }

    pub fn set_rpc(&self, rpc: Option<LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>>) {
        *lock_or_recover(&self.rpc, "runtime_rpc") = rpc;
    }

    pub fn rpc(&self) -> Option<LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>> {
        lock_or_recover(&self.rpc, "runtime_rpc").clone()
    }

    pub fn clients(
        &self,
    ) -> Option<(
        OnlineClient<PolkadotConfig>,
        LegacyRpcMethods<RpcConfigFor<PolkadotConfig>>,
    )> {
        Some((self.api()?, self.rpc()?))
    }

    pub fn set_finalized_mode(&self, finalized_mode: bool) {
        self.finalized_mode.store(finalized_mode, Ordering::Relaxed);
    }

    pub fn finalized_mode(&self) -> bool {
        self.finalized_mode.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_or_recover_returns_guard_on_clean_lock() {
        let mutex = Mutex::new(42u32);
        let guard = lock_or_recover(&mutex, "test");
        assert_eq!(*guard, 42);
    }

    #[test]
    fn lock_or_recover_recovers_a_poisoned_mutex() {
        let mutex = Mutex::new(String::from("value"));
        // Poison the mutex by panicking while holding the lock.
        let _panic_guard = catch_poison(&mutex);
        let guard = lock_or_recover(&mutex, "test");
        assert_eq!(*guard, "value");
    }

    #[test]
    fn finalized_mode_defaults_false_and_toggles() {
        let state = RuntimeState::new(10);
        assert!(!state.finalized_mode());
        state.set_finalized_mode(true);
        assert!(state.finalized_mode());
        state.set_finalized_mode(false);
        assert!(!state.finalized_mode());
    }

    #[test]
    fn clients_returns_none_before_api_and_rpc_set() {
        let state = RuntimeState::new(10);
        assert!(state.clients().is_none());
        assert!(state.api().is_none());
        assert!(state.rpc().is_none());
    }

    #[test]
    fn with_metrics_initializes_clean_state() {
        let metrics = Arc::new(Metrics::new());
        let state = RuntimeState::with_metrics(5, metrics.clone());
        assert!(state.clients().is_none());
        // The metrics counter is wired by Arc: both handles reference the same set.
        assert_eq!(Arc::strong_count(&state.metrics), 2);
        // The caller handle and the state handle are the same underlying set.
        assert_eq!(Arc::as_ptr(&state.metrics), Arc::as_ptr(&metrics));
    }

    // Poison a mutex by panicking inside a dropped scope that holds the guard,
    // returning the guard so it is poisoned while held.
    fn catch_poison(mutex: &Mutex<String>) -> &'static str {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("deliberate poison");
        }));
        assert!(result.is_err());
        "poisoned"
    }
}
