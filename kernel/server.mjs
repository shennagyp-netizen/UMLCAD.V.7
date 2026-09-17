import { createServer } from "node:http";
import { FilesystemPartSourceLoader, V4KernelApplication, analyzeSourceModule, orderSourceModules } from "./src/index.js";

const port = Number.parseInt(process.env.PORT ?? "8080", 10);
const root = process.env.UMLCAD_PART_ROOT;
if (!root) throw new Error("UMLCAD_PART_ROOT is required");
const application = new V4KernelApplication(new FilesystemPartSourceLoader(root));

async function readJson(request) {
  const chunks = [];
  for await (const chunk of request) chunks.push(chunk);
  const text = Buffer.concat(chunks).toString("utf8");
  return text ? JSON.parse(text) : {};
}
function send(response, status, value) { const body = JSON.stringify(value); response.statusCode = status; response.setHeader("content-type", "application/json; charset=utf-8"); response.setHeader("content-length", Buffer.byteLength(body)); response.end(body); }
const server = createServer(async (request, response) => {
  try {
    if (request.method !== "POST") return send(response, 405, { error: "method-not-allowed" });
    const body = await readJson(request);
    switch (request.url) {
      case "/v1/build/part": return send(response, 200, await application.buildPart(body));
      case "/v1/build/assembly": return send(response, 200, await application.buildAssembly(body));
      case "/v1/analyze/constraints": return send(response, 200, await application.analyzeConstraints(body));
      case "/v1/solve": return send(response, 200, await application.solveConstraints(body));
      case "/v1/analyze/dimensions": return send(response, 200, await application.evaluateDimensions(body));
      case "/v1/analyze/spatial": return send(response, 200, await application.analyzeSpatial(body));
      case "/v1/references/resolve": return send(response, 200, await application.resolveReference(body));
      case "/v1/export/dxf": return send(response, 200, await application.exportDxf(body));
      case "/v1/source/analyze": return send(response, 200, analyzeSourceModule(body.id, body.sourceText));
      case "/v1/source/dependencies": return send(response, 200, { order: orderSourceModules(body.modules) });
      case "/v1/query": return send(response, 200, await application.query(body));
      case "/v1/query/geometry": return send(response, 200, await application.geometryQuery(body));
      default: return send(response, 404, { error: "not-found" });
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return send(response, 400, { error: "kernel-request-rejected", message });
  }
});
server.listen(port, "127.0.0.1", () => console.log(`UMLCAD Kernel API listening on http://127.0.0.1:${port}`));
