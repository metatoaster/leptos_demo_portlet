use leptos::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct PortletCtx<
    T: serde::Serialize
    + serde::de::DeserializeOwned
    + Clone
    + PartialEq
    + Send
    + Sync
    + IntoRender
    + 'static
> {
    inner: Option<ArcResource<Result<T, ServerFnError>>>,
}

impl<
    T: serde::Serialize
    + serde::de::DeserializeOwned
    + Clone
    + PartialEq
    + Send
    + Sync
    + IntoRender
    + 'static
> PortletCtx<T> {
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
pub struct PortletCtxRenderer<
    T: serde::Serialize
    + serde::de::DeserializeOwned
    + Clone
    + PartialEq
    + Send
    + Sync
    + IntoRender
    + 'static
>(ReadSignal<PortletCtx<T>>);

impl<
    T: serde::Serialize
    + serde::de::DeserializeOwned
    + Clone
    + PartialEq
    + Send
    + Sync
    + IntoRender
    + 'static
> From<ReadSignal<PortletCtx<T>>> for PortletCtxRenderer<T> {
    fn from(value: ReadSignal<PortletCtx<T>>) -> Self {
        Self(value)
    }
}

impl<
    T: serde::Serialize
    + serde::de::DeserializeOwned
    + Clone
    + std::fmt::Debug
    + PartialEq
    + Send
    + Sync
    + IntoRender
    + 'static
> IntoRender for PortletCtxRenderer<T>
where
    <T as leptos::prelude::IntoRender>::Output: RenderHtml,
{
    type Output = Suspend<Result<AnyView, ServerFnError>>;

    fn into_render(self) -> Self::Output {
        Suspend::new(async move {
            let ctx = self.0.get();
            leptos::logging::log!("portlet_ctx = {ctx:?}");
            if let Some(resource) = ctx.inner {
                Ok::<_, ServerFnError>(resource.await?
                    .into_render()
                    .into_any()
                )
            } else {
                leptos::logging::log!("returning empty view");
                // XXX somehow this dummy value works around the hydration
                // error?
                Ok::<_, ServerFnError>(view! {
                    <noscript></noscript>
                }.into_any())
            }
        })
    }
}
