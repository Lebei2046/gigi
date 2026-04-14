export type SessionScope = "per-sender" | "global" | "per-agent";

export type ResponseUsageMode = "hidden" | "detailed" | "summary";

export type TuiOptions = {
  nickname: string;
  session?: string;
  deliver?: boolean;
  message?: string;
  thinking?: string;
  timeoutMs?: number;
  historyLimit?: number;
  host?: string;
  port?: number;
};

export type SessionInfo = {
  thinkingLevel?: string;
  fastMode?: boolean;
  verboseLevel?: string;
  reasoningLevel?: string;
  model?: string;
  contextTokens?: number;
  inputTokens?: number;
  outputTokens?: number;
  totalTokens?: number;
  modelProvider?: string;
  displayName?: string;
  responseUsage?: ResponseUsageMode;
};

export type AgentSummary = {
  id: string;
  name?: string;
};

export type TuiStateAccess = {
  agentDefaultId: string;
  sessionMainKey: string;
  sessionScope: SessionScope;
  agents: AgentSummary[];
  currentAgentId: string;
  currentSessionKey: string;
  currentSessionId: string | null;
  activeChatRunId: string | null;
  pendingOptimisticUserMessage: boolean;
  historyLoaded: boolean;
  sessionInfo: SessionInfo;
  initialSessionApplied: boolean;
  isConnected: boolean;
  autoMessageSent: boolean;
  toolsExpanded: boolean;
  showThinking: boolean;
  connectionStatus: string;
  activityStatus: string;
  statusTimeout: NodeJS.Timeout | null;
  lastCtrlCAt: number;
};
