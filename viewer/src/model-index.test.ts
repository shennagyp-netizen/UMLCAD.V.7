import { describe, expect, it } from 'vitest';
import type { CompiledModelManifest, CompiledNode } from './types';
import { ModelIndex } from './model-index';

const caps = { visible: true, hideable: true, selectable: true, focusable: true };
const node = (id: string, name: string, parentId: string | null = null, childIds: string[] = []): CompiledNode => ({
  id,
  name,
  kind: 'generic',
  parentId,
  childIds,
  metadata: {},
  relationshipIds: [],
  representationIds: [],
  capabilities: caps,
  source: null,
  state: {},
});

const manifest = (): CompiledModelManifest => ({
  schema: 'uml-cad-compiled-model/1.1.0',
  applicationId: 'app',
  applicationVersion: '1',
  buildIdentity: 'b',
  rootNodeIds: ['root'],
  nodes: [node('root', 'Root', null, ['b', 'a']), node('a', 'Same', 'root'), node('b', 'Same', 'root')],
  relationships: [],
  representations: [],
  topologyBindings: [],
  sourceBindings: [],
  diagnostics: [],
});

describe('ModelIndex', () => {
  it('sorts children by name then id', () =>
    expect(new ModelIndex(manifest()).getChildren('root').map(x => x.id)).toEqual(['a', 'b']));

  it('returns a full root-to-leaf path', () => {
    const m = manifest();
    m.nodes.push(node('leaf', 'Leaf', 'a'));
    m.nodes.find(x => x.id === 'a')!.childIds.push('leaf');
    expect(new ModelIndex(m).pathTo('leaf').map(x => x.id)).toEqual(['root', 'a', 'leaf']);
  });

  it('returns an empty path for an unknown id', () =>
    expect(new ModelIndex(manifest()).pathTo('missing')).toEqual([]));

  it('uses representation bounds and indexes topology by render primitive', () => {
    const m = manifest();
    m.nodes[1].representationIds.push('rep');
    m.representations.push({
      id: 'rep', kind: 'mesh', artifactId: 'artifact', renderNodeId: 'a',
      bounds: [0, 0, 0, 2, 2, 2], capabilities: {}, selectableSubTargets: [],
    });
    m.topologyBindings.push({
      id: 'face-1', topologyKind: 'face', semanticNodeId: 'a', artifactId: 'artifact',
      renderPrimitiveId: 'primitive-1', name: 'Face', nomenclature: 'F1', number: 1,
      bounds: [0, 0, 0, 1, 1, 1], metadata: {},
    });
    const index = new ModelIndex(m);
    expect(index.boundsFor('a')).toEqual([0, 0, 0, 2, 2, 2]);
    expect(index.getTopologyForRenderPrimitive('primitive-1')?.id).toBe('face-1');
  });

  it('propagates explicit visibility through ancestors', () => {
    const m = manifest();
    m.nodes.push(node('leaf', 'Leaf', 'a'));
    m.nodes.find(x => x.id === 'a')!.childIds.push('leaf');
    const index = new ModelIndex(m);
    expect(index.isHidden('leaf', new Set(['a']))).toBe(true);
    expect(index.isHidden('leaf', new Set(['root']))).toBe(true);
    expect(index.isHidden('leaf', new Set(['b']))).toBe(false);
    expect(index.isHidden('leaf', new Set(['leaf']))).toBe(true);
  });

  it('detects malformed parent cycles during path resolution', () => {
    const m = manifest();
    m.nodes[0].parentId = 'a';
    expect(() => new ModelIndex(m).pathTo('root')).toThrow(/Cycle detected/);
  });
});
