//! Plugin GUI support - Parameter binding and widget integration.
//!
//! Provides reusable UI components for audio plugins with automatic
//! parameter synchronization between UI and DSP threads.
//!
//! # Architecture
//!
//! ```text
//! UI Thread                    DSP Thread
//! ─────────                    ──────────
//! Widget::update()             DspNode::process()
//!     │                            │
//!     │  ParamBridge (arc-swap)    │
//!     └────────────────────────────┤
//!                                  │
//!                            Param::tick()
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use aether_ui::plugin_gui::{ParamWidget, ParamBridge};
//! use aetherdsp_core::param::Param;
//!
//! // Create parameter bridge
//! let mut bridge = ParamBridge::new();
//! let gain_id = bridge.add_param("gain", 0.0, 1.0, 0.5);
//!
//! // In UI thread: bind widget to parameter
//! // let widget = ParamWidget::slider(gain_id, &bridge);
//!
//! // In DSP thread: read parameter value
//! let gain_value = bridge.get_value(gain_id);
//! ```

use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;

/// Parameter identifier.
pub type ParamId = usize;

/// Parameter metadata.
#[derive(Debug, Clone)]
pub struct ParamMeta {
    /// Parameter name (e.g., "Cutoff", "Resonance").
    pub name: String,

    /// Minimum value.
    pub min: f32,

    /// Maximum value.
    pub max: f32,

    /// Default value.
    pub default: f32,

    /// Current value.
    pub value: f32,

    /// Unit (e.g., "Hz", "dB", "%").
    pub unit: Option<String>,

    /// Display format (e.g., "{:.2}" for 2 decimal places).
    pub format: String,
}

impl ParamMeta {
    /// Creates a new parameter metadata.
    pub fn new(name: impl Into<String>, min: f32, max: f32, default: f32) -> Self {
        Self {
            name: name.into(),
            min,
            max,
            default,
            value: default,
            unit: None,
            format: "{:.2}".to_string(),
        }
    }

    /// Sets the unit.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Sets the display format.
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Formats the value for display.
    pub fn format_value(&self, value: f32) -> String {
        let formatted = format!("{}", value);
        if let Some(ref unit) = self.unit {
            format!("{} {}", formatted, unit)
        } else {
            formatted
        }
    }

    /// Normalizes a value to 0.0-1.0 range.
    pub fn normalize(&self, value: f32) -> f32 {
        if self.max == self.min {
            0.0
        } else {
            (value - self.min) / (self.max - self.min)
        }
    }

    /// Denormalizes a 0.0-1.0 value to parameter range.
    pub fn denormalize(&self, normalized: f32) -> f32 {
        self.min + normalized * (self.max - self.min)
    }
}

/// Parameter bridge - Thread-safe parameter synchronization.
///
/// Uses arc-swap for lock-free reads from DSP thread.
pub struct ParamBridge {
    /// Parameter metadata.
    params: Vec<ParamMeta>,

    /// Current parameter values (lock-free).
    values: Arc<ArcSwap<Vec<f32>>>,

    /// Parameter name to ID mapping.
    name_to_id: HashMap<String, ParamId>,
}

impl ParamBridge {
    /// Creates a new parameter bridge.
    pub fn new() -> Self {
        Self {
            params: Vec::new(),
            values: Arc::new(ArcSwap::from_pointee(Vec::new())),
            name_to_id: HashMap::new(),
        }
    }

    /// Adds a parameter.
    ///
    /// # Arguments
    ///
    /// * `name` - Parameter name
    /// * `min` - Minimum value
    /// * `max` - Maximum value
    /// * `default` - Default value
    ///
    /// # Returns
    ///
    /// Parameter ID for use in widgets and DSP code.
    pub fn add_param(
        &mut self,
        name: impl Into<String>,
        min: f32,
        max: f32,
        default: f32,
    ) -> ParamId {
        let name = name.into();
        let id = self.params.len();

        let meta = ParamMeta::new(name.clone(), min, max, default);
        self.params.push(meta);
        self.name_to_id.insert(name, id);

        // Update values vector
        let mut values = (*self.values.load().as_ref()).clone();
        values.push(default);
        self.values.store(Arc::new(values));

        id
    }

    /// Adds a parameter with metadata.
    pub fn add_param_meta(&mut self, meta: ParamMeta) -> ParamId {
        let id = self.params.len();
        let name = meta.name.clone();
        let default = meta.default;

        self.params.push(meta);
        self.name_to_id.insert(name, id);

        // Update values vector
        let mut values = (*self.values.load().as_ref()).clone();
        values.push(default);
        self.values.store(Arc::new(values));

        id
    }

