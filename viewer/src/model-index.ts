import type { CompiledModelManifest, CompiledNode, CompiledTopologyBinding } from './types';

export class ModelIndex {
  readonly nodes = new Map<string, CompiledNode>();
  readonly children = new Map<string, CompiledNode[]>();
  readonly representations = new Map<string, CompiledModelManifest['representations'][number]>();
  readonly topology = new Map<string, CompiledTopologyBinding>();
  readonly topologyByRenderPrimitive = new Map<string, CompiledTopologyBinding>();

  constructor(readonly manifest: CompiledModelManifest) {
    for (const node of manifest.nodes) this.nodes.set(node.id, node);
    for (const representation of manifest.representations) this.representations.set(representation.id, representation);
    for (const binding of manifest.topologyBindings) {
      this.topology.set(binding.id, binding);
      this.topologyByRenderPrimitive.set(binding.renderPrimitiveId, binding);
    }
    for (const node of manifest.nodes) {
      if (!node.parentId) continue;
      const list = this.children.get(node.parentId) ?? [];
      list.push(node);
      this.children.set(node.parentId, list);
    }
    for (const [id, list] of this.children) {
      list.sort((a, b) => a.name.localeCompare(b.name) || a.id.localeCompare(b.id));
      this.children.set(id, list);
    }
  }

  get(id: string): CompiledNode | undefined { return this.nodes.get(id); }
  getChildren(id: string): CompiledNode[] { return this.children.get(id) ?? []; }
  getTopology(id: string): CompiledTopologyBinding | undefined { return this.topology.get(id); }
  getTopologyForRenderPrimitive(id: string): CompiledTopologyBinding | undefined { return this.topologyByRenderPrimitive.get(id); }

  isHidden(id: string, explicitlyHidden: ReadonlySet<string>): boolean {
    let current = this.get(id);
    const seen = new Set<string>();
    while (current) {
      if (!seen.add(current.id)) throw new Error(`Cycle detected while resolving visibility for '${id}'.`);
      if (explicitlyHidden.has(current.id)) return true;
      current = current.parentId ? this.get(current.parentId) : undefined;
    }
    return false;
  }

  boundsFor(id: string): number[] | undefined {
    const node = this.get(id);
    if (!node) return undefined;
    for (const representationId of node.representationIds) {
      const bounds = this.representations.get(representationId)?.bounds;
      if (bounds && bounds.length === 6) return bounds;
    }
    const topology = this.getTopology(id);
    return topology?.bounds && topology.bounds.length === 6 ? topology.bounds : undefined;
  }

  pathTo(id: string): CompiledNode[] {
    const result: CompiledNode[] = [];
    let current = this.get(id);
    const seen = new Set<string>();
    while (current) {
      if (!seen.add(current.id)) throw new Error(`Cycle detected while resolving path to '${id}'.`);
      result.unshift(current);
      current = current.parentId ? this.get(current.parentId) : undefined;
    }
    return result;
  }
}
