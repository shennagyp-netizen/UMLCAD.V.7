from pathlib import Path

# Controlled baseline-repair helper; removed from the final authoritative tree.
MATH_DIR = Path("kernel/math")
FILES = [p for p in MATH_DIR.glob("*.rs")]


def normalize_leading_float_literals(text: str) -> str:
    out = []
    i = 0
    n = len(text)
    block_depth = 0
    state = "normal"
    raw_hashes = 0
    last_word = ""

    def prev_nonspace():
        j = len(out) - 1
        while j >= 0 and out[j].isspace():
            j -= 1
        return out[j] if j >= 0 else ""

    expr_keywords = {"return", "yield", "else"}

    while i < n:
        c = text[i]
        nxt = text[i + 1] if i + 1 < n else ""

        if state == "line_comment":
            out.append(c); i += 1
            if c == "\n": state = "normal"
            continue
        if state == "block_comment":
            if c == "/" and nxt == "*":
                out.extend((c, nxt)); block_depth += 1; i += 2
            elif c == "*" and nxt == "/":
                out.extend((c, nxt)); block_depth -= 1; i += 2
                if block_depth == 0: state = "normal"
            else:
                out.append(c); i += 1
            continue
        if state == "string":
            out.append(c); i += 1
            if c == "\\" and i < n:
                out.append(text[i]); i += 1
            elif c == '"': state = "normal"
            continue
        if state == "char":
            out.append(c); i += 1
            if c == "\\" and i < n:
                out.append(text[i]); i += 1
            elif c == "'": state = "normal"
            continue

        if c == "r" and (nxt == '"' or nxt == "#"):
            j = i + 1
            while j < n and text[j] == "#": j += 1
            if j < n and text[j] == '"':
                raw_hashes = j - (i + 1)
                out.append(text[i:j + 1]); i = j + 1
                end = '"' + ('#' * raw_hashes)
                k = text.find(end, i)
                if k == -1:
                    out.append(text[i:]); break
                out.append(text[i:k + len(end)]); i = k + len(end)
                continue
        if c == "/" and nxt == "/":
            out.extend((c, nxt)); i += 2; state = "line_comment"; continue
        if c == "/" and nxt == "*":
            out.extend((c, nxt)); i += 2; block_depth = 1; state = "block_comment"; continue
        if c == '"':
            out.append(c); i += 1; state = "string"; continue
        if c == "'":
            if i + 2 < n and text[i + 2] == "'":
                out.append(c); i += 1; state = "char"; continue
            out.append(c); i += 1; continue

        if c.isalpha() or c == "_":
            j = i + 1
            while j < n and (text[j].isalnum() or text[j] == "_"): j += 1
            word = text[i:j]; out.append(word); last_word = word; i = j; continue

        if c == "." and i + 1 < n and text[i + 1].isdigit():
            p = prev_nonspace()
            can_start = p == "" or p in "([{,:=+\-*/%!?&|<>;"
            if not can_start and last_word in expr_keywords: can_start = True
            if can_start:
                out.append("0."); i += 1
                while i < n and text[i].isdigit(): out.append(text[i]); i += 1
                if i < n and text[i] in "eE":
                    out.append(text[i]); i += 1
                    if i < n and text[i] in "+-": out.append(text[i]); i += 1
                    while i < n and text[i].isdigit(): out.append(text[i]); i += 1
                continue
        out.append(c); i += 1
    return "".join(out)


for path in FILES:
    original = path.read_text()
    normalized = normalize_leading_float_literals(original)
    if normalized != original: path.write_text(normalized)

mod = MATH_DIR / "mod.rs"
t = mod.read_text()
if "pub mod dimensions;" not in t:
    t = t.replace("pub mod diagnostics;\n", "pub mod diagnostics;\npub mod dimensions;\n")
if "pub mod solver;" not in t:
    t = t.replace("pub mod spatial_accel;\n", "pub mod spatial_accel;\npub mod solver;\n")
mod.write_text(t)

lib = Path("kernel/native/src/lib.rs")
t = lib.read_text()
if "pub use functions as math;" not in t:
    t = t.replace("pub mod services;\n", "pub mod services;\npub use functions as math;\n")