    /// Sets a parameter value (from UI thread).
    ///
    /// # Arguments
    ///
    /// * `id` - Parameter ID
    /// * `value` - New value (will be clamped to min/max)
    pub fn set_value(&mut self, id: ParamId, value: f32) {
        if id >= self.params.len() {
            return;
        }

        let meta = &mut self.params[id];
        let clamped = value.clamp(meta.min, meta.max);
        meta.value = clamped;

        // Update values vector (lock-free)
        let mut values = (*self.values.load().as_ref()).clone();
        values[id] = clamped;
        self.values.store(Arc::new(values));
    }

    /// Gets a parameter value (from DSP thread - lock-free).
    ///
    /// # Arguments
    ///
    /// * `id` - Parameter ID
    ///
    /// # Returns
    ///
    /// Current parameter value, or 0.0 if ID is invalid.
    pub fn get_value(&self, id: ParamId) -> f32 {
        let values = self.values.load();
        values.get(id).copied().unwrap_or(0.0)
    }

    /// Gets parameter metadata.
    pub fn get_meta(&self, id: ParamId) -> Option<&ParamMeta> {
        self.params.get(id)
    }

    /// Gets parameter metadata (mutable).
    pub fn get_meta_mut(&mut self, id: ParamId) -> Option<&mut ParamMeta> {
        self.params.get_mut(id)
    }

    /// Gets parameter ID by name.
    pub fn get_id(&self, name: &str) -> Option<ParamId> {
        self.name_to_id.get(name).copied()
    }

    /// Gets all parameters.
    pub fn params(&self) -> &[ParamMeta] {
        &self.params
    }

    /// Resets a parameter to default value.
    pub fn reset(&mut self, id: ParamId) {
        if let Some(meta) = self.params.get(id) {
            let default = meta.default;
            self.set_value(id, default);
        }
    }

    /// Resets all parameters to default values.
    pub fn reset_all(&mut self) {
        for id in 0..self.params.len() {
            self.reset(id);
        }
    }

    /// Gets a clone of the values Arc for sharing with DSP thread.
    pub fn values_arc(&self) -> Arc<ArcSwap<Vec<f32>>> {
        Arc::clone(&self.values)
    }
}

impl Default for ParamBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameter widget type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamWidgetType {
    /// Horizontal slider.
    Slider,
    /// Rotary knob.
    Knob,
    /// Numeric input.
    NumberInput,
    /// Toggle button.
    Toggle,
    /// Dropdown menu.
    Dropdown,
}

/// Parameter widget configuration.
#[derive(Debug, Clone)]
pub struct ParamWidget {
    /// Parameter ID.
    pub param_id: ParamId,

    /// Widget type.
    pub widget_type: ParamWidgetType,

    /// Width in pixels (optional).
    pub width: Option<f32>,

    /// Height in pixels (optional).
    pub height: Option<f32>,

    /// Show value label.
    pub show_label: bool,

    /// Show parameter name.
    pub show_name: bool,
}

impl ParamWidget {
    /// Creates a slider widget.
    pub fn slider(param_id: ParamId) -> Self {
        Self {
            param_id,
            widget_type: ParamWidgetType::Slider,
            width: Some(200.0),
            height: Some(30.0),
            show_label: true,
            show_name: true,
        }
    }

    /// Creates a knob widget.
    pub fn knob(param_id: ParamId) -> Self {
        Self {
            param_id,
            widget_type: ParamWidgetType::Knob,
            width: Some(60.0),
            height: Some(60.0),
            show_label: true,
            show_name: true,
        }
    }

    /// Creates a number input widget.
    pub fn number_input(param_id: ParamId) -> Self {
        Self {
            param_id,
            widget_type: ParamWidgetType::NumberInput,
            width: Some(100.0),
            height: Some(30.0),
            show_label: false,
            show_name: true,
        }
    }

    /// Creates a toggle widget.
    pub fn toggle(param_id: ParamId) -> Self {
        Self {
            param_id,
            widget_type: ParamWidgetType::Toggle,
            width: Some(50.0),
            height: Some(30.0),
            show_label: false,
            show_name: true,
        }
    }

    /// Sets the width.
    pub fn with_width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the height.
    pub fn with_height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets whether to show the value label.
    pub fn with_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    /// Sets whether to show the parameter name.
    pub fn with_name(mut self, show: bool) -> Self {
        self.show_name = show;
        self
    }
}

/// Plugin GUI layout.
///
/// Defines the arrangement of parameter widgets in a plugin UI.
#[derive(Debug, Clone)]
pub struct PluginLayout {
    /// Layout name.
    pub name: String,

