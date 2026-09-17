import type { CompiledModelPackage, CompiledModelManifest } from './types';

const MAX_NODES = 2_000_000;
const MAX_RELATIONSHIPS = 4_000_000;
const MAX_REPRESENTATIONS = 2_000_000;
const MAX_TOPOLOGY_BINDINGS = 4_000_000;
const MAX_DEPTH = 20_000;
const MAX_METADATA_KEYS = 128;
const MAX_INLINE_BYTES = 16 * 1024 * 1024;
const MAX_ARTIFACT_BYTES = 2 * 1024 * 1024 * 1024;
const SHA256_HEX = /^[0-9a-f]{64}$/i;

export function validateCompiledModelPackage(pkg: CompiledModelPackage): void {
  if (pkg.schema !== 'uml-cad-compiled-model/1.1.0') throw new Error(`Unsupported compiled-model schema '${pkg.schema}'.`);
  if (pkg.manifest.schema !== pkg.schema) throw new Error('Manifest schema does not match package schema.');
  if (pkg.buildIdentity !== pkg.manifest.buildIdentity) throw new Error('Package and manifest build identities differ.');
  if (pkg.applicationId !== pkg.manifest.applicationId || pkg.applicationVersion !== pkg.manifest.applicationVersion) throw new Error('Compiled application identity is inconsistent.');

  const artifact = pkg.renderArtifact ?? null;
  if (artifact) {
    if (!artifact.artifactId || !artifact.assetIdentity) throw new Error('Render artifact identity is incomplete.');
    if (artifact.buildIdentity !== pkg.buildIdentity) throw new Error('Render artifact belongs to a different build.');
    if (artifact.format.toLowerCase() !== 'glb' && artifact.format.toLowerCase() !== 'gltf') throw new Error(`Unsupported render format '${artifact.format}'.`);
    if (artifact.sizeBytes != null && (!Number.isFinite(artifact.sizeBytes) || artifact.sizeBytes < 0 || artifact.sizeBytes > MAX_ARTIFACT_BYTES)) throw new Error('Render artifact size is invalid.');
    if (artifact.integritySha256 != null && !SHA256_HEX.test(artifact.integritySha256)) throw new Error('Render artifact integritySha256 is not a valid SHA-256 hexadecimal digest.');
    if (artifact.inlineBase64 && Math.floor(artifact.inlineBase64.length * 0.75) > MAX_INLINE_BYTES) throw new Error('Inline render artifact exceeds the viewer limit.');
    if (artifact.inlineBase64 != null && !isBase64(artifact.inlineBase64)) throw new Error('Inline render artifact is not valid base64.');
    if (artifact.uri != null && !isAllowedUri(artifact.uri)) throw new Error('Render artifact URI is not allowed.');
    const hasUri = artifact.uri != null && artifact.uri.length > 0;
    const hasInline = artifact.inlineBase64 != null && artifact.inlineBase64.length > 0;
    if (hasUri === hasInline) throw new Error('Render artifact must contain exactly one non-empty URI or inline payload.');
  }

  validateManifest(pkg.manifest, artifact?.artifactId ?? null);
}

