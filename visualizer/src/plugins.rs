use std::sync::Arc;
use walkers::{Plugin, Position, Projector};

/// The visual shape of an object on the map.
pub enum GeoShape {
    /// Axis-aligned bounding box (e.g. a car or truck).
    Rect {
        top_left: Position,
        bottom_right: Position,
    },
    /// Circular marker (e.g. a pedestrian or cyclist).
    #[expect(dead_code, reason = "Support for other shapes not added yet.")]
    Circle {
        center: Position,
        radius_px: f32, // screen-space radius, fixed size regardless of zoom
    },
}

/// A single object to be rendered on the map.
pub struct MapObject {
    pub shape: GeoShape,
    pub stroke: egui::Stroke,
    pub fill: egui::Color32,
}

/// walkers Plugin that renders a shared slice of MapObjects.
pub struct ObjectsPlugin {
    pub objects: Arc<Vec<MapObject>>,
}

impl Plugin for ObjectsPlugin {
    fn run(
        self: Box<Self>,
        ui: &mut egui::Ui,
        _response: &egui::Response,
        projector: &Projector,
        _map_memory: &walkers::MapMemory,
    ) {
        let painter = ui.painter();
        for obj in self.objects.iter() {
            match &obj.shape {
                GeoShape::Rect {
                    top_left,
                    bottom_right,
                } => {
                    let tl = projector.project(*top_left).to_pos2();
                    let br = projector.project(*bottom_right).to_pos2();
                    let rect = egui::Rect::from_two_pos(tl, br);
                    painter.rect(rect, 0.0, obj.fill, obj.stroke, egui::StrokeKind::Outside);
                }
                GeoShape::Circle { center, radius_px } => {
                    let c = projector.project(*center).to_pos2();
                    painter.circle(c, *radius_px, obj.fill, obj.stroke);
                }
            }
        }
    }
}
