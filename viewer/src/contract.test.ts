import { describe, expect, it } from 'vitest';
import type { CompiledModelManifest, CompiledModelPackage, CompiledNode } from './types';
import { validateCompiledModelPackage } from './validate-package';

const caps = { visible: true, hideable: true, selectable: true, focusable: true };

const node = (id: string, parentId: string | null = null, childIds: string[] = []): CompiledNode => ({
  id,
  name: id,
  kind: 'future:any',
  parentId,
  childIds,
  metadata: {},
  relationshipIds: [],
  representationIds: [],
  capabilities: caps,
  source: null,
  state: {},
});

const manifest = (
  nodes: CompiledNode[] = [node('root')],
  extra: Partial<CompiledModelManifest> = {},
): CompiledModelManifest => ({
  schema: 'uml-cad-compiled-model/1.1.0',
  applicationId: 'app',
  applicationVersion: '1.0',
  buildIdentity: 'build',
  rootNodeIds: ['root'],
  nodes,
  relationships: [],
  representations: [],
  topologyBindings: [],
  sourceBindings: [],
  diagnostics: [],
  ...extra,
});

const pkg = (
  m: CompiledModelManifest = manifest(),
  extra: Partial<CompiledModelPackage> = {},
): CompiledModelPackage => ({
  schema: 'uml-cad-compiled-model/1.1.0',
  applicationId: 'app',
  applicationVersion: '1.0',
  buildIdentity: 'build',
  manifest: m,
  renderArtifact: null,
  diagnostics: [],
  ...extra,
});

describe('compiled package validation', () => {
  it('accepts unknown kinds', () => {
    expect(() => validateCompiledModelPackage(pkg())).not.toThrow();
  });

  it('rejects duplicate node ids', () => {
    const duplicate = pkg(manifest([node('root'), node('root')]));
    expect(() => validateCompiledModelPackage(duplicate)).toThrow(/Duplicate/i);
  });

  it('rejects missing roots', () => {
    const missingRoot = pkg(manifest([node('root')], { rootNodeIds: ['missing'] }));
    expect(() => validateCompiledModelPackage(missingRoot)).toThrow(/does not resolve/i);
  });

  it('rejects missing parents', () => {
    const missingParent = pkg(manifest([node('root', 'missing')]));
    expect(() => validateCompiledModelPackage(missingParent)).toThrow(/Parent/i);
  });

  it('rejects hierarchy cycles', () => {
    const a = node('a', null, ['b']);
    const b = node('b', 'a', ['a']);
    const cyclic = pkg(manifest([a, b], { rootNodeIds: ['a'] }));
    expect(() => validateCompiledModelPackage(cyclic)).toThrow(/cycle/i);
  });

  it('rejects relationship source corruption', () => {
    const root: CompiledNode = { ...node('root'), relationshipIds: ['r'] };
    const rel = { id: 'r', kind: 'ref', sourceId: 'missing', targetIds: ['root'], metadata: {} };
    const corrupted = pkg(manifest([root], { relationships: [rel] }));
    expect(() => validateCompiledModelPackage(corrupted)).toThrow(/source/i);
  });

  it('rejects duplicate relationships', () => {
    const rel = { id: 'r', kind: 'ref', sourceId: 'root', targetIds: ['root'], metadata: {} };
    const duplicate = pkg(manifest([node('root')], { relationships: [rel, { ...rel }] }));
    expect(() => validateCompiledModelPackage(duplicate)).toThrow(/Duplicate/i);
  });

  it('rejects unsafe artifact URI', () => {
    const artifact = {
      artifactId: 'artifact',
      buildIdentity: 'build',
      format: 'glb',
      mediaType: 'model/gltf-binary',
      assetIdentity: 'asset',
      sizeBytes: 1,
      integritySha256: null,
      uri: 'javascript:alert(1)',
      inlineBase64: null,
      properties: {},
    };
    const unsafe = pkg(manifest(), { renderArtifact: artifact });
    expect(() => validateCompiledModelPackage(unsafe)).toThrow(/URI/i);
  });

  it('rejects malformed artifact integrity and ambiguous payload sources', () => {
    const invalidDigest = {
      artifactId: 'artifact',
      buildIdentity: 'build',
      format: 'glb',
      mediaType: 'model/gltf-binary',
      assetIdentity: 'asset',
      sizeBytes: 1,
      integritySha256: 'not-a-sha256',
      uri: 'https://example.invalid/model.glb',
      inlineBase64: null,
      properties: {},
    };
    const malformed = pkg(manifest(), { renderArtifact: invalidDigest });
    expect(() => validateCompiledModelPackage(malformed)).toThrow(/SHA-256/i);

    const bothSources = { ...invalidDigest, integritySha256: null, inlineBase64: 'YWJj' };
    const ambiguous = pkg(manifest(), { renderArtifact: bothSources });
    expect(() => validateCompiledModelPackage(ambiguous)).toThrow(/exactly one/i);
  });
});
