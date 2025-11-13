// Copyright 2025-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use super::writer::{ContextLabelWriter, TraceContext};

/// Writer implementation using Polar Signals custom-labels TL lib.
#[cfg(feature = "context-observer")]
pub struct CustomLabelsWriter;

#[cfg(feature = "context-observer")]
impl CustomLabelsWriter {
    /// Create a new CustomLabelsWriter
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "context-observer")]
impl Default for CustomLabelsWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "context-observer")]
impl ContextLabelWriter for CustomLabelsWriter {
    fn write_labels(&self, context: &TraceContext) {
        let thread_id = std::thread::current().id();
        dd_trace::dd_debug!(
            "write_labels called on thread {:?} - trace_id={}, span_id={}",
            thread_id,
            context.trace_id,
            context.span_id
        );

        // Initialize the labelset if it doesn't exist for this thread yet
        unsafe {
            if custom_labels::sys::labelset_current().is_null() {
                dd_trace::dd_debug!("Initializing new labelset for thread {:?}", thread_id);
                let l = custom_labels::sys::labelset_new(0);
                if l.is_null() {
                    dd_trace::dd_error!("Failed to allocate labelset for thread {:?}", thread_id);
                    return;
                }
                custom_labels::sys::labelset_replace(l);
            }
        }

        // Access the current thread's label set via the CURRENT_LABELSET constant
        // TODO - we're just mutating inline here. we should be able to do this with an atomic
        // swap of the labelset.
        let labelset = &custom_labels::CURRENT_LABELSET;

        labelset.set("trace_id", &context.trace_id);
        labelset.set("span_id", &context.span_id);
        labelset.set("local_root_span_id", &context.local_root_span_id);

        if let Some(ref service) = context.service_name {
            labelset.set("service", service);
        }

        if let Some(ref resource) = context.resource_name {
            labelset.set("resource", resource);
        }

        dd_trace::dd_debug!("Labels written successfully");
    }

    fn clear_labels(&self) {
        dd_trace::dd_debug!("clear_labels called");

        // Only clear if labelset exists (defensive check)
        unsafe {
            if custom_labels::sys::labelset_current().is_null() {
                dd_trace::dd_debug!("No labelset to clear");
                return;
            }
        }

        let labelset = &custom_labels::CURRENT_LABELSET;

        // Delete all labels we set
        labelset.delete("trace_id");
        labelset.delete("span_id");
        labelset.delete("local_root_span_id");
        labelset.delete("service");
        labelset.delete("resource");

        dd_trace::dd_debug!("Labels cleared successfully");
    }
}
