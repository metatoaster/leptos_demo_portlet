use leptos::prelude::*;

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::sync::{Arc, RwLock};
    use tokio::sync::broadcast::{channel, Receiver, Sender};
    use super::*;

    #[derive(Clone)]
    struct Message;

    struct WaiterInner {
        sender: Sender<Message>,
        resolved: RwLock<bool>,
    }

    #[derive(Clone)]
    pub struct Waiter(Arc<WaiterInner>);

    #[derive(Clone)]
    pub struct MaybeWaiter(Option<Waiter>);

    struct WaiterHandleInner {
        waiter: Waiter,
        receiver: Receiver<Message>,
    }

    pub struct WaiterHandle(Option<WaiterHandleInner>);

    impl MaybeWaiter {
        pub fn subscribe(&self) -> WaiterHandle {
            WaiterHandle(self.0.clone().map(|waiter| {
                WaiterHandleInner {
                    waiter: waiter.clone(),
                    receiver: waiter.0.sender.subscribe(),
                }
            }))
        }
    }

    impl Waiter {
        pub fn maybe() -> MaybeWaiter {
            MaybeWaiter(use_context::<Waiter>())
        }

        pub fn count() {
            let waiter = expect_context::<Waiter>();
            leptos::logging::log!(
                "count of subscribers: {}",
                waiter.0.sender.receiver_count(),
            );
        }

        pub(super) fn complete() {
            let waiter = expect_context::<Waiter>();
            *waiter.0.resolved.write().unwrap() = true;
            if let Ok(_) = waiter.0.sender.send(Message) {
                leptos::logging::log!(
                    "broadcasted complete to {} subscribers",
                    waiter.0.sender.receiver_count(),
                );
            } else {
                leptos::logging::log!(
                    "no subscribers available to receive completion"
                );
            }
        }
    }

    impl WaiterHandle {
        pub async fn wait(mut self) {
            if let Some(mut inner) = self.0.take() {
                if !*inner.waiter.0.resolved.read().unwrap() {
                    inner.receiver
                        .recv()
                        .await
                        .expect("internal error: sender not properly managed");
                }
            }
        }
    }

    pub(super) fn provide_async_wait() -> Waiter {
        let (sender, _) = channel(1);
        let resolved = RwLock::new(false);
        let waiter = Waiter(WaiterInner { sender, resolved }.into());
        provide_context(waiter.clone());
        waiter
    }
}

#[cfg(feature = "ssr")]
use ssr::*;

#[component]
pub fn SyncAwait(children: Children) -> impl IntoView {
    let enter = move || {
        leptos::logging::log!("entering SyncAwait");
        #[cfg(feature = "ssr")]
        provide_async_wait();
    };

    let exit = move || {
        #[cfg(feature = "ssr")]
        Waiter::complete();
        leptos::logging::log!("exiting SyncAwait");
    };

    view! {
        {enter}
        {children()}
        {exit}
    }
}
