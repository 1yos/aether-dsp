// Graph canvas for visual node editing

use super::{CableType, Connection, ConnectionId, DspGraphState, GraphNode, NodeId};
use crate::theme::AetherTheme;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::{mouse, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

#[derive(Debug)]
pub struct GraphCanvas {
    state: DspGraphState,
    cache: canvas::Cache,
}

#[derive(Debug, Clone)]
pub enum CanvasMessage {
    NodeClicked(NodeId),
    NodeDragged(NodeId, Vector),
    ConnectionClicked(ConnectionId),
    CanvasPanned(Vector),
    CanvasZoomed(f32),
    AddNode(Point),
}

impl GraphCanvas {
    pub fn new() -> Self {
        Self {
            state: DspGraphState::new(),
            cache: canvas::Cache::new(),
        }
    }

    pub fn view(&self) -> Element<'_, CanvasMessage> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn state(&self) -> &DspGraphState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut DspGraphState {
        self.cache.clear();
        &mut self.state
    }
}

impl Default for GraphCanvas {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message> canvas::Program<Message> for GraphCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Draw background
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), AetherTheme::CANVAS_BACKGROUND);

            // Draw grid
            self.draw_grid(frame, bounds.size());

            // Draw connections
            for connection in self.state.connections.values() {
                self.draw_connection(frame, connection);
            }

            // Draw nodes
            for node in self.state.nodes.values() {
                self.draw_node(frame, node);
            }
        });

        vec![geometry]
    }
}

impl GraphCanvas {
    fn draw_grid(&self, frame: &mut Frame, size: Size) {
        let grid_size = 20.0 * self.state.canvas_zoom;
        let offset = self.state.canvas_offset;

        let start_x = (offset.x % grid_size) - grid_size;
        let start_y = (offset.y % grid_size) - grid_size;

        // Vertical lines
        let mut x = start_x;
        while x < size.width {
            let path = Path::line(Point::new(x, 0.0), Point::new(x, size.height));
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))
                    .with_width(1.0),
            );
            x += grid_size;
        }

        // Horizontal lines
        let mut y = start_y;
        while y < size.height {
            let path = Path::line(Point::new(0.0, y), Point::new(size.width, y));
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))
                    .with_width(1.0),
            );
            y += grid_size;
        }
    }

    fn draw_node(&self, frame: &mut Frame, node: &GraphNode) {
        let position = Point::new(
            node.position.x + self.state.canvas_offset.x,
            node.position.y + self.state.canvas_offset.y,
        );

        // Node background
        let is_selected = self.state.selected_node == Some(node.id);
        let bg_color = if is_selected {
            AetherTheme::ACTIVE_STATE
        } else {
            AetherTheme::NODE_BACKGROUND
        };

        frame.fill_rectangle(position, node.size, bg_color);

        // Node border
        let border_color = if is_selected {
            AetherTheme::PRIMARY
        } else {
            self.get_node_category_color(&node.node_type)
        };

        let border_path = Path::rectangle(position, node.size);
        frame.stroke(
            &border_path,
            Stroke::default()
                .with_color(border_color)
                .with_width(if is_selected { 2.0 } else { 1.0 }),
        );

        // Node title
        let title = Text {
            content: node.node_type.name().to_string(),
            position: Point::new(position.x + 8.0, position.y + 8.0),
            color: AetherTheme::TEXT_PRIMARY,
            size: 12.0.into(),
            ..Default::default()
        };
        frame.fill_text(title);

        // Input ports (left side)
        for (i, _port) in node.inputs.iter().enumerate() {
            let port_y = position.y + 30.0 + (i as f32 * 15.0);
            let port_center = Point::new(position.x, port_y);
            self.draw_port(frame, port_center, false);
        }

        // Output ports (right side)
        for (i, _port) in node.outputs.iter().enumerate() {
            let port_y = position.y + 30.0 + (i as f32 * 15.0);
            let port_center = Point::new(position.x + node.size.width, port_y);
            self.draw_port(frame, port_center, true);
        }
    }

    fn draw_port(&self, frame: &mut Frame, center: Point, _is_output: bool) {
        let radius = 4.0;
        let port_path = Path::circle(center, radius);
        frame.fill(&port_path, AetherTheme::TEXT_SECONDARY);
        frame.stroke(
            &port_path,
            Stroke::default()
                .with_color(AetherTheme::TEXT_PRIMARY)
                .with_width(1.0),
        );
    }

    fn draw_connection(&self, frame: &mut Frame, connection: &Connection) {
        // Find the nodes
        let from_node = match self.state.nodes.get(&connection.from_node) {
            Some(n) => n,
            None => return,
        };
        let to_node = match self.state.nodes.get(&connection.to_node) {
            Some(n) => n,
            None => return,
        };

        // Calculate connection points
        let from_point = Point::new(
            from_node.position.x + from_node.size.width + self.state.canvas_offset.x,
            from_node.position.y + 30.0 + self.state.canvas_offset.y,
        );
        let to_point = Point::new(
            to_node.position.x + self.state.canvas_offset.x,
            to_node.position.y + 30.0 + self.state.canvas_offset.y,
        );

        // Draw bezier curve
        let control_offset = (to_point.x - from_point.x).abs() * 0.5;
        let control1 = Point::new(from_point.x + control_offset, from_point.y);
        let control2 = Point::new(to_point.x - control_offset, to_point.y);

        let path = Path::new(|builder| {
            builder.move_to(from_point);
            builder.bezier_curve_to(control1, control2, to_point);
        });

        let cable_color = self.get_cable_color(connection.cable_type);
        frame.stroke(
            &path,
            Stroke::default().with_color(cable_color).with_width(2.0),
        );
    }

    fn get_node_category_color(&self, node_type: &super::NodeType) -> Color {
        use super::NodeCategory;
        match node_type.category() {
            NodeCategory::AudioIO => AetherTheme::NODE_AUDIO_IO,
            NodeCategory::Generator => AetherTheme::NODE_GENERATOR,
            NodeCategory::Filter => AetherTheme::NODE_EFFECT,
            NodeCategory::Dynamics => AetherTheme::NODE_EFFECT,
            NodeCategory::TimeBased => AetherTheme::NODE_EFFECT,
            NodeCategory::Distortion => AetherTheme::NODE_EFFECT,
            NodeCategory::Utility => AetherTheme::NODE_UTILITY,
            NodeCategory::Modulator => AetherTheme::NODE_MODULATOR,
            NodeCategory::Custom => AetherTheme::NODE_PARAMETER,
        }
    }

    fn get_cable_color(&self, cable_type: CableType) -> Color {
        match cable_type {
            CableType::Audio => AetherTheme::CABLE_AUDIO,
            CableType::Control => AetherTheme::CABLE_CONTROL,
            CableType::Midi => AetherTheme::CABLE_MIDI,
            CableType::Modulation => AetherTheme::CABLE_MODULATION,
        }
    }
}
