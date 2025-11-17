// Copyright 2025-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

/// Trace context extracted from OpenTelemetry Context
/// TODO if we really want this extra layer of mapping, or not.
/// TODO encoding trace/span ids as hex strings is rather wasteful
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    /// Full trace ID in hexadecimal format (32 characters for 128-bit trace ID)
    pub trace_id: String,

    /// Current span ID in hexadecimal format (16 characters for 64-bit span ID)
    pub span_id: String,

    /// Local root span ID - the topmost span within this service for this trace
    /// in hexadecimal format (16 characters for 64-bit span ID)
    pub local_root_span_id: String,

    /// HTTP route/path (e.g., "/do_work")
    pub http_route: Option<String>,
}

impl std::fmt::Display for TraceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "trace_id={}, span_id={}, local_root_span_id={}",
            self.trace_id, self.span_id, self.local_root_span_id
        )?;

        if let Some(ref route) = self.http_route {
            write!(f, ", http_route={}", route)?;
        }

        Ok(())
    }
}

/// A thing that consumes TraceContext and writes it out someplace. Typically this
/// someplace would be our polarsignals TL impl, or the console!
pub trait ContextLabelWriter: Send + Sync + 'static {
    /// Write labels. This is called every time we enter an OTel context, so
    /// impls should be snappy.
    fn write_labels(&self, context: &TraceContext);

    /// Clear all labels
    ///
    /// Called when we leave a context and no active context remains.
    fn clear_labels(&self);
}
