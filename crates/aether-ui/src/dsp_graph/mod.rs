// DSP Graph Mode - Visual node editor + code editor

pub mod canvas;
pub mod node_editor;
pub mod code_editor;
pub mod node_library;
pub mod inspector;

pub use canvas::GraphCanvas;
pub use node_library::NodeLibrary;
pub use inspector::Inspector;

use iced::{Point, Size};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the DSP graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub node_type: NodeType,
    #[serde(with = "point_serde")]
    pub position: Point,
    #[serde(with = "size_serde")]
    pub size: Size,
    pub parameters: HashMap<String, f32>,
    pub inputs: Vec<PortId>,
    pub outputs: Vec<PortId>,
}

// Serde helpers for iced types
mod point_serde {
    use iced::Point;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(point: &Point, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (point.x, point.y).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Point, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (x, y) = <(f32, f32)>::deserialize(deserializer)?;
        Ok(Point::new(x, y))
    }
}

mod size_serde {
    use iced::Size;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(size: &Size, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (size.width, size.height).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Size, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (width, height) = <(f32, f32)>::deserialize(deserializer)?;
        Ok(Size::new(width, height))
    }
}

/// Unique identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

/// Unique identifier for a port (input/output)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub u64);

/// Connection between two ports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: ConnectionId,
    pub from_node: NodeId,
    pub from_port: PortId,
    pub to_node: NodeId,
    pub to_port: PortId,
    pub cable_type: CableType,
}

/// Unique identifier for a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub u64);

/// Type of cable/connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CableType {
    Audio,
    Control,
    Midi,
    Modulation,
}

/// Type of DSP node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // Audio I/O
    AudioInput,
    AudioOutput,
    MidiInput,
    MidiOutput,
    
    // Generators
    Oscillator { waveform: Waveform },
    NoiseGenerator,
    SamplePlayer,
    Wavetable,
    
    // Filters
    LowPass,
    HighPass,
    BandPass,
    Notch,
    AllPass,
    StateVariable,
    MoogLadder,
    
    // Dynamics
    Compressor,
    Limiter,
    Gate,
    Expander,
    
    // Time-based
    Delay,
    Reverb,
    Chorus,
    Flanger,
    Phaser,
    
    // Distortion
    Waveshaper,
    Saturation,
    BitCrusher,
    
    // Utilities
    Gain,
    Mixer,
    Pan,
    Scope,
    Analyzer,
    
    // Modulators
    #[allow(clippy::upper_case_acronyms)]
    LFO,
    Envelope,
    
    // Custom (user-defined)
    Custom { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
    Noise,
}

impl NodeType {
    pub fn name(&self) -> &str {
        match self {
            NodeType::AudioInput => "Audio Input",
            NodeType::AudioOutput => "Audio Output",
            NodeType::MidiInput => "MIDI Input",
            NodeType::MidiOutput => "MIDI Output",
            NodeType::Oscillator { .. } => "Oscillator",
            NodeType::NoiseGenerator => "Noise Generator",
            NodeType::SamplePlayer => "Sample Player",
            NodeType::Wavetable => "Wavetable",
            NodeType::LowPass => "Low Pass Filter",
            NodeType::HighPass => "High Pass Filter",
            NodeType::BandPass => "Band Pass Filter",
            NodeType::Notch => "Notch Filter",
            NodeType::AllPass => "All Pass Filter",
            NodeType::StateVariable => "State Variable Filter",
            NodeType::MoogLadder => "Moog Ladder Filter",
            NodeType::Compressor => "Compressor",
            NodeType::Limiter => "Limiter",
            NodeType::Gate => "Gate",
            NodeType::Expander => "Expander",
            NodeType::Delay => "Delay",
            NodeType::Reverb => "Reverb",
            NodeType::Chorus => "Chorus",
            NodeType::Flanger => "Flanger",
            NodeType::Phaser => "Phaser",
            NodeType::Waveshaper => "Waveshaper",
            NodeType::Saturation => "Saturation",
            NodeType::BitCrusher => "Bit Crusher",
            NodeType::Gain => "Gain",
            NodeType::Mixer => "Mixer",
            NodeType::Pan => "Pan",
            NodeType::Scope => "Scope",
            NodeType::Analyzer => "Analyzer",
            NodeType::LFO => "LFO",
            NodeType::Envelope => "Envelope",
            NodeType::Custom { name } => name,
        }
    }

