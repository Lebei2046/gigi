export function formatTokens(total?: number | null, context?: number | null): string | null {
  if (total === null || total === undefined) {
    return null;
  }
  const totalStr = total.toLocaleString();
  if (context === null || context === undefined) {
    return `tokens ${totalStr}`;
  }
  const usedPct = Math.round((total / context) * 100);
  return `tokens ${totalStr}/${context.toLocaleString()} (${usedPct}%)`;
}

export function resolveFinalAssistantText(text: string): string {
  return text.replace(/\n/g, "\n  ");
}
