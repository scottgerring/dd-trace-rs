// Copyright 2025-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use super::writer::{ContextLabelWriter, TraceContext};
use crate::TraceRegistry;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{context::ContextObserver, Context};

/// Observer that extracts trace context and writes labels on context changes
/// This is glue between OTel's ContextWriter and a configurable writer.
/// This makes it easy for us to plug in a debugging impl while we are hacking
/// about.
pub struct ContextLabelObserver<W> {
    writer: W,
    registry: TraceRegistry,
}

impl<W: ContextLabelWriter> ContextLabelObserver<W> {
    /// Create a new observer with the given writer and registry
    pub fn new(writer: W, registry: TraceRegistry) -> Self {
        Self { writer, registry }
    }

    /// Extract trace context from an OTel context
    ///
    /// Returns `None` if:
    /// - The context has no active span
    /// - The span context is invalid
    /// - The span is not sampled
    ///
    fn extract_trace_context(&self, ctx: &Context) -> Option<TraceContext> {
        if !ctx.has_active_span() {
            return None;
        }

        let span = ctx.span();
        let span_context = span.span_context();

        // Only export valid, sampled spans to reduce overhead
        if !span_context.is_valid() || !span_context.is_sampled() {
            return None;
        }

        let trace_id = span_context.trace_id().to_bytes();
        let span_id = span_context.span_id().to_bytes();

        // Get local root span ID from registry, falling back to current span ID if not available
        let local_root_span_id = self
            .registry
            .get_local_root_span_id(trace_id)
            .unwrap_or(span_id);

        // Get active span metadata (http route) from registry
        let metadata = self.registry.get_active_span_metadata(trace_id, span_id);

        Some(TraceContext {
            trace_id: format!("{:032x}", u128::from_be_bytes(trace_id)),
            span_id: format!("{:016x}", u64::from_be_bytes(span_id)),
            local_root_span_id: format!("{:016x}", u64::from_be_bytes(local_root_span_id)),
            http_route: metadata.as_ref().and_then(|m| m.http_route.clone()),
        })
    }
}

impl<W: ContextLabelWriter> ContextObserver for ContextLabelObserver<W> {
    /// Called when entering a new context
    fn on_context_enter(&self, _from: &Context, to: &Context) {
        if let Some(trace_ctx) = self.extract_trace_context(to) {
            self.writer.write_labels(&trace_ctx);
        } else {
            // No active span in the new context, clear labels
            self.writer.clear_labels();
        }
    }

    /// Called when exiting to a previous context
    fn on_context_exit(&self, _from: &Context, to: &Context) {
        if let Some(trace_ctx) = self.extract_trace_context(to) {
            self.writer.write_labels(&trace_ctx);
        } else {
            // Returning to a context with no active span, clear labels
            self.writer.clear_labels();
        }
    }
}