    pub fn category(&self) -> NodeCategory {
        match self {
            NodeType::AudioInput | NodeType::AudioOutput | NodeType::MidiInput | NodeType::MidiOutput => NodeCategory::AudioIO,
            NodeType::Oscillator { .. } | NodeType::NoiseGenerator | NodeType::SamplePlayer | NodeType::Wavetable => NodeCategory::Generator,
            NodeType::LowPass | NodeType::HighPass | NodeType::BandPass | NodeType::Notch | NodeType::AllPass | NodeType::StateVariable | NodeType::MoogLadder => NodeCategory::Filter,
            NodeType::Compressor | NodeType::Limiter | NodeType::Gate | NodeType::Expander => NodeCategory::Dynamics,
            NodeType::Delay | NodeType::Reverb | NodeType::Chorus | NodeType::Flanger | NodeType::Phaser => NodeCategory::TimeBased,
            NodeType::Waveshaper | NodeType::Saturation | NodeType::BitCrusher => NodeCategory::Distortion,
            NodeType::Gain | NodeType::Mixer | NodeType::Pan | NodeType::Scope | NodeType::Analyzer => NodeCategory::Utility,
            NodeType::LFO | NodeType::Envelope => NodeCategory::Modulator,
            NodeType::Custom { .. } => NodeCategory::Custom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeCategory {
    AudioIO,
    Generator,
    Filter,
    Dynamics,
    TimeBased,
    Distortion,
    Utility,
    Modulator,
    Custom,
}

impl NodeCategory {
    pub fn name(&self) -> &str {
        match self {
            NodeCategory::AudioIO => "Audio I/O",
            NodeCategory::Generator => "Generators",
            NodeCategory::Filter => "Filters",
            NodeCategory::Dynamics => "Dynamics",
            NodeCategory::TimeBased => "Time-Based",
            NodeCategory::Distortion => "Distortion",
            NodeCategory::Utility => "Utilities",
            NodeCategory::Modulator => "Modulators",
            NodeCategory::Custom => "Custom",
        }
    }
}

/// The state of the DSP graph editor
#[derive(Debug, Clone)]
pub struct DspGraphState {
    pub nodes: HashMap<NodeId, GraphNode>,
    pub connections: HashMap<ConnectionId, Connection>,
    pub selected_node: Option<NodeId>,
    pub next_node_id: u64,
    pub next_port_id: u64,
    pub next_connection_id: u64,
    pub canvas_offset: Point,
    pub canvas_zoom: f32,
}

impl Default for DspGraphState {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: HashMap::new(),
            selected_node: None,
            next_node_id: 0,
            next_port_id: 0,
            next_connection_id: 0,
            canvas_offset: Point::ORIGIN,
            canvas_zoom: 1.0,
        }
    }
}

impl DspGraphState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node_type: NodeType, position: Point) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        let node = GraphNode {
            id,
            node_type,
            position,
            size: Size::new(120.0, 80.0),
            parameters: HashMap::new(),
            inputs: vec![],
            outputs: vec![],
        };

        self.nodes.insert(id, node);
        id
    }

    pub fn remove_node(&mut self, id: NodeId) {
        self.nodes.remove(&id);
        // Remove all connections to/from this node
        self.connections.retain(|_, conn| {
            conn.from_node != id && conn.to_node != id
        });
    }

    pub fn add_connection(&mut self, from_node: NodeId, from_port: PortId, to_node: NodeId, to_port: PortId, cable_type: CableType) -> ConnectionId {
        let id = ConnectionId(self.next_connection_id);
        self.next_connection_id += 1;

        let connection = Connection {
            id,
            from_node,
            from_port,
            to_node,
            to_port,
            cable_type,
        };

        self.connections.insert(id, connection);
        id
    }

    pub fn remove_connection(&mut self, id: ConnectionId) {
        self.connections.remove(&id);
    }
}