    /// Widgets in the layout.
    pub widgets: Vec<ParamWidget>,

    /// Window width.
    pub width: u32,

    /// Window height.
    pub height: u32,
}

impl PluginLayout {
    /// Creates a new plugin layout.
    pub fn new(name: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            name: name.into(),
            widgets: Vec::new(),
            width,
            height,
        }
    }

    /// Adds a widget to the layout.
    pub fn add_widget(&mut self, widget: ParamWidget) {
        self.widgets.push(widget);
    }

    /// Adds multiple widgets to the layout.
    pub fn add_widgets(&mut self, widgets: impl IntoIterator<Item = ParamWidget>) {
        self.widgets.extend(widgets);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_meta_normalize() {
        let meta = ParamMeta::new("gain", 0.0, 1.0, 0.5);

        assert_eq!(meta.normalize(0.0), 0.0);
        assert_eq!(meta.normalize(0.5), 0.5);
        assert_eq!(meta.normalize(1.0), 1.0);
    }

    #[test]
    fn test_param_meta_denormalize() {
        let meta = ParamMeta::new("cutoff", 20.0, 20000.0, 1000.0);

        assert_eq!(meta.denormalize(0.0), 20.0);
        assert_eq!(meta.denormalize(1.0), 20000.0);
        assert!((meta.denormalize(0.5) - 10010.0).abs() < 1.0);
    }

    #[test]
    fn test_param_meta_format() {
        let meta = ParamMeta::new("gain", 0.0, 1.0, 0.5).with_unit("dB");

        let formatted = meta.format_value(0.75);
        assert!(formatted.contains("0.75"));
        assert!(formatted.contains("dB"));
    }

    #[test]
    fn test_param_bridge_basic() {
        let mut bridge = ParamBridge::new();

        let gain_id = bridge.add_param("gain", 0.0, 1.0, 0.5);
        assert_eq!(bridge.get_value(gain_id), 0.5);

        bridge.set_value(gain_id, 0.75);
        assert_eq!(bridge.get_value(gain_id), 0.75);
    }

    #[test]
    fn test_param_bridge_clamping() {
        let mut bridge = ParamBridge::new();

        let gain_id = bridge.add_param("gain", 0.0, 1.0, 0.5);

        // Should clamp to max
        bridge.set_value(gain_id, 2.0);
        assert_eq!(bridge.get_value(gain_id), 1.0);

        // Should clamp to min
        bridge.set_value(gain_id, -1.0);
        assert_eq!(bridge.get_value(gain_id), 0.0);
    }

    #[test]
    fn test_param_bridge_reset() {
        let mut bridge = ParamBridge::new();

        let gain_id = bridge.add_param("gain", 0.0, 1.0, 0.5);
        bridge.set_value(gain_id, 0.75);

        bridge.reset(gain_id);
        assert_eq!(bridge.get_value(gain_id), 0.5);
    }

    #[test]
    fn test_param_bridge_get_by_name() {
        let mut bridge = ParamBridge::new();

        let gain_id = bridge.add_param("gain", 0.0, 1.0, 0.5);

        let found_id = bridge.get_id("gain");
        assert_eq!(found_id, Some(gain_id));

        let not_found = bridge.get_id("nonexistent");
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_param_widget_creation() {
        let slider = ParamWidget::slider(0);
        assert_eq!(slider.widget_type, ParamWidgetType::Slider);
        assert!(slider.show_label);

        let knob = ParamWidget::knob(1);
        assert_eq!(knob.widget_type, ParamWidgetType::Knob);

        let toggle = ParamWidget::toggle(2);
        assert_eq!(toggle.widget_type, ParamWidgetType::Toggle);
    }

    #[test]
    fn test_plugin_layout() {
        let mut layout = PluginLayout::new("My Plugin", 800, 600);

        layout.add_widget(ParamWidget::slider(0));
        layout.add_widget(ParamWidget::knob(1));

        assert_eq!(layout.widgets.len(), 2);
        assert_eq!(layout.width, 800);
        assert_eq!(layout.height, 600);
    }

    #[test]
    fn test_param_bridge_thread_safety() {
        let mut bridge = ParamBridge::new();
        let gain_id = bridge.add_param("gain", 0.0, 1.0, 0.5);

        // Get Arc for DSP thread
        let values_arc = bridge.values_arc();

        // Simulate UI thread update
        bridge.set_value(gain_id, 0.75);

        // Simulate DSP thread read (lock-free)
        let values = values_arc.load();
        assert_eq!(values[gain_id], 0.75);
    }
}