lib.write_text(t)

geom = MATH_DIR / "geometry.rs"
t = geom.read_text()
needle = "    pub fn norm(self) -> f64 {\n        self.x.hypot(self.y)\n    }\n"
addition = needle + "    pub fn length(self) -> f64 {\n        self.norm()\n    }\n    pub fn is_finite(self) -> bool {\n        self.x.is_finite() && self.y.is_finite()\n    }\n    pub fn cross(self, other: Self) -> f64 {\n        self.x * other.y - self.y * other.x\n    }\n"
if "pub fn cross(self, other: Self)" not in t and needle in t:
    t = t.replace(needle, addition, 1)
geom.write_text(t)

inter = MATH_DIR / "intersections.rs"
t = inter.read_text()
t = t.replace("    let offset = line.origin.sub(circle.center);\n", "    let circle_center = Vec2::new(circle.center.x, circle.center.y);\n    let offset = line.origin.sub(circle_center);\n", 1)
t = t.replace("        let base = a.center.add(delta.scale(x / distance));", "        let center = Vec2::new(a.center.x, a.center.y);\n        let delta_vec = Vec2::new(delta.x, delta.y);\n        let base = center.add(delta_vec.scale(x / distance));", 1)
t = t.replace("    let base = a.center.add(delta.scale(x / distance));\n    let unit_perp = Vec2::new(-delta.y / distance, delta.x / distance);", "    let a_center = Vec2::new(a.center.x, a.center.y);\n    let delta_vec = Vec2::new(delta.x, delta.y);\n    let base = a_center.add(delta_vec.scale(x / distance));\n    let unit_perp = Vec2::new(-delta.y / distance, delta.x / distance);", 1)
t = t.replace("            center: Vec2::new(0.0, 0.0),\n            radius: 1.0,", "            center: Point { x: 0.0, y: 0.0 },\n            radius: 1.0,", 1)
inter.write_text(t)

tol = MATH_DIR / "tolerance.rs"
t = tol.read_text().replace("tolerance.threshold(2.0) - 1.0002e-6", "tolerance.threshold(2.0).unwrap() - 1.0002e-6")
tol.write_text(t)

mat = MATH_DIR / "mat.rs"
t = mat.read_text()
t = t.replace("let a = std::array::from_fn(|i| std::array::from_fn(|j| self.m[i][j] / scale));", "let a: [[f64; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| self.m[i][j] / scale));")
t = t.replace("let result = std::array::from_fn(|i| (0..3).map(|j| inv.m[i][j] * b[j]).sum());", "let result: [f64; 3] = std::array::from_fn(|i| (0..3).map(|j| inv.m[i][j] * b[j]).sum());")
t = t.replace("let result = std::array::from_fn(|i| (0..4).map(|j| inverse.m[i][j] * b[j]).sum());", "let result: [f64; 4] = std::array::from_fn(|i| (0..4).map(|j| inverse.m[i][j] * b[j]).sum());")
mat.write_text(t)

lin = MATH_DIR / "linalg.rs"
t = lin.read_text().replace("assert_eq!(svd(&bad, RTOL, ICT), Err(LinAlgError::NonFinite));", "assert!(matches!(svd(&bad, RTOL, ICT), Err(LinAlgError::NonFinite)));")
lin.write_text(t)

bvh = MATH_DIR / "spatial_accel.rs"
t = bvh.read_text()
t = t.replace("            match self.nodes[index] {\n                BvhNode::Leaf { item } => {", "            match &self.nodes[index] {\n                BvhNode::Leaf { item } => {")
t = t.replace("            if let BvhNode::Leaf { item: a } = self.nodes[i] {", "            if let BvhNode::Leaf { item: a } = &self.nodes[i] {")
t = t.replace("                    if let BvhNode::Leaf { item: b } = self.nodes[j] {", "                    if let BvhNode::Leaf { item: b } = &self.nodes[j] {")
t = t.replace("                        stack.push(right);", "                        stack.push(*right);")
t = t.replace("                        stack.push(left);", "                        stack.push(*left);")
bvh.write_text(t)
