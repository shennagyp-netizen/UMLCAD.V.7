import { Canvas, useThree } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { useEffect, useMemo, useRef, useState, type MutableRefObject } from 'react';
import * as THREE from 'three';
import type { CompiledModelPackage, CompiledNode, JsonValue } from './types';
import { ModelIndex } from './model-index';
import { validateCompiledModelPackage } from './validate-package';

type GeometryItem = {
  id: string;
  kind: string;
  properties: Record<string, string>;
};

type BuildPart = {
  id: string;
  name: string;
  geometry: GeometryItem[];
};

type BuildPackage = {
  schema: string;
  applicationId: string;
  applicationVersion: string;
  buildIdentity: string;
  semantic: {
    parts: BuildPart[];
  };
};

type Point2 = [number, number];

type SolidEntry = {
  part: BuildPart;
  id: string;
  nodeId: string;
  kind: 'profile' | 'circle';
  points?: Point2[];
  center?: Point2;
  radius?: number;
  depth: number;
};

type EdgeEntry = {
  part: BuildPart;
  item: GeometryItem;
  nodeId: string;
  points: Point2[];
};

const GEOMETRY_EPSILON = 1e-6;

function encodeSegment(value: string): string {
  return value
    .replaceAll('%', '%25')
    .replaceAll(' ', '%20')
    .replaceAll('/', '%2F')
    .replaceAll(':', '%3A');
}

function parsePoint(value: string | undefined): Point2 | null {
  if (!value) return null;
  const [x, y] = value.split(',').map(Number);
  return Number.isFinite(x) && Number.isFinite(y) ? [x, y] : null;
}

function number(value: string | undefined): number | null {
  const result = Number(value);
  return Number.isFinite(result) ? result : null;
}

function parseLine(item: GeometryItem): [Point2, Point2] | null {
  if (item.kind !== 'line') return null;
  const start = parsePoint(item.properties.start);
  const end = parsePoint(item.properties.end);
  if (!start || !end) return null;
  if (distance2(start, end) <= GEOMETRY_EPSILON ** 2) return null;
  return [start, end];
}

function distance2(a: Point2, b: Point2): number {
  const dx = a[0] - b[0];
  const dy = a[1] - b[1];
  return Math.hypot(dx, dy);
}

function samePoint(a: Point2, b: Point2): boolean {
  return distance2(a, b) <= GEOMETRY_EPSILON;
}

function circleData(item: GeometryItem): { center: Point2; radius: number } | null {
  if (item.kind !== 'circle') return null;
  const center = parsePoint(item.properties.center);
  const radius = number(item.properties.radius);
  return center && radius !== null && radius > 0 ? { center, radius } : null;
}

function partDepth(part: BuildPart): number {
  if (part.id.includes('base')) return 18;
  if (part.id.includes('jaw')) return 35;
  if (part.id.includes('screw')) return 20;
  if (part.id.includes('handle')) return 10;
  return 15;
}

function closeLineLoops(lines: Array<{ item: GeometryItem; points: [Point2, Point2] }>): Array<{ id: string; points: Point2[]; sourceIds: string[] }> {
  const remaining = [...lines];
  const loops: Array<{ id: string; points: Point2[]; sourceIds: string[] }> = [];

  while (remaining.length > 0) {
    const first = remaining.shift()!;
    const points: Point2[] = [first.points[0], first.points[1]];
    const sourceIds = [first.item.id];
    let closed = false;

    while (remaining.length > 0) {
      const current = points[points.length - 1];
      if (samePoint(current, points[0])) {
        closed = true;
        break;
      }

      const index = remaining.findIndex(candidate =>
        samePoint(candidate.points[0], current) || samePoint(candidate.points[1], current));
      if (index < 0) break;

      const next = remaining.splice(index, 1)[0];
      const oriented: [Point2, Point2] = samePoint(next.points[0], current)
        ? next.points
        : [next.points[1], next.points[0]];
      points.push(oriented[1]);
      sourceIds.push(next.item.id);
    }

    if (closed && points.length >= 4) {
      points.pop();
      loops.push({ id: sourceIds[0], points, sourceIds });
    }
  }

  return loops;
}

