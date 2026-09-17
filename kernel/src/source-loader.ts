import { resolve, relative } from "node:path";
import { pathToFileURL } from "node:url";
import type { PartSourceLoader } from "./contracts.js";

export class FilesystemPartSourceLoader implements PartSourceLoader {
  private readonly root: string;

  public constructor(rootDirectory: string) {
    this.root = resolve(rootDirectory);
  }

  public async loadDrawing(moduleId: string, exportName: string): Promise<unknown> {
    if (!moduleId || moduleId.startsWith("/") || moduleId.includes("\\")) {
      throw new Error(`Invalid moduleId: ${moduleId}`);
    }
    const absolute = resolve(this.root, moduleId);
    const escaped = relative(this.root, absolute);
    if (escaped.startsWith("..") || escaped.includes("/../")) {
      throw new Error(`Module path escapes configured part root: ${moduleId}`);
    }
    const module = await import(pathToFileURL(absolute).href);
    if (!(exportName in module)) throw new Error(`Export not found: ${moduleId}#${exportName}`);
    const value = module[exportName];
    return typeof value === "function" ? value() : value;
  }
}
