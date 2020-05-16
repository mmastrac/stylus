import initYaml, {
  parse as parseYaml,
} from "https://deno.land/x/yaml_wasm@0.1.9/index.js";
import { readFileStr } from "https://deno.land/std/fs/mod.ts";
import { parse as parseArgs } from "https://deno.land/std/flags/mod.ts";
import * as path from "https://deno.land/std/path/mod.ts";
import { Status } from "./status.ts";

await initYaml(undefined);

async function getConfigFile(): Promise<string> {
  const { args } = Deno;
  const parsed = parseArgs(args);
  if (parsed._ && parsed._[0]) {
    return path.resolve("" + parsed._[0]);
  }
  return path.resolve("config.yaml");
}

export class Config {
  public constructor(
    public port: number,
    public staticDir: string,
    public monitorDir: string,
    public css: CssConfig,
  ) {}
}

export class CssConfig {
  public constructor(
    public metadata: { [key in Status]: any },
    public rules: CssRuleConfig[],
  ) {}
}

export class CssRuleConfig {
  public constructor(public selectors: string, public declarations: string) {}
}

export async function readConfig(): Promise<Config> {
  const file = await getConfigFile();
  let configText: string;
  try {
    configText = await readFileStr(file);
  } catch (e) {
    throw new Error("Failed to read file: " + file + " (" + e + ")");
  }
  const configObj = parseYaml(configText, undefined);
  if (configObj.length !== 1) {
    throw new Error(
      "Invalid configuration object: expected one and only one root",
    );
  }
  const root = configObj[0];
  const port = root["server"]["port"] || 8000;
  const staticDir = path.resolve(
    path.dirname(file),
    root["server"]["static"] || "static",
  );
  const monitorDir = path.resolve(
    path.dirname(file),
    root["monitor"]["dir"] || "monitor.d",
  );
  const cssObj = root["css"];
  const rules: CssRuleConfig[] = [];
  cssObj["rules"].forEach((rule: any) => {
    const selectors = rule["selectors"] || "#${monitor.id}";
    const declarations = rule["declarations"] ||
      "background-color: ${monitor.status}";
    rules.push(new CssRuleConfig(selectors, declarations));
  });
  const metadataObj = cssObj["metadata"] || {};
  const metadata: any = {};
  metadata["red"] = metadataObj["red"] || {};
  metadata["yellow"] = metadataObj["yellow"] || {};
  metadata["green"] = metadataObj["green"] || {};
  const css = new CssConfig(metadata, rules);
  const config = new Config(port, staticDir, monitorDir, css);
  return config;
}

export class MonitorConfig {
  public constructor(
    public id: string,
    public interval: number,
    public timeout: number,
    public script: string,
    public root: string,
  ) {}
}

function parseInterval(dflt: number, s?: string): number {
  if (!s) {
    return dflt;
  }

  if (s.endsWith("s")) {
    return +s.slice(0, -1) * 1000;
  }

  throw new Error("Invalid interval: " + s);
}

export async function readMonitorConfig(
  dir: string,
  file: string,
): Promise<MonitorConfig> {
  let configText: string;
  try {
    configText = await readFileStr(file);
  } catch (e) {
    throw new Error("Failed to read file: " + file + " (" + e + ")");
  }
  const configObj = parseYaml(configText, undefined);
  if (configObj.length !== 1) {
    throw new Error(
      "Invalid configuration object: expected one and only one root",
    );
  }
  const id = configObj[0]["test"]["id"] || dir;
  const interval = parseInterval(60000, configObj[0]["test"]["interval"]);
  const timeout = parseInterval(30000, configObj[0]["test"]["timeout"]);
  const script = path.resolve(
    path.dirname(file),
    configObj[0]["test"]["script"] || "test.sh",
  );
  const root = path.dirname(script);
  const config = new MonitorConfig(id, interval, timeout, script, root);
  return config;
}
