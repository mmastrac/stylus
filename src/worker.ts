import { readLines, BufReader } from "https://deno.land/std/io/bufio.ts";
import { delay } from "https://deno.land/std/async/delay.ts";
import { MonitorConfig } from "./config.ts";

const UTF8 = new TextDecoder("utf-8");

function updateStatus(
  status: {
    status: "green" | "yellow" | "red";
    code: number;
    description: string;
  },
  log: any[],
) {
  self.postMessage(
    { "message": "status", status: { status: status, log: log } },
  );
}

async function test(config: MonitorConfig) {
  const process = Deno.run({
    cwd: config.root,
    cmd: [config.script],
    env: { "TERM": "dumb", "LANG": "UTF-8", "LC_ALL": "UTF-8" },
    stderr: "piped",
    stdout: "piped",
    stdin: "null",
  });

  let timeout = delay(config.timeout);
  let stderr = process.stderr ? new BufReader(process.stderr) : null;
  let stdout = process.stdout ? new BufReader(process.stdout) : null;

  let output = [];
  let rl1, rl2;
  while (stderr || stdout) {
    const tuple = <T extends any[]>(...args: T): T => args;
    rl1 = rl1 ? rl1 : stderr?.readLine().then((r) => tuple(1, r));
    rl2 = rl2 ? rl2 : stdout?.readLine().then((r) => tuple(2, r));
    let res = await Promise.race([rl1, rl2, timeout].filter((e) => !!e));
    if (!res) {
      // Timeout
      break;
    }
    if (res[0] === 1) {
      const s = res[1];
      if (!s) {
        stderr = null;
      } else {
        output.push(tuple("ERROR", UTF8.decode(s.line)));
      }
      rl1 = null;
    } else if (res[0] === 2) {
      const s = res[1];
      if (!s) {
        stdout = null;
      } else {
        output.push(tuple("OUTPUT", UTF8.decode(s.line)));
      }
      rl2 = null;
    }
  }

  const result = await Promise.race([process.status(), timeout]);
  if (result === undefined) {
    // Timeout
    process.kill(1);
    const result = await Promise.race([process.status(), delay(5000)]);
    if (result === undefined) {
      // If we need to kill -9 a script, mark it red
      updateStatus(
        {
          status: "red",
          code: -1,
          description: "Process timed out and was forceably killed",
        },
        output,
      );
      process.kill(9);
    } else {
      updateStatus(
        {
          status: "yellow",
          code: -1,
          description: "Process timed out and was killed",
        },
        output,
      );
    }
  } else {
    // Script completed, read out the status
    updateStatus(
      {
        status: result.success ? "green" : "red",
        code: result.code,
        description: result.success
          ? "Success"
          : "Monitor script returned an error",
      },
      output,
    );
  }

  setTimeout(() => {
    test(config);
  }, config.interval);
}

async function init(config: MonitorConfig) {
  // Run the test initially as well
  test(config);
}

self.onmessage = async (e) => {
  if (e.data["message"] == "init") {
    try {
      init(<MonitorConfig> e.data["config"]);
    } catch (e) {
      console.log(e);
      self.postMessage({ error: e.toString() });
    }
  }

  if (e.data["message"] == "shutdown") {
    self.close();
  }
};
