declare module '*.md' {
  import type React from 'react'

  const ReactComponent: React.VFC
  interface MarkdownAttributes {
    attributes?: Record<string, string>
    toc?: Array<{ id: string; text: string; level: number }>
    html?: string
  }
  const attributes: MarkdownAttributes
  const toc: MarkdownAttributes['toc']
  const html: MarkdownAttributes['html']
  export { ReactComponent, attributes, toc, html }
}
