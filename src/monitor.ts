import * as path from "https://deno.land/std/path/mod.ts";
import { MonitorConfig, readMonitorConfig } from "./config.ts";

class TestWorker {
  constructor(public worker: Worker, public config: MonitorConfig, public status: string) {}
}

export class MonitorStatus {
  constructor(public config: MonitorConfig, public status: string) {}
}

export class Monitor {
  #workers: TestWorker[] = [];

  private constructor(readonly dir: string) {
  }

  public static async create(dir: string): Promise<Monitor> {
    const m = new Monitor(dir);
    return m.refresh().then(_ => m);
  }

  public status(): MonitorStatus[] {
    return this.#workers.map(w => new MonitorStatus(w.config, w.status));
  }

  async monitor(id: string, name: string) {
    const config = await readMonitorConfig(id, name);
    const worker = new Worker("./worker.ts", { type: "module", deno: true });
    const testWorker = new TestWorker(worker, config, "yellow");
    worker.postMessage({ message: "init", config: config });
    worker.onmessage = (e) => {
      if (e.data["message"] === "status") {
        testWorker.status = e.data["status"]["status"];
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
          await this.monitor(folder.name, name);
        }
      } catch (e) {
        console.log(e);
        continue;
      }
    }
  }
}
