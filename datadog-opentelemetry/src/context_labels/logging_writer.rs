// Copyright 2025-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use super::writer::{ContextLabelWriter, TraceContext};

/// Debug writer to dd-trace-rs logging subsytem
#[derive(Debug, Clone, Copy, Default)]
pub struct LoggingContextWriter;

impl LoggingContextWriter {
    /// Create a new LoggingContextWriter
    pub fn new() -> Self {
        Self
    }
}

impl ContextLabelWriter for LoggingContextWriter {
    fn write_labels(&self, context: &TraceContext) {
        dd_trace::dd_debug!("trace context entered: {}", context);
    }

    fn clear_labels(&self) {
        dd_trace::dd_debug!("trace context cleared");
    }
}
