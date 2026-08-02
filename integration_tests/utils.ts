function getLanguageFromFileName(fileName: string): string {
  // Extract the extension from the filename
  const extension = fileName.split('.').pop()?.toLowerCase() || '';

  // Map of file extensions to language IDs
  const extensionToLanguage: { [key: string]: string } = {
    // Programming Languages
    'ts': 'typescript',
    'js': 'javascript',
    'py': 'python',
    'java': 'java',
    'cpp': 'cpp',
    'c': 'c',
    'cs': 'csharp',
    'rb': 'ruby',
    'php': 'php',
    'go': 'go',
    'rs': 'rust',
    'swift': 'swift',
    'kt': 'kotlin',

    // Web Technologies
    'html': 'html',
    'htm': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'sass',
    'jsx': 'javascriptreact',
    'tsx': 'typescriptreact',
    'vue': 'vue',

    // Data Formats
    'json': 'json',
    'xml': 'xml',
    'yaml': 'yaml',
    'yml': 'yaml',
    'md': 'markdown',

    // Shell Scripts
    'sh': 'shell',
    'bash': 'shell',
    'ps1': 'powershell',

    // Other
    'sql': 'sql',
    'dockerfile': 'dockerfile',
    'txt': 'plaintext'
  };

  // Return the language ID if found, otherwise return 'plaintext'
  return extensionToLanguage[extension] || 'plaintext';
}

import type { LSPTestClient } from "./client";

interface Diagnostic {
  message: string;
  range: unknown;
}

interface PublishDiagnosticsParams {
  uri: string;
  diagnostics: Diagnostic[];
}

/**
 * Resolve with the first publishDiagnostics matching the filters. Dictionary
 * downloads happen in the background, so a check against a cold cache
 * legitimately publishes empty diagnostics first and re-publishes once the
 * dictionaries land — callers asserting on content should pass
 * `nonEmpty: true` rather than grabbing the first event.
 */
function waitForDiagnostics(
  client: LSPTestClient,
  options: { uri?: string; nonEmpty?: boolean; timeoutMs?: number } = {},
): Promise<PublishDiagnosticsParams> {
  const { uri, nonEmpty = false, timeoutMs = 30000 } = options;
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      client.removeListener("textDocument/publishDiagnostics", listener);
      reject(
        new Error(
          `Timeout waiting for ${nonEmpty ? "non-empty " : ""}diagnostics${uri ? ` for ${uri}` : ""}`,
        ),
      );
    }, timeoutMs);

    const listener = (params: PublishDiagnosticsParams) => {
      if (uri && params.uri !== uri) return;
      if (nonEmpty && params.diagnostics.length === 0) return;
      clearTimeout(timeoutId);
      client.removeListener("textDocument/publishDiagnostics", listener);
      resolve(params);
    };
    client.on("textDocument/publishDiagnostics", listener);
  });
}

export { getLanguageFromFileName, waitForDiagnostics };
export type { PublishDiagnosticsParams };
