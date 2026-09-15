use super::geometry::Geometry;
use super::snapshot::SemanticSnapshot;

pub fn export_dxf(snapshot: &SemanticSnapshot) -> String {
    let s = snapshot.clone().deterministic();
    let mut out = String::from("0\nSECTION\n2\nENTITIES\n");
    for item in s.geometry {
        match item.geometry {
            Geometry::Line(l) => {
                out.push_str(&format!(
                    "0\nLINE\n8\n0\n10\n{:.15}\n20\n{:.15}\n11\n{:.15}\n21\n{:.15}\n",
                    l.start.x, l.start.y, l.end.x, l.end.y
                ));
            }
            Geometry::Circle(c) => {
                out.push_str(&format!(
                    "0\nCIRCLE\n8\n0\n10\n{:.15}\n20\n{:.15}\n40\n{:.15}\n",
                    c.center.x, c.center.y, c.radius
                ));
            }
            Geometry::Arc(a) => {
                out.push_str(&format!(
                    "0\nARC\n8\n0\n10\n{:.15}\n20\n{:.15}\n40\n{:.15}\n50\n{:.15}\n51\n{:.15}\n",
                    a.center.x,
                    a.center.y,
                    a.radius,
                    a.start_angle.to_degrees(),
                    a.end_angle.to_degrees()
                ));
            }
        }
    }
    out.push_str("0\nENDSEC\n0\nEOF\n");
    out
}
