// Copyright 2025-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use super::writer::{ContextLabelWriter, TraceContext};
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{context::ContextObserver, Context};

/// Observer that extracts trace context and writes labels on context changes
/// This is glue between OTel's ContextWriter and a configurable writer.
/// This makes it easy for us to plug in a debugging impl while we are hacking
/// about.
pub struct ContextLabelObserver<W> {
    writer: W,
}

impl<W: ContextLabelWriter> ContextLabelObserver<W> {
    /// Create a new observer with the given writer
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Extract trace context from an OTel context
    ///
    /// Returns `None` if:
    /// - The context has no active span
    /// - The span context is invalid
    /// - The span is not sampled
    ///
    fn extract_trace_context(ctx: &Context) -> Option<TraceContext> {
        if !ctx.has_active_span() {
            return None;
        }

        let span = ctx.span();
        let span_context = span.span_context();

        // Only export valid, sampled spans to reduce overhead
        if !span_context.is_valid() || !span_context.is_sampled() {
            return None;
        }

        // Get local root span ID, falling back to current span ID if not available
        let local_root_span_id = ctx
            .local_root_span_id()
            .unwrap_or_else(|| span_context.span_id());

        Some(TraceContext {
            trace_id: format!("{:032x}", span_context.trace_id()),
            span_id: format!("{:016x}", span_context.span_id()),
            local_root_span_id: format!("{:016x}", local_root_span_id),
            service_name: None,    // TODO: Extract from resource attributes. Something something conventions?
            resource_name: None,   // TODO: Extract from span name/attributes
        })
    }
}

impl<W: ContextLabelWriter> ContextObserver for ContextLabelObserver<W> {
    /// Called when entering a new context
    fn on_context_enter(&self, _from: &Context, to: &Context) {
        if let Some(trace_ctx) = Self::extract_trace_context(to) {
            self.writer.write_labels(&trace_ctx);
        } else {
            // No active span in the new context, clear labels
            self.writer.clear_labels();
        }
    }

    /// Called when exiting to a previous context
    fn on_context_exit(&self, _from: &Context, to: &Context) {
        if let Some(trace_ctx) = Self::extract_trace_context(to) {
            self.writer.write_labels(&trace_ctx);
        } else {
            // Returning to a context with no active span, clear labels
            self.writer.clear_labels();
        }
    }
}
