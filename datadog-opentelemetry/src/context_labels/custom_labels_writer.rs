// Copyright 2025-Present Datadog, Inc. https://www.datadoghq.com/
// SPDX-License-Identifier: Apache-2.0

use super::writer::{ContextLabelWriter, ExtractedSpanData};
use crate::v2_keys;

/// Writer implementation using the custom-labels lib
///
/// For the sake of giving a complete, standalone demo, this writes both the PolarSignals v1
/// format (custom labelsets), and our V2 format. This is obviously not a thing to do for
/// Serious Production Code!
/// 
#[cfg(feature = "context-observer")]
pub struct CustomLabelsWriter {
    http_route_key: custom_labels::v2::KeyHandle,
    some_custom_field_key: custom_labels::v2::KeyHandle,
}

#[cfg(feature = "context-observer")]
impl CustomLabelsWriter {
    /// Create a new CustomLabelsWriter
    pub fn new() -> Self {
        Self {
            http_route_key: custom_labels::v2::KeyHandle::new(v2_keys::HTTP_ROUTE_IDX),
            some_custom_field_key: custom_labels::v2::KeyHandle::new(v2_keys::SOME_CUSTOM_FIELD_IDX),
        }
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
    fn write_labels(&self, data: &ExtractedSpanData) {
        let thread_id = std::thread::current().id();
        dd_trace::dd_debug!(
            "write_labels called on thread {:?} - trace_id={:032x}, span_id={:016x}",
            thread_id,
            u128::from_be_bytes(data.trace_id),
            u64::from_be_bytes(data.span_id)
        );

        //
        // ** PolarSignals / V1 format **
        //

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

        // Store raw byte arrays directly - the profiler will handle formatting
        labelset.set("trace_id", &data.trace_id[..]);
        labelset.set("span_id", &data.span_id[..]);
        labelset.set("local_root_span_id", &data.local_root_span_id[..]);

        if let Some(route) = data.http_route {
            labelset.set("http_route", route);
        }

        dd_trace::dd_debug!("V1 labels written successfully");

        //
        // V2 format
        //

        let http_route_key = self.http_route_key;
        let some_custom_field_key = self.some_custom_field_key;
        custom_labels::v2::set_current_record(Some(&data.span_id), |builder| {
            builder.set_trace(&data.trace_id, &data.span_id, &data.local_root_span_id);
            if let Some(route) = data.http_route {
                let _ = builder.set_attr_str(http_route_key, route);
            }
            // Always set some_custom_field to "123abc"
            let _ = builder.set_attr_str(some_custom_field_key, "123abc");
        });

        dd_trace::dd_debug!("V2 record written successfully");
    }

    fn clear_labels(&self) {
        dd_trace::dd_debug!("clear_labels called");

        //
        // v1 format
        //

        unsafe {
            if custom_labels::sys::labelset_current().is_null() {
                dd_trace::dd_debug!("No V1 labelset to clear");
            } else {
                let labelset = &custom_labels::CURRENT_LABELSET;

                // Delete all labels we set
                labelset.delete("trace_id");
                labelset.delete("span_id");
                labelset.delete("local_root_span_id");
                labelset.delete("http_route");

                dd_trace::dd_debug!("V1 labels cleared successfully");
            }
        }

        //
        // v2 format
        //

        custom_labels::v2::clear_current_record();
        dd_trace::dd_debug!("V2 record cleared successfully");
    }
}