function polygonArea(points: Point2[]): number {
  let area = 0;
  for (let i = 0; i < points.length; i++) {
    const next = points[(i + 1) % points.length];
    area += points[i][0] * next[1] - next[0] * points[i][1];
  }
  return area / 2;
}

function build3DEntries(build: BuildPackage): { solids: SolidEntry[]; edges: EdgeEntry[]; bounds: [number, number, number, number, number, number] | null } {
  const solids: SolidEntry[] = [];
  const edges: EdgeEntry[] = [];
  let minX = Infinity;
  let minY = Infinity;
  let minZ = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;
  let maxZ = -Infinity;

  const includePoint = (point: Point2, zMin: number, zMax: number) => {
    minX = Math.min(minX, point[0]);
    maxX = Math.max(maxX, point[0]);
    minY = Math.min(minY, point[1]);
    maxY = Math.max(maxY, point[1]);
    minZ = Math.min(minZ, zMin);
    maxZ = Math.max(maxZ, zMax);
  };

  for (const part of build.semantic.parts) {
    const depth = partDepth(part);
    const lineEntries = part.geometry
      .map(item => {
        const points = parseLine(item);
        return points ? { item, points } : null;
      })
      .filter((entry): entry is { item: GeometryItem; points: [Point2, Point2] } => entry !== null);

    const loops = closeLineLoops(lineEntries);
    const loopSourceIds = new Set(loops.flatMap(loop => loop.sourceIds));

    for (const loop of loops) {
      const id = loop.id;
      const points = polygonArea(loop.points) < 0 ? [...loop.points].reverse() : loop.points;
      solids.push({
        part,
        id,
        nodeId: `definition:part:${encodeSegment(part.id)}/geometry:${encodeSegment(id)}`,
        kind: 'profile',
        points,
        depth,
      });
      points.forEach(point => includePoint(point, 0, depth));
    }

    for (const item of part.geometry) {
      const circle = circleData(item);
      if (circle) {
        solids.push({
          part,
          id: item.id,
          nodeId: `definition:part:${encodeSegment(part.id)}/geometry:${encodeSegment(item.id)}`,
          kind: 'circle',
          center: circle.center,
          radius: circle.radius,
          depth: Math.max(depth, 12),
        });
        includePoint([circle.center[0] - circle.radius, circle.center[1] - circle.radius], 0, depth);
        includePoint([circle.center[0] + circle.radius, circle.center[1] + circle.radius], 0, depth);
      }
    }

    for (const line of lineEntries) {
      if (loopSourceIds.has(line.item.id)) continue;
      edges.push({
        part,
        item: line.item,
        nodeId: `definition:part:${encodeSegment(part.id)}/geometry:${encodeSegment(line.item.id)}`,
        points: line.points,
      });
      includePoint(line.points[0], 0, 0);
      includePoint(line.points[1], 0, 0);
    }
  }

  if (!Number.isFinite(minX)) return { solids, edges, bounds: null };
  return { solids, edges, bounds: [minX, minY, minZ, maxX, maxY, maxZ] };
}

