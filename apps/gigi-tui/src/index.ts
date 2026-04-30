#!/usr/bin/env node
import { Command } from "commander";
import { runTui } from "./tui/tui";

const program = new Command();

program
  .name("gigi-tui")
  .description("Gigi P2P Terminal UI")
  .version("1.0.0")
  .option("--nickname <name>", "Nickname for the client node", "gigi-tui")
  .option("--session <key>", 'Session key (default: "main", or "global" when scope is global)')
  .option("--deliver", "Deliver assistant replies", false)
  .option("--thinking <level>", "Thinking level override")
  .option("--message <text>", "Send an initial message after connecting")
  .option("--timeout-ms <ms>", "Agent timeout in ms")
  .option("--history-limit <n>", "History entries to load", "200")
  .option("--host <host>", "P2P bootstrap host")
  .option("--port <port>", "P2P bootstrap port")
  .action(async (opts) => {
    try {
      await runTui({
        nickname: opts.nickname,
        session: opts.session,
        deliver: opts.deliver,
        thinking: opts.thinking,
        message: opts.message,
        timeoutMs: opts.timeoutMs ? parseInt(opts.timeoutMs) : undefined,
        historyLimit: opts.historyLimit ? parseInt(opts.historyLimit) : undefined,
        host: opts.host,
        port: opts.port ? parseInt(opts.port) : undefined,
      });
    } catch (err) {
      console.error(String(err));
      process.exit(1);
    }
  });

// Parse command line arguments
program.parse(process.argv);
