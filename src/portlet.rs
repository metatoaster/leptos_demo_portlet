use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::sync_await::ssr::Waiter;

#[derive(Clone, Debug, Default)]
pub struct PortletCtx<T> {
    inner: Option<ArcResource<Result<T, ServerFnError>>>,
}

// `PartialEq` is required for `PortletCtx<T>` in order for it to be
// enclosed inside a `ReadSignal`.  Since implementing `PartialEq` for
// `ArcResource<...> is not exactly feasible, and that what this use
// case ultimately cares about is whether or not there is some resource
// being assigned, and thus comparison using `.is_some()` is sufficient.
impl<T> PartialEq for PortletCtx<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.is_some() == other.inner.is_some()
    }
}

impl<T> PortletCtx<T>
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + PartialEq
        + Send
        + Sync
        + IntoRender
        + 'static,
{
    /// Clear the resource in the portlet.  The component using this
    /// may decide to not render anything.
    pub fn clear(&mut self) {
        leptos::logging::log!("PortletCtx clear");
        self.inner = None;
    }

    /// Set the resource for this portlet.
    pub fn set(&mut self, value: ArcResource<Result<T, ServerFnError>>) {
        leptos::logging::log!("PortletCtx set");
        self.inner = Some(value);
    }

    pub fn provide() {
        let (rs, ws) = signal(PortletCtx::<T> { inner: None });
        provide_context(rs);
        provide_context(ws);
    }

    pub fn expect_renderer() -> PortletCtxRenderer<T> {
        PortletCtxRenderer::from(expect_context::<ReadSignal<PortletCtx<T>>>())
    }
}

#[derive(Clone)]
pub struct PortletCtxRenderer<T>{
    inner: ReadSignal<PortletCtx<T>>,
}

impl<T> From<ReadSignal<PortletCtx<T>>> for PortletCtxRenderer<T> {
    fn from(inner: ReadSignal<PortletCtx<T>>) -> Self {
        Self { inner }
    }
}

impl<T> IntoRender for PortletCtxRenderer<T>
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + std::fmt::Debug
        + PartialEq
        + Send
        + Sync
        + IntoRender
        + 'static,
    <T as leptos::prelude::IntoRender>::Output: RenderHtml,
{
    type Output = Suspend<Result<Option<AnyView>, ServerFnError>>;

    fn into_render(self) -> Self::Output {
        #[cfg(feature = "ssr")]
        let waiter = Waiter::maybe();

        leptos::logging::log!("PortletCtxRender Suspend entering");
        let sus = Suspend::new(async move {
            let result = Resource::new_blocking(
                {
                    let rs = self.inner.clone();
                    move || rs.get()
                },
                move |_| {
                    let rs = self.inner.clone();
                    #[cfg(feature = "ssr")]
                    let waiter = waiter.clone();
                    async move {
                        #[cfg(feature = "ssr")]
                        waiter.subscribe().wait().await;
                        let ctx = rs.get();
                        leptos::logging::log!("portlet_ctx = {ctx:?}");
                        if let Some(resource) = ctx.inner {
                            Ok::<_, ServerFnError>(Some(resource.await?))
                        } else {
                            Ok(None)
                        }
                    }
                },
            ).await?;

            if let Some(result) = result {
                leptos::logging::log!("returning actual view");
                Ok::<_, ServerFnError>(Some(result.into_render().into_any()))
            } else {
                leptos::logging::log!("returning empty view");
                Ok(None)
            }

        });
        leptos::logging::log!("PortletCtxRender Suspend exiting");
        sus
    }
}

pub fn render_portlet<T>() -> impl IntoView
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Clone
        + std::fmt::Debug
        + PartialEq
        + Send
        + Sync
        + IntoRender
        + 'static,
    <T as leptos::prelude::IntoRender>::Output: RenderHtml,
{
    let renderer = PortletCtx::<T>::expect_renderer();
    view! { <Transition>{move || renderer.clone().into_render()}</Transition> }
}
