export const theme = {
  header: (text: string) => `\x1b[1;36m${text}\x1b[0m`,
  accent: (text: string) => `\x1b[1;34m${text}\x1b[0m`,
  accentSoft: (text: string) => `\x1b[34m${text}\x1b[0m`,
  bold: (text: string) => `\x1b[1m${text}\x1b[0m`,
  dim: (text: string) => `\x1b[2m${text}\x1b[0m`,
  error: (text: string) => `\x1b[31m${text}\x1b[0m`,
  success: (text: string) => `\x1b[32m${text}\x1b[0m`,
};

export const editorTheme = {
  cursor: `\x1b[32m`,
  selection: `\x1b[46m\x1b[30m`,
  text: `\x1b[37m`,
  background: `\x1b[40m`,
};
