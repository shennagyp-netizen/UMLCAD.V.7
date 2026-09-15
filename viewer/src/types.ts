export type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

export interface CompiledNode {
  id: string;
  name: string;
  kind: string;
  parentId?: string | null;
  childIds: string[];
  metadata: Record<string, JsonValue>;
  relationshipIds: string[];
  representationIds: string[];
  capabilities: {
    visible: boolean;
    hideable: boolean;
    selectable: boolean;
    focusable: boolean;
  };
  source?: { file: string; symbol?: string; start?: string; end?: string; revision?: string } | null;
  state: Record<string, string>;
}

export interface CompiledRelationship {
  id: string;
  kind: string;
  sourceId: string;
  targetIds: string[];
  metadata: Record<string, JsonValue>;
}

export interface CompiledRepresentation {
  id: string;
  kind: string;
  artifactId: string;
  renderNodeId?: string | null;
  bounds?: number[] | null;
  capabilities: Record<string, string>;
  selectableSubTargets: string[];
}

export interface CompiledTopologyBinding {
  id: string;
  topologyKind: string;
  semanticNodeId: string;
  artifactId: string;
  renderPrimitiveId: string;
  name?: string | null;
  nomenclature?: string | null;
  number?: number | null;
  bounds?: number[] | null;
  metadata: Record<string, JsonValue>;
}

export interface CompiledDiagnostic {
  code: string;
  severity: string;
  message: string;
  targetId?: string | null;
}

export interface CompiledModelManifest {
  schema: string;
  applicationId: string;
  applicationVersion: string;
  buildIdentity: string;
  rootNodeIds: string[];
  nodes: CompiledNode[];
  relationships: CompiledRelationship[];
  representations: CompiledRepresentation[];
  topologyBindings: CompiledTopologyBinding[];
  sourceBindings: Array<{ file: string; symbol?: string; start?: string; end?: string; revision?: string }>;
  diagnostics: CompiledDiagnostic[];
}

export interface CompiledRenderArtifact {
  artifactId: string;
  buildIdentity: string;
  format: string;
  mediaType: string;
  assetIdentity: string;
  sizeBytes?: number | null;
  integritySha256?: string | null;
  uri?: string | null;
  inlineBase64?: string | null;
}

export interface CompiledModelPackage {
  schema: string;
  applicationId: string;
  applicationVersion: string;
  buildIdentity: string;
  manifest: CompiledModelManifest;
  renderArtifact?: CompiledRenderArtifact | null;
  diagnostics: CompiledDiagnostic[];
}
