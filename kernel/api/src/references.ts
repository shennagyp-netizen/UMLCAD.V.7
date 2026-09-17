export interface SemanticReference{readonly kind:string;readonly id:string;readonly path?:readonly string[]}
export type ReferenceMigrationMap=Readonly<Record<string,string>>
export function resolveReference(source:SemanticReference,migration:ReferenceMigrationMap):readonly SemanticReference[]{return Object.freeze([{...source,id:migration[source.id]??source.id}])}
