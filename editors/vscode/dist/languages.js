"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.DOCUMENT_SELECTOR = exports.SUPPORTED_LANGUAGES = void 0;
exports.SUPPORTED_LANGUAGES = [
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
];
exports.DOCUMENT_SELECTOR = exports.SUPPORTED_LANGUAGES.map((language) => ({
    scheme: "file",
    language,
}));
//# sourceMappingURL=languages.js.map