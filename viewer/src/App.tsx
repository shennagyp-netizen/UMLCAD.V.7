import { Canvas, useThree } from '@react-three/fiber';
import { OrbitControls, useGLTF } from '@react-three/drei';
import { useEffect, useMemo, useRef, useState, type MutableRefObject } from 'react';
import * as THREE from 'three';
import type { CompiledModelPackage, CompiledNode } from './types';
import { ModelIndex } from './model-index';
import { validateCompiledModelPackage } from './validate-package';

function RenderModel({
  uri,
  selectedId,
  hiddenIds,
  index,
  onSelect,
}: {
  uri: string;
  selectedId: string | null;
  hiddenIds: Set<string>;
  index: ModelIndex;
  onSelect: (id: string) => void;
}) {
  const gltf = useGLTF(uri);

  useEffect(() => {
    gltf.scene.traverse((object) => {
      const mesh = object as THREE.Mesh;
      const semanticId = object.userData?.umlCadId as string | undefined;
      const topologyId = object.userData?.umlCadTopologyId as string | undefined;
      const binding = topologyId ? index.getTopology(topologyId) : undefined;

      if (semanticId) object.visible = !index.isHidden(semanticId, hiddenIds);
      if (!mesh.isMesh) return;

      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
      object.userData.umlCadOriginalMaterials ??= materials.map((material) => material.clone());
      const originals = object.userData.umlCadOriginalMaterials as THREE.Material[];
      const selected = semanticId === selectedId || binding?.id === selectedId;
      const nextMaterials = originals.map((source) => {
        const next = source.clone();
        if (selected && next instanceof THREE.MeshStandardMaterial) next.emissive.setHex(0x3366ff);
        return next;
      });
      mesh.material = Array.isArray(mesh.material) ? nextMaterials : nextMaterials[0];
    });
  }, [gltf.scene, hiddenIds, index, selectedId]);

  return <primitive object={gltf.scene} onPointerDown={(event: any) => {
    event.stopPropagation();
    const topologyId = event.object?.userData?.umlCadTopologyId as string | undefined;
    if (topologyId && index.getTopology(topologyId)) {
      onSelect(topologyId);
      return;
    }

    let object: THREE.Object3D | null = event.object;
    while (object) {
      const id = object.userData?.umlCadId as string | undefined;
      if (id) {
        onSelect(id);
        return;
      }
      object = object.parent;
    }
  }} />;
}

function CameraFocus({ focusId, index, controlsRef }: { focusId: string | null; index: ModelIndex; controlsRef: MutableRefObject<any> }) {
  const { camera } = useThree();

  useEffect(() => {
    if (!focusId) return;
    const bounds = index.boundsFor(focusId);
    if (!bounds) return;

    const min = new THREE.Vector3(bounds[0], bounds[1], bounds[2]);
    const max = new THREE.Vector3(bounds[3], bounds[4], bounds[5]);
    const center = min.clone().add(max).multiplyScalar(0.5);
    const radius = Math.max(max.distanceTo(min) * 0.75, 1);

    camera.position.copy(center.clone().add(new THREE.Vector3(radius, radius, radius)));
    if (controlsRef.current) {
      controlsRef.current.target.copy(center);
      controlsRef.current.update();
    }
  }, [camera, controlsRef, focusId, index]);

  return null;
}

function TreeNode({ node, index, selectedId, onSelect, depth = 0 }: { node: CompiledNode; index: ModelIndex; selectedId: string | null; onSelect: (id: string) => void; depth?: number }) {
  const children = index.getChildren(node.id);
  const [expanded, setExpanded] = useState(depth < 2);
  const visible = node.capabilities.visible && !node.state.suppressed;

  return <div>
    <button className={`tree-row ${selectedId === node.id ? 'selected' : ''}`} style={{ paddingLeft: 10 + depth * 16 }} onClick={() => onSelect(node.id)}>
      {children.length > 0 ? <span className="tree-toggle" onClick={(event) => { event.stopPropagation(); setExpanded((value) => !value); }}>{expanded ? '▾' : '▸'}</span> : <span className="tree-toggle">·</span>}
      <span>{node.name}</span><small>{node.kind}</small><small>{visible ? 'shown' : 'hidden'}</small>
    </button>
    {expanded && children.map(child => <TreeNode key={child.id} node={child} index={index} selectedId={selectedId} onSelect={onSelect} depth={depth + 1} />)}
  </div>;
}

