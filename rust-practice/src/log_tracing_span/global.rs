const NAME: &'static str = "default-root";

#[macro_export]
macro_rules! make_span {
    () => {
        $crate::log_tracing_span::global::get_span()
    };

    ($name:expr) => {
        tracing::span!(parent: $crate::log_tracing_span::global::get_span(), tracing::Level::DEBUG, $name)
    };

    ($name:expr, $($fields:tt)*) => {
        tracing::span!(parent: $crate::log_tracing_span::global::get_span(), tracing::Level::DEBUG, $name, $($fields)*)
    }
}

pub fn init_span(span: tracing::Span) {
    inner_span().get_or_init(|| span);
}

pub(crate) fn get_span() -> tracing::Span {
    get_span_ref().clone()
}


pub(crate) fn get_span_ref() -> &'static tracing::Span {
    inner_span().get_or_init(||{
        let span = tracing::span!(tracing::Level::DEBUG, NAME);
        span
    })
}

fn inner_span() -> &'static std::sync::OnceLock<tracing::Span> {
    static SPAN: std::sync::OnceLock<tracing::Span> = std::sync::OnceLock::new();
    &SPAN
}

