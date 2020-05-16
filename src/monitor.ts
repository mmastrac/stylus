import * as path from "https://deno.land/std/path/mod.ts";
import { Config, MonitorConfig, readMonitorConfig } from "./config.ts";
import { Status } from "./status.ts";

class TestWorker {
  constructor(
    public worker: Worker,
    public config: MonitorConfig,
    public status: StatusResult,
  ) {}
}

export class StatusResult {
  constructor(
    public status: Status,
    public code: number,
    public description: string,
    public metadata: any,
  ) {}

  public static default(): StatusResult {
    return {
      status: "yellow",
      code: -1,
      description: "Unknown",
      metadata: null,
    };
  }
}

export class MonitorStatus {
  constructor(public config: MonitorConfig, public status: StatusResult) {}
}

export class Monitor {
  #workers: TestWorker[] = [];

  private constructor(readonly config: Config, readonly dir: string) {
  }

  public static async create(config: Config): Promise<Monitor> {
    const m = new Monitor(config, config.monitorDir);
    return m.refresh().then((_) => m);
  }

  public status(): MonitorStatus[] {
    return this.#workers.map((w) => new MonitorStatus(w.config, w.status));
  }

  async monitor(cfg: Config, id: string, name: string) {
    const config = await readMonitorConfig(id, name);
    const worker = new Worker("./worker.ts", { type: "module", deno: true });
    const testWorker = new TestWorker(worker, config, StatusResult.default());
    testWorker.status.metadata = cfg.css.metadata[testWorker.status.status];
    worker.postMessage({ message: "init", config: config });
    worker.onmessage = (e) => {
      if (e.data["message"] === "status") {
        testWorker.status = e.data["status"]["status"];
        testWorker.status.metadata = cfg.css.metadata[testWorker.status.status];
      }
    };
    this.#workers.push(testWorker);
  }

  async refresh() {
    this.#workers.forEach((w) => {
      w.worker.postMessage({ message: "shutdown" });
    });
    this.#workers = [];
    const folders = await Deno.readDir(this.dir);
    for await (const folder of folders) {
      if (!folder.isDirectory) {
        continue;
      }

      const name = path.resolve(this.dir, folder.name, "config.yaml");
      try {
        const stat = await Deno.lstat(name);
        if (stat.isFile) {
          await this.monitor(this.config, folder.name, name);
        }
      } catch (e) {
        console.log(e);
        continue;
      }
    }
  }
}