function validateManifest(manifest: CompiledModelManifest, artifactId: string | null): void {
  if (manifest.nodes.length > MAX_NODES) throw new Error('Compiled model contains too many nodes.');
  if (manifest.relationships.length > MAX_RELATIONSHIPS) throw new Error('Compiled model contains too many relationships.');
  if (manifest.representations.length > MAX_REPRESENTATIONS) throw new Error('Compiled model contains too many representations.');
  if (manifest.topologyBindings.length > MAX_TOPOLOGY_BINDINGS) throw new Error('Compiled model contains too many topology bindings.');

  const nodes = new Map<string, CompiledModelManifest['nodes'][number]>();
  for (const node of manifest.nodes) {
    if (!node.id || !node.name || !node.kind || nodes.has(node.id)) throw new Error(`Duplicate, empty, or incomplete node ID '${node.id}'.`);
    if (new Set(node.childIds).size !== node.childIds.length) throw new Error(`Node '${node.id}' contains duplicate child IDs.`);
    if (Object.keys(node.metadata).length > MAX_METADATA_KEYS) throw new Error(`Node '${node.id}' has too much metadata.`);
    nodes.set(node.id, node);
  }

  const roots = new Set<string>();
  for (const root of manifest.rootNodeIds) {
    if (roots.has(root)) throw new Error(`Root node '${root}' is duplicated.`);
    roots.add(root);
    const node = nodes.get(root);
    if (!node) throw new Error(`Root node '${root}' does not resolve.`);
    if (node.parentId != null) throw new Error(`Root node '${root}' unexpectedly has parent '${node.parentId}'.`);
  }

  const relationships = new Map<string, CompiledModelManifest['relationships'][number]>();
  for (const relationship of manifest.relationships) {
    if (!relationship.id || relationships.has(relationship.id)) throw new Error(`Duplicate or empty relationship ID '${relationship.id}'.`);
    if (!nodes.has(relationship.sourceId)) throw new Error(`Relationship '${relationship.id}' source does not resolve.`);
    if (relationship.targetIds.length === 0) throw new Error(`Relationship '${relationship.id}' has no targets.`);
    for (const target of relationship.targetIds) if (!nodes.has(target)) throw new Error(`Relationship '${relationship.id}' target '${target}' does not resolve.`);
    relationships.set(relationship.id, relationship);
  }

  const representations = new Map<string, CompiledModelManifest['representations'][number]>();
  for (const representation of manifest.representations) {
    if (!representation.id || representations.has(representation.id)) throw new Error(`Duplicate or empty representation ID '${representation.id}'.`);
    if (artifactId !== null && representation.artifactId !== artifactId) throw new Error(`Representation '${representation.id}' references the wrong artifact.`);
    if (representation.renderNodeId != null && !nodes.has(representation.renderNodeId)) throw new Error(`Representation '${representation.id}' render node does not resolve.`);
    validateBounds(representation.bounds, `Representation '${representation.id}'`);
    representations.set(representation.id, representation);
  }

  const topologyIds = new Set<string>();
  const primitiveIds = new Set<string>();
  for (const binding of manifest.topologyBindings) {
    if (!binding.id || topologyIds.has(binding.id)) throw new Error(`Duplicate or empty topology binding '${binding.id}'.`);
    if (!binding.renderPrimitiveId || primitiveIds.has(binding.renderPrimitiveId)) throw new Error(`Duplicate or empty render primitive ID '${binding.renderPrimitiveId}'.`);
    if (!nodes.has(binding.semanticNodeId)) throw new Error(`Topology target '${binding.semanticNodeId}' does not resolve.`);
    if (artifactId !== null && binding.artifactId !== artifactId) throw new Error(`Topology binding '${binding.id}' references the wrong artifact.`);
    validateBounds(binding.bounds, `Topology binding '${binding.id}'`);
    topologyIds.add(binding.id);
    primitiveIds.add(binding.renderPrimitiveId);
  }

  for (const node of manifest.nodes) {
    if (node.parentId && !nodes.has(node.parentId)) throw new Error(`Parent '${node.parentId}' of '${node.id}' does not resolve.`);
    for (const childId of node.childIds) {
      const child = nodes.get(childId);
      if (!child) throw new Error(`Child '${childId}' of '${node.id}' does not resolve.`);
      if (child.parentId !== node.id) throw new Error(`Child '${childId}' does not point back to '${node.id}'.`);
    }
    for (const relationshipId of node.relationshipIds) {
      const relationship = relationships.get(relationshipId);
      if (!relationship) throw new Error(`Relationship '${relationshipId}' listed by '${node.id}' does not resolve.`);
      if (relationship.sourceId !== node.id) throw new Error(`Relationship '${relationshipId}' is not sourced by '${node.id}'.`);
    }
    for (const representationId of node.representationIds) {
      if (!representations.has(representationId)) throw new Error(`Representation '${representationId}' of '${node.id}' does not resolve.`);
    }
  }

  detectCycles(nodes);
}

function validateBounds(bounds: number[] | null | undefined, owner: string): void {
  if (bounds == null) return;
  if (bounds.length !== 6 || bounds.some(value => !Number.isFinite(value))) throw new Error(`${owner} has invalid bounds.`);
}

function detectCycles(nodes: Map<string, CompiledModelManifest['nodes'][number]>): void {
  const state = new Map<string, 0 | 1 | 2>();
  const stack: Array<{ id: string; index: number }> = [];
  for (const node of nodes.values()) {
    if (state.get(node.id) === 2) continue;
    stack.push({ id: node.id, index: 0 });
    state.set(node.id, 1);
    while (stack.length) {
      const frame = stack[stack.length - 1];
      const current = nodes.get(frame.id)!;
      if (frame.index >= current.childIds.length) {
        state.set(frame.id, 2);
        stack.pop();
        continue;
      }
      const childId = current.childIds[frame.index++];
      const childState = state.get(childId) ?? 0;
      if (childState === 1) throw new Error(`Compiled model hierarchy contains a cycle involving '${childId}'.`);
      if (childState === 2) continue;
      if (stack.length >= MAX_DEPTH) throw new Error('Compiled model hierarchy exceeds the maximum supported depth.');
      state.set(childId, 1);
      stack.push({ id: childId, index: 0 });
    }
  }
}

function isAllowedUri(value: string): boolean {
  try {
    const uri = new URL(value);
    return uri.protocol === 'http:' || uri.protocol === 'https:';
  } catch {
    return false;
  }
}

function isBase64(value: string): boolean {
  if (value.length % 4 !== 0 || !/^[A-Za-z0-9+/]*={0,2}$/.test(value)) return false;
  try {
    return btoa(atob(value)) === value;
  } catch {
    return false;
  }
}