export default function App({ compiled }: { compiled: CompiledModelPackage }) {
  validateCompiledModelPackage(compiled);
  const index = useMemo(() => new ModelIndex(compiled.manifest), [compiled]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [focusId, setFocusId] = useState<string | null>(null);
  const [hiddenIds, setHiddenIds] = useState<Set<string>>(new Set());
  const controlsRef = useRef<any>(null);
  const selectedNode = selectedId ? index.get(selectedId) : undefined;
  const selectedTopology = selectedId ? index.getTopology(selectedId) : undefined;
  const roots = compiled.manifest.rootNodeIds.map(id => index.get(id)).filter((node): node is CompiledNode => !!node);
  const renderUri = compiled.renderArtifact?.uri;

  const toggleVisibility = () => {
    if (!selectedNode || !selectedNode.capabilities.hideable) return;
    setHiddenIds(current => {
      const next = new Set(current);
      if (next.has(selectedNode.id)) next.delete(selectedNode.id);
      else next.add(selectedNode.id);
      return next;
    });
  };

  const focusTarget = selectedTopology?.semanticNodeId ?? selectedNode?.id ?? null;

  return <div className="viewer-shell">
    <aside className="tree-panel">
      <header><strong>Model</strong><span>{compiled.buildIdentity.slice(0, 12)}</span></header>
      {roots.map(node => <TreeNode key={node.id} node={node} index={index} selectedId={selectedId} onSelect={setSelectedId} />)}
    </aside>
    <main className="viewport-panel">
      <Canvas camera={{ position: [5, 5, 5], fov: 45 }}>
        <color attach="background" args={['#11151c']} />
        <ambientLight intensity={1.2} />
        <directionalLight position={[5, 10, 5]} intensity={2} />
        {renderUri ? <RenderModel uri={renderUri} selectedId={selectedId} hiddenIds={hiddenIds} index={index} onSelect={setSelectedId} /> : null}
        <gridHelper args={[20, 20]} />
        <OrbitControls ref={controlsRef} makeDefault />
        <CameraFocus focusId={focusId} index={index} controlsRef={controlsRef} />
      </Canvas>
      <div className="toolbar">
        <button onClick={() => setFocusId(focusTarget)}>Focus</button>
        <button onClick={() => setFocusId(null)}>Reset focus</button>
        <button onClick={() => setSelectedId(null)}>Clear</button>
      </div>
    </main>
    <aside className="inspector">
      {selectedNode || selectedTopology ? <>
        <h2>{selectedTopology?.name ?? selectedNode?.name}</h2>
        <div className="kind">{selectedTopology?.topologyKind ?? selectedNode?.kind}</div>
        {selectedTopology ? <>
          <dl><dt>Topology ID</dt><dd>{selectedTopology.id}</dd><dt>Number</dt><dd>{selectedTopology.number ?? '—'}</dd><dt>Nomenclature</dt><dd>{selectedTopology.nomenclature ?? '—'}</dd><dt>Object</dt><dd>{selectedTopology.semanticNodeId}</dd></dl>
          <h3>Face metadata</h3>
          {Object.entries(selectedTopology.metadata).map(([key, value]) => <div className="property" key={key}><span>{key}</span><code>{typeof value === 'string' ? value : JSON.stringify(value)}</code></div>)}
        </> : <>
          <dl><dt>ID</dt><dd>{selectedNode!.id}</dd><dt>Parent</dt><dd>{selectedNode!.parentId ?? '—'}</dd></dl>
          <h3>Metadata</h3>
          {Object.entries(selectedNode!.metadata).map(([key, value]) => <div className="property" key={key}><span>{key}</span><code>{typeof value === 'string' ? value : JSON.stringify(value)}</code></div>)}
          <button onClick={toggleVisibility}>{hiddenIds.has(selectedNode!.id) ? 'Show' : 'Hide'} selected</button>
        </>}
      </> : <div className="empty">Select an object or topology target.</div>}
    </aside>
  </div>;
}
