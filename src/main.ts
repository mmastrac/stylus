import { Application, Router, send } from "https://deno.land/x/oak/mod.ts";
import { readConfig } from "./config.ts";
import { Monitor } from "./monitor.ts";

const config = await readConfig();
const monitor = await Monitor.create(config.monitorDir);

const router = new Router();
router
  .get("/config.json", (context) => {
    context.response.body = { server: config, monitor: monitor.status() };
  })
  .get("/style.css", (context) => {
    let css = `/* ${new Date()}*/\n`;
    monitor.status().forEach(m => {
      let monitor = m;
      css += `/* ${monitor.config.id} */\n`;
      try {
        css += eval("`" + config.cssSelectorFmt + "`") + "{" + eval("`" + config.cssStyleFmt + "`") + "}" + "\n";
      } catch (e) {
        css += "/* Error: " + e + "*/" + "\n";
      }
    });
    context.response.type = "text/css";
    context.response.body = css;
  });

const app = new Application();
app.use(router.routes());
app.use(router.allowedMethods());
app.use(async (context) => {
  await send(context, context.request.url.pathname, {
    root: config.staticDir,
    index: "index.html",
  });
});
await app.listen({ port: config.port });