function SolidObject({ entry, selected, hidden, onSelect }: { entry: SolidEntry; selected: boolean; hidden: boolean; onSelect: (id: string) => void }) {
  const object = useMemo(() => {
    const group = new THREE.Group();
    group.userData.umlCadId = entry.nodeId;

    if (entry.kind === 'circle' && entry.center && entry.radius) {
      const geometry = new THREE.CylinderGeometry(entry.radius, entry.radius, entry.depth, 64);
      geometry.rotateX(Math.PI / 2);
      geometry.translate(entry.center[0], entry.center[1], entry.depth / 2);
      const material = new THREE.MeshStandardMaterial({ metalness: 0.15, roughness: 0.7, color: 0x7b8794 });
      group.add(new THREE.Mesh(geometry, material));
    } else if (entry.points && entry.points.length >= 3) {
      const shape = new THREE.Shape();
      shape.moveTo(entry.points[0][0], entry.points[0][1]);
      for (const point of entry.points.slice(1)) shape.lineTo(point[0], point[1]);
      shape.closePath();
      const geometry = new THREE.ExtrudeGeometry(shape, { depth: entry.depth, bevelEnabled: false });
      const material = new THREE.MeshStandardMaterial({ metalness: 0.15, roughness: 0.7, color: 0x7b8794 });
      group.add(new THREE.Mesh(geometry, material));
      const edges = new THREE.LineSegments(
        new THREE.EdgesGeometry(geometry),
        new THREE.LineBasicMaterial({ color: 0x26313d }),
      );
      group.add(edges);
    }

    return group;
  }, [entry]);

  useEffect(() => {
    object.visible = !hidden;
    object.traverse(child => {
      const mesh = child as THREE.Mesh;
      if (!mesh.isMesh) return;
      const material = mesh.material as THREE.MeshStandardMaterial;
      if (material?.color) material.color.setHex(selected ? 0x4ea1ff : 0x7b8794);
    });
  }, [hidden, object, selected]);

  return <primitive object={object} onPointerDown={(event: any) => { event.stopPropagation(); onSelect(entry.nodeId); }} />;
}

function EdgeObject({ entry, selected, hidden, onSelect }: { entry: EdgeEntry; selected: boolean; hidden: boolean; onSelect: (id: string) => void }) {
  const object = useMemo(() => {
    const geometry = new THREE.BufferGeometry().setFromPoints(entry.points.map(point => new THREE.Vector3(point[0], point[1], 0)));
    const line = new THREE.Line(geometry, new THREE.LineBasicMaterial({ color: selected ? 0x4ea1ff : 0xd8dee9 }));
    line.userData.umlCadId = entry.nodeId;
    return line;
  }, [entry, selected]);

  useEffect(() => {
    object.visible = !hidden;
  }, [hidden, object]);

  return <primitive object={object} onPointerDown={(event: any) => { event.stopPropagation(); onSelect(entry.nodeId); }} />;
}

function CameraFit({ bounds, controlsRef }: { bounds: [number, number, number, number, number, number] | null; controlsRef: MutableRefObject<any> }) {
  const { camera } = useThree();

  useEffect(() => {
    if (!bounds) return;
    const min = new THREE.Vector3(bounds[0], bounds[1], bounds[2]);
    const max = new THREE.Vector3(bounds[3], bounds[4], bounds[5]);
    const center = min.clone().add(max).multiplyScalar(0.5);
    const diagonal = max.distanceTo(min);
    const distance = Math.max(diagonal * 1.6, 50);
    camera.position.copy(center.clone().add(new THREE.Vector3(distance, distance * 0.8, distance)));
    camera.lookAt(center);
    if (controlsRef.current) {
      controlsRef.current.target.copy(center);
      controlsRef.current.update();
    }
  }, [bounds, camera, controlsRef]);

  return null;
}

function TreeNode({ node, index, selectedId, onSelect, depth = 0 }: { node: CompiledNode; index: ModelIndex; selectedId: string | null; onSelect: (id: string) => void; depth?: number }) {
  const children = index.getChildren(node.id);
  const [expanded, setExpanded] = useState(depth < 1);
  return <div>
    <button className={`tree-row ${selectedId === node.id ? 'selected' : ''}`} style={{ paddingLeft: 10 + depth * 16 }} onClick={() => onSelect(node.id)}>
      {children.length > 0 ? <span className="tree-toggle" onClick={(event) => { event.stopPropagation(); setExpanded(value => !value); }}>{expanded ? '▾' : '▸'}</span> : <span className="tree-toggle">·</span>}
      <span>{node.name}</span><small>{node.kind}</small>
    </button>
    {expanded && children.map(child => <TreeNode key={child.id} node={child} index={index} selectedId={selectedId} onSelect={onSelect} depth={depth + 1} />)}
  </div>;
}

