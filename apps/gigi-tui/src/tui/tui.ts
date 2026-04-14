import { randomUUID } from "node:crypto";
import { P2PChatClient, type P2PConnectionOptions } from "./p2p-chat-client";
import { theme } from "./theme/theme";
import { formatTokens } from "./tui-formatters";
import type {
  AgentSummary,
  SessionInfo,
  SessionScope,
  TuiOptions,
  TuiStateAccess,
} from "./tui-types";

export function resolveTuiSessionKey(params: {
  raw?: string;
  sessionScope: SessionScope;
  currentAgentId: string;
  sessionMainKey: string;
}) {
  const trimmed = (params.raw ?? "").trim();
  if (!trimmed) {
    if (params.sessionScope === "global") {
      return "global";
    }
    return `agent:${params.currentAgentId}:${params.sessionMainKey}`;
  }
  if (trimmed === "global" || trimmed === "unknown") {
    return trimmed;
  }
  if (trimmed.startsWith("agent:")) {
    return trimmed;
  }
  return `agent:${params.currentAgentId}:${trimmed}`;
}

export function resolveGatewayDisconnectState(reason?: string): {
  connectionStatus: string;
  activityStatus: string;
  pairingHint?: string;
} {
  const reasonLabel = reason?.trim() ? reason.trim() : "closed";
  if (/pairing required/i.test(reasonLabel)) {
    return {
      connectionStatus: `gateway disconnected: ${reasonLabel}`,
      activityStatus: "pairing required: run openclaw devices list",
      pairingHint:
        "Pairing required. Run `openclaw devices list`, approve your request ID, then reconnect.",
    };
  }
  return {
    connectionStatus: `gateway disconnected: ${reasonLabel}`,
    activityStatus: "idle",
  };
}

export function isIgnorableTuiStopError(error: unknown): boolean {
  if (!error || typeof error !== "object") {
    return false;
  }
  const err = error as { code?: unknown; syscall?: unknown; message?: unknown };
  const code = typeof err.code === "string" ? err.code : "";
  const syscall = typeof err.syscall === "string" ? err.syscall : "";
  const message = typeof err.message === "string" ? err.message : "";
  if (code === "EBADF" && syscall === "setRawMode") {
    return true;
  }
  return /setRawMode/i.test(message) && /EBADF/i.test(message);
}

export function stopTuiSafely(stop: () => void): void {
  try {
    stop();
  } catch (error) {
    if (!isIgnorableTuiStopError(error)) {
      throw error;
    }
  }
}

type CtrlCAction = "clear" | "warn" | "exit";

export function resolveCtrlCAction(params: {
  hasInput: boolean;
  now: number;
  lastCtrlCAt: number;
  exitWindowMs?: number;
}): { action: CtrlCAction; nextLastCtrlCAt: number } {
  const exitWindowMs = Math.max(1, Math.floor(params.exitWindowMs ?? 1000));
  if (params.hasInput) {
    return {
      action: "clear",
      nextLastCtrlCAt: params.now,
    };
  }
  if (params.now - params.lastCtrlCAt <= exitWindowMs) {
    return {
      action: "exit",
      nextLastCtrlCAt: params.lastCtrlCAt,
    };
  }
  return {
    action: "warn",
    nextLastCtrlCAt: params.now,
  };
}

