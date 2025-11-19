export const SUPPORTED_LANGUAGES = [
  "c",
  "cpp",
  "css",
  "elixir",
  "go",
  "html",
  "haskell",
  "java",
  "javascript",
  "latex",
  "lua",
  "markdown",
  "php",
  "plaintext",
  "python",
  "ruby",
  "rust",
  "toml",
  "typescript",
  "typst",
  "zig",
  "csharp"
] as const;

export const DOCUMENT_SELECTOR = SUPPORTED_LANGUAGES.map((language) => ({
  scheme: "file",
  language,
}));