export default function ModelViewer({ compiled, build }: { compiled: CompiledModelPackage; build: BuildPackage }) {
  validateCompiledModelPackage(compiled);
  const index = useMemo(() => new ModelIndex(compiled.manifest), [compiled]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [hiddenIds, setHiddenIds] = useState<Set<string>>(new Set());
  const controlsRef = useRef<any>(null);
  const geometry3D = useMemo(() => build3DEntries(build), [build]);
  const selectedNode = selectedId ? index.get(selectedId) : undefined;

  const toggleVisibility = () => {
    if (!selectedNode?.capabilities.hideable) return;
    setHiddenIds(current => {
      const next = new Set(current);
      if (next.has(selectedNode.id)) next.delete(selectedNode.id);
      else next.add(selectedNode.id);
      return next;
    });
  };

  return <div className="viewer-shell">
    <aside className="tree-panel">
      <header><strong>{build.applicationId}</strong><span>{build.buildIdentity.slice(0, 12)}</span></header>
      {compiled.manifest.rootNodeIds.map(id => {
        const node = index.get(id);
        return node ? <TreeNode key={id} node={node} index={index} selectedId={selectedId} onSelect={setSelectedId} /> : null;
      })}
    </aside>

    <main className="viewport-panel">
      <Canvas camera={{ position: [180, 140, 180], fov: 45 }}>
        <color attach="background" args={['#11151c']} />
        <ambientLight intensity={1.1} />
        <directionalLight position={[100, 140, 160]} intensity={2} />
        <directionalLight position={[-80, 60, -40]} intensity={0.8} />
        <gridHelper args={[260, 26]} />
        {geometry3D.solids.map(entry => {
          const partNodeId = `definition:part:${encodeSegment(entry.part.id)}`;
          return <SolidObject
            key={`${entry.part.id}:${entry.id}`}
            entry={entry}
            selected={selectedId === entry.nodeId}
            hidden={index.isHidden(partNodeId, hiddenIds)}
            onSelect={setSelectedId}
          />;
        })}
        {geometry3D.edges.map(entry => {
          const partNodeId = `definition:part:${encodeSegment(entry.part.id)}`;
          return <EdgeObject
            key={`${entry.part.id}:${entry.item.id}`}
            entry={entry}
            selected={selectedId === entry.nodeId}
            hidden={index.isHidden(partNodeId, hiddenIds)}
            onSelect={setSelectedId}
          />;
        })}
        <OrbitControls ref={controlsRef} makeDefault />
        <CameraFit bounds={geometry3D.bounds} controlsRef={controlsRef} />
      </Canvas>
      <div className="toolbar">
        <button onClick={() => {
          if (!geometry3D.bounds || !controlsRef.current) return;
          const min = new THREE.Vector3(geometry3D.bounds[0], geometry3D.bounds[1], geometry3D.bounds[2]);
          const max = new THREE.Vector3(geometry3D.bounds[3], geometry3D.bounds[4], geometry3D.bounds[5]);
          const center = min.clone().add(max).multiplyScalar(0.5);
          controlsRef.current.target.copy(center);
          controlsRef.current.update();
        }}>Fit</button>
        <button onClick={() => setSelectedId(null)}>Clear</button>
      </div>
    </main>

    <aside className="inspector">
      {selectedNode ? <>
        <h2>{selectedNode.name}</h2>
        <div className="kind">{selectedNode.kind}</div>
        <dl><dt>ID</dt><dd>{selectedNode.id}</dd><dt>Parent</dt><dd>{selectedNode.parentId ?? '—'}</dd></dl>
        <h3>Metadata</h3>
        {Object.entries(selectedNode.metadata).map(([key, value]) => <div className="property" key={key}><span>{key}</span><code>{JSON.stringify(value satisfies JsonValue)}</code></div>)}
        <button onClick={toggleVisibility}>{hiddenIds.has(selectedNode.id) ? 'Show' : 'Hide'} selected</button>
      </> : <div className="empty">Select a model geometry.</div>}
    </aside>
  </div>;
}

export async function loadBuildPackage(url: string): Promise<BuildPackage> {
  const response = await fetch(url, { cache: 'no-store' });
  if (!response.ok) throw new Error(`Unable to load build package: HTTP ${response.status}.`);
  return await response.json() as BuildPackage;
}