export async function runTui(opts: TuiOptions) {
  const initialSessionInput = (opts.session ?? "").trim();
  let sessionScope: SessionScope = "per-sender";
  let sessionMainKey = "main";
  let agentDefaultId = "default";
  let currentAgentId = agentDefaultId;
  let agents: AgentSummary[] = [];
  const agentNames = new Map<string, string>();
  let currentSessionKey = "";
  let initialSessionApplied = false;
  let currentSessionId: string | null = null;
  let activeChatRunId: string | null = null;
  let pendingOptimisticUserMessage = false;
  let historyLoaded = false;
  let isConnected = false;
  let wasDisconnected = false;
  let toolsExpanded = false;
  let showThinking = false;
  let pairingHintShown = false;
  const localRunIds = new Set<string>();
  const localBtwRunIds = new Set<string>();

  const deliverDefault = opts.deliver ?? false;
  const autoMessage = opts.message?.trim();
  let autoMessageSent = false;
  let sessionInfo: SessionInfo = {};
  let lastCtrlCAt = 0;
  let exitRequested = false;
  let activityStatus = "idle";
  let connectionStatus = "connecting";
  let statusTimeout: NodeJS.Timeout | null = null;

  const state: TuiStateAccess = {
    get agentDefaultId() {
      return agentDefaultId;
    },
    set agentDefaultId(value) {
      agentDefaultId = value;
    },
    get sessionMainKey() {
      return sessionMainKey;
    },
    set sessionMainKey(value) {
      sessionMainKey = value;
    },
    get sessionScope() {
      return sessionScope;
    },
    set sessionScope(value) {
      sessionScope = value;
    },
    get agents() {
      return agents;
    },
    set agents(value) {
      agents = value;
    },
    get currentAgentId() {
      return currentAgentId;
    },
    set currentAgentId(value) {
      currentAgentId = value;
    },
    get currentSessionKey() {
      return currentSessionKey;
    },
    set currentSessionKey(value) {
      currentSessionKey = value;
    },
    get currentSessionId() {
      return currentSessionId;
    },
    set currentSessionId(value) {
      currentSessionId = value;
    },
    get activeChatRunId() {
      return activeChatRunId;
    },
    set activeChatRunId(value) {
      activeChatRunId = value;
    },
    get pendingOptimisticUserMessage() {
      return pendingOptimisticUserMessage;
    },
    set pendingOptimisticUserMessage(value) {
      pendingOptimisticUserMessage = value;
    },
    get historyLoaded() {
      return historyLoaded;
    },
    set historyLoaded(value) {
      historyLoaded = value;
    },
    get sessionInfo() {
      return sessionInfo;
    },
    set sessionInfo(value) {
      sessionInfo = value;
    },
    get initialSessionApplied() {
      return initialSessionApplied;
    },
    set initialSessionApplied(value) {
      initialSessionApplied = value;
    },
    get isConnected() {
      return isConnected;
    },
    set isConnected(value) {
      isConnected = value;
    },
    get autoMessageSent() {
      return autoMessageSent;
    },
    set autoMessageSent(value) {
      autoMessageSent = value;
    },
    get toolsExpanded() {
      return toolsExpanded;
    },
    set toolsExpanded(value) {
      toolsExpanded = value;
    },
    get showThinking() {
      return showThinking;
    },
    set showThinking(value) {
      showThinking = value;
    },
    get connectionStatus() {
      return connectionStatus;
    },
    set connectionStatus(value) {
      connectionStatus = value;
    },
    get activityStatus() {
      return activityStatus;
    },
    set activityStatus(value) {
      activityStatus = value;
    },
    get statusTimeout() {
      return statusTimeout;
    },
    set statusTimeout(value) {
      statusTimeout = value;
    },
    get lastCtrlCAt() {
      return lastCtrlCAt;
    },
    set lastCtrlCAt(value) {
      lastCtrlCAt = value;
    },
  };

  const noteLocalRunId = (runId: string) => {
    if (!runId) {
      return;
    }
    localRunIds.add(runId);
    if (localRunIds.size > 200) {
      const [first] = localRunIds;
      if (first) {
        localRunIds.delete(first);
      }
    }
  };

  const forgetLocalRunId = (runId: string) => {
    localRunIds.delete(runId);
  };

  const isLocalRunId = (runId: string) => localRunIds.has(runId);

  const clearLocalRunIds = () => {
    localRunIds.clear();
  };

  const noteLocalBtwRunId = (runId: string) => {
    if (!runId) {
      return;
    }
    localBtwRunIds.add(runId);
    if (localBtwRunIds.size > 200) {
      const [first] = localBtwRunIds;
      if (first) {
        localBtwRunIds.delete(first);
      }
    }
  };

  const forgetLocalBtwRunId = (runId: string) => {
    localBtwRunIds.delete(runId);
  };

  const isLocalBtwRunId = (runId: string) => localBtwRunIds.has(runId);

  const clearLocalBtwRunIds = () => {
    localBtwRunIds.clear();
  };

  const p2pOptions: P2PConnectionOptions = {
    nickname: opts.nickname,
    host: opts.host,
    port: opts.port,
  };

  const client = await P2PChatClient.connect(p2pOptions);

  const formatSessionKey = (key: string) => {
    if (key === "global" || key === "unknown") {
      return key;
    }
    const parts = key.split(":");
    return parts.length > 2 ? parts.slice(2).join(":") : key;
  };

  const formatAgentLabel = (id: string) => {
    const name = agentNames.get(id);
    return name ? `${id} (${name})` : id;
  };

  const resolveSessionKey = (raw?: string) => {
    return resolveTuiSessionKey({
      raw,
      sessionScope,
      currentAgentId,
      sessionMainKey,
    });
  };

  currentSessionKey = resolveSessionKey(initialSessionInput);

  const updateHeader = () => {
    const sessionLabel = formatSessionKey(currentSessionKey);
    const agentLabel = formatAgentLabel(currentAgentId);
    console.log(theme.header(
      `gigi-tui - P2P - nickname ${opts.nickname} - agent ${agentLabel} - session ${sessionLabel}`,
    ));
  };

  const setConnectionStatus = (text: string, ttlMs?: number) => {
    connectionStatus = text;
    if (statusTimeout) {
      clearTimeout(statusTimeout);
    }
    if (ttlMs && ttlMs > 0) {
      statusTimeout = setTimeout(() => {
        connectionStatus = isConnected ? "connected" : "disconnected";
      }, ttlMs);
    }
  };

  const setActivityStatus = (text: string) => {
    activityStatus = text;
  };

  const updateFooter = () => {
    const sessionKeyLabel = formatSessionKey(currentSessionKey);
    const sessionLabel = sessionInfo.displayName
      ? `${sessionKeyLabel} (${sessionInfo.displayName})`
      : sessionKeyLabel;
    const agentLabel = formatAgentLabel(currentAgentId);
    const modelLabel = sessionInfo.model
      ? sessionInfo.modelProvider
        ? `${sessionInfo.modelProvider}/${sessionInfo.model}`
        : sessionInfo.model
      : "unknown";
    const tokens = formatTokens(sessionInfo.totalTokens ?? null, sessionInfo.contextTokens ?? null);
    const think = sessionInfo.thinkingLevel ?? "off";
    const fast = sessionInfo.fastMode === true;
    const verbose = sessionInfo.verboseLevel ?? "off";
    const reasoning = sessionInfo.reasoningLevel ?? "off";
    const reasoningLabel =
      reasoning === "on" ? "reasoning" : reasoning === "stream" ? "reasoning:stream" : null;
    const footerParts = [
      `agent ${agentLabel}`,
      `session ${sessionLabel}`,
      modelLabel,
      think !== "off" ? `think ${think}` : null,
      fast ? "fast" : null,
      verbose !== "off" ? `verbose ${verbose}` : null,
      reasoningLabel,
      tokens,
    ].filter(Boolean);
    console.log(theme.dim(footerParts.join(" | ")));
  };

  const displayStatus = () => {
    const text = activityStatus ? `${connectionStatus} | ${activityStatus}` : connectionStatus;
    console.log(theme.dim(text));
  };

  const refreshAgents = async () => {
    try {
      const result = await client.listAgents();
      agents = result.agents;
      agentDefaultId = result.defaultId;
      sessionMainKey = result.mainKey;
      sessionScope = result.scope as SessionScope;
      agents.forEach(agent => {
        if (agent.name) {
          agentNames.set(agent.id, agent.name);
        }
      });
    } catch (error) {
      console.error(theme.error(`Failed to refresh agents: ${String(error)}`));
    }
  };

  const refreshSessionInfo = async () => {
    try {
      const sessions = await client.listSessions({ includeGlobal: true });
      const session = sessions.sessions.find(s => s.key === currentSessionKey);
      if (session) {
        sessionInfo = {
          thinkingLevel: session.thinkingLevel,
          fastMode: session.fastMode,
          verboseLevel: session.verboseLevel,
          reasoningLevel: session.reasoningLevel,
          model: session.model,
          contextTokens: session.contextTokens,
          inputTokens: session.inputTokens,
          outputTokens: session.outputTokens,
          totalTokens: session.totalTokens,
          modelProvider: session.modelProvider,
          displayName: session.displayName,
        };
      }
    } catch (error) {
      console.error(theme.error(`Failed to refresh session info: ${String(error)}`));
    }
  };

  const loadHistory = async () => {
    try {
      const history = await client.loadHistory({
        sessionKey: currentSessionKey,
        limit: opts.historyLimit ? parseInt(opts.historyLimit.toString()) : 200,
      });
      // Display history
      if (Array.isArray(history.messages)) {
        history.messages.forEach((msg: any) => {
          if (msg.role === "user") {
            console.log(`You: ${msg.content}`);
          } else if (msg.role === "assistant") {
            console.log(`Assistant: ${msg.content}`);
          }
        });
      }
      historyLoaded = true;
    } catch (error) {
      console.error(theme.error(`Failed to load history: ${String(error)}`));
    }
  };

  const sendMessage = async (message: string) => {
    try {
      setActivityStatus("sending");
      displayStatus();
      const { runId } = await client.sendChat({
        sessionKey: currentSessionKey,
        message,
        deliver: deliverDefault,
        timeoutMs: opts.timeoutMs,
      });
      noteLocalRunId(runId);
      activeChatRunId = runId;
      setActivityStatus("waiting");
      displayStatus();
    } catch (error) {
      console.error(theme.error(`Failed to send message: ${String(error)}`));
      setActivityStatus("idle");
      displayStatus();
    }
  };

  const abortActive = async () => {
    if (activeChatRunId) {
      try {
        await client.abortChat({
          sessionKey: currentSessionKey,
          runId: activeChatRunId,
        });
        setActivityStatus("aborted");
        activeChatRunId = null;
      } catch (error) {
        console.error(theme.error(`Failed to abort chat: ${String(error)}`));
      }
    }
  };

  const handleChatEvent = (payload: any) => {
    if (payload.type === "message") {
      if (payload.role === "assistant") {
        console.log(`Assistant: ${payload.content}`);
        setActivityStatus("idle");
        // Only display status after the entire response is shown
        setTimeout(() => {
          displayStatus();
        }, 100);
      }
    } else if (payload.content) {
      // Handle direct content responses
      console.log(`Assistant: ${payload.content}`);
      setActivityStatus("idle");
      // Only display status after the entire response is shown
      setTimeout(() => {
        displayStatus();
      }, 100);
    } else if (payload.type === "error") {
      console.error(theme.error(`Error: ${payload.message}`));
      setActivityStatus("error");
      displayStatus();
    } else {
      // Log any other response format for debugging
      console.log(`Assistant: ${JSON.stringify(payload)}`);
      setActivityStatus("idle");
      // Only display status after the entire response is shown
      setTimeout(() => {
        displayStatus();
      }, 100);
    }
  };

  const handleAgentEvent = (payload: any) => {
    // Handle agent events if needed
  };

  const handleBtwEvent = (payload: any) => {
    // Handle side result events if needed
  };

  const requestExit = () => {
    if (exitRequested) {
      return;
    }
    exitRequested = true;
    client.stop();
    console.log("Disconnecting from P2P network...");
    process.exit(0);
  };

  const handleCommand = (command: string) => {
    // Handle slash commands
    const parts = command.split(" ");
    const cmd = parts[0].substring(1);
    const args = parts.slice(1);

    switch (cmd) {
      case "session":
        if (args.length > 0) {
          currentSessionKey = resolveSessionKey(args.join(" "));
          updateHeader();
          updateFooter();
          setActivityStatus(`switched to session ${currentSessionKey}`);
        }
        break;
      case "agent":
        if (args.length > 0) {
          currentAgentId = args[0];
          currentSessionKey = resolveSessionKey();
          updateHeader();
          updateFooter();
          setActivityStatus(`switched to agent ${currentAgentId}`);
        }
        break;
      case "clear":
        console.clear();
        updateHeader();
        updateFooter();
        break;
      case "exit":
        requestExit();
        break;
      default:
        console.log(theme.error(`Unknown command: ${cmd}`));
    }
  };

  client.onEvent = (evt) => {
    if (evt.event === "chat") {
      handleChatEvent(evt.payload);
    } else if (evt.event === "chat.side_result") {
      handleBtwEvent(evt.payload);
    } else if (evt.event === "agent") {
      handleAgentEvent(evt.payload);
    }
  };

  client.onConnected = () => {
    isConnected = true;
    pairingHintShown = false;
    const reconnected = wasDisconnected;
    wasDisconnected = false;
    setConnectionStatus("connected");
    void (async () => {
      await refreshAgents();
      updateHeader();
      await loadHistory();
      setConnectionStatus(reconnected ? "gateway reconnected" : "gateway connected", 4000);
      displayStatus();
      if (!autoMessageSent && autoMessage) {
        autoMessageSent = true;
        await sendMessage(autoMessage);
      }
      updateFooter();
    })();
  };

  client.onDisconnected = (reason) => {
    isConnected = false;
    wasDisconnected = true;
    historyLoaded = false;
    const disconnectState = resolveGatewayDisconnectState(reason);
    setConnectionStatus(disconnectState.connectionStatus, 5000);
    setActivityStatus(disconnectState.activityStatus);
    displayStatus();
    if (disconnectState.pairingHint && !pairingHintShown) {
      pairingHintShown = true;
      console.log(theme.error(disconnectState.pairingHint));
    }
    updateFooter();
  };

  updateHeader();
  setConnectionStatus("connecting");
  displayStatus();
  updateFooter();

  const handleCtrlC = () => {
    const now = Date.now();
    const decision = resolveCtrlCAction({
      hasInput: false,
      now,
      lastCtrlCAt,
    });
    lastCtrlCAt = decision.nextLastCtrlCAt;
    if (decision.action === "exit") {
      requestExit();
      return;
    }
    setActivityStatus("press ctrl+c again to exit");
  };

  // Set up signal handlers
  process.on("SIGINT", handleCtrlC);
  process.on("SIGTERM", requestExit);

  // Start reading user input
  console.log("\nType messages below (press Enter to send, / for commands):");
  process.stdin.on('data', (data) => {
    const input = data.toString().trim();
    if (input) {
      if (input.startsWith("/")) {
        handleCommand(input);
      } else {
        void sendMessage(input);
      }
    }
  });

  // Keep process running
  await new Promise<void>((resolve) => {
    process.once("exit", resolve);
  });
}
