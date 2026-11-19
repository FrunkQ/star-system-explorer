// A tiny OpenAI-compatible provider for tests: /v1/models and /v1/chat/completions,
// with CORS for the browser test. Records every request it sees (path and authorization).
import { createServer } from "node:http";

export function mockProvider({ key = null, priced = false, context = 4096 } = {}) {
  const seen = [];
  const server = createServer((req, res) => {
    const cors = { "access-control-allow-origin": "*", "access-control-allow-headers": "authorization, content-type, x-title", "access-control-allow-methods": "GET, POST, OPTIONS" };
    if (req.method === "OPTIONS") return void (res.writeHead(204, cors), res.end());
    let body = "";
    req.on("data", (c) => (body += c));
    req.on("end", () => {
      seen.push({ path: req.url, authorization: req.headers.authorization ?? null, body });
      if (key && req.headers.authorization !== `Bearer ${key}`) return void (res.writeHead(401, cors), res.end("{}"));
      res.writeHead(200, { ...cors, "content-type": "application/json" });
      if (req.url.endsWith("/models")) {
        const m = { id: "mock/small", context_length: context, ...(priced ? { pricing: { prompt: "0.000001", completion: "0.000002" } } : {}) };
        return void res.end(JSON.stringify({ data: [m, { id: "mock/large" }] }));
      }
      const q = JSON.parse(body || "{}");
      const last = q.messages?.[q.messages.length - 1]?.content ?? "";
      res.end(JSON.stringify({ choices: [{ message: { content: `Mock answer to: ${last}` } }], usage: { prompt_tokens: 120, completion_tokens: 12 } }));
    });
  });
  return new Promise((resolve) =>
    server.listen(0, "127.0.0.1", () => {
      const { port } = server.address();
      resolve({ base: `http://127.0.0.1:${port}/v1`, seen, close: () => new Promise((r) => server.close(r)) });
    }),
  );
}
