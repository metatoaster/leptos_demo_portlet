use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::sync_await::ssr::Waiter;

#[derive(Clone, Debug, Default)]
pub struct PortletCtx<T> {
    inner: Option<ArcResource<Result<T, ServerFnError>>>,
    refresh: RwSignal<usize>,
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
        self.refresh.try_update(|n| *n += 1);
        self.inner = None;
    }

    /// Set the resource for this portlet.
    pub fn set(&mut self, value: ArcResource<Result<T, ServerFnError>>) {
        leptos::logging::log!("PortletCtx set");
        self.refresh.try_update(|n| *n += 1);
        self.inner = Some(value);
    }

    /// The reason why there is no constructor provided and only done so
    /// via signal is to have these contexts function as a singleton.
    pub fn provide() {
        let (rs, ws) = signal(PortletCtx::<T> {
            inner: None,
            refresh: RwSignal::new(0),
        });
        provide_context(rs);
        provide_context(ws);
    }

    pub fn expect_renderer() -> PortletCtxRenderer<T> {
        let inner = expect_context::<ReadSignal<PortletCtx<T>>>();
        let refresh = inner.get_untracked().refresh.clone();
        PortletCtxRenderer { inner, refresh }
    }
}

#[derive(Clone)]
pub struct PortletCtxRenderer<T>{
    inner: ReadSignal<PortletCtx<T>>,
    refresh: RwSignal<usize>,
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
        let refresh = self.refresh.clone();
        let resource = Resource::new_blocking(
            {
                move || {
                    leptos::logging::log!("into_render suspend resource signaled!");
                    refresh.get()
                }
            },
            move |id| {
                let rs = self.inner.clone();
                leptos::logging::log!("refresh id {id}");
                #[cfg(feature = "ssr")]
                let waiter = waiter.clone();
                async move {
                    leptos::logging::log!("PortletCtxRender Suspend resource entering");
                    leptos::logging::log!("refresh id {id}");
                    #[cfg(feature = "ssr")]
                    waiter.subscribe().wait().await;
                    let ctx = rs.get();
                    leptos::logging::log!("portlet_ctx.inner = {:?}", ctx.inner);
                    let result = if let Some(resource) = ctx.inner {
                        Ok::<_, ServerFnError>(Some(resource.await?))
                    } else {
                        Ok(None)
                    };
                    leptos::logging::log!("PortletCtxRender Suspend resource exiting");
                    result
                }
            },
        );

        Suspend::new(async move {
            leptos::logging::log!("PortletCtxRender Suspend entering");
            let result = resource.await?;
            let result = if let Some(result) = result {
                leptos::logging::log!("returning actual view");
                Ok::<_, ServerFnError>(Some(result.into_render().into_any()))
            } else {
                leptos::logging::log!("returning empty view");
                Ok(None)
            };
            leptos::logging::log!("PortletCtxRender Suspend exiting");
            result
        })
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
