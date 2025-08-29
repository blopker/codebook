# Development Guide for Codebook VS Code Extension

This guide will help you set up your development environment and contribute to the Codebook VS Code extension.

## Prerequisites

### Required Tools

1. **Node.js** (v18 or later)
   - Download from [nodejs.org](https://nodejs.org/)
   - Verify installation: `node --version` and `npm --version`

2. **Visual Studio Code**
   - Download from [code.visualstudio.com](https://code.visualstudio.com/)

3. **Codebook LSP Server**
   - Install via cargo: `cargo install codebook-lsp`
   - Or build from source:
     ```bash
     cd ../crates/codebook-lsp
     cargo build --release
     # Binary will be in ../../target/release/codebook-lsp
     ```

4. **Git**
   - For version control and contributing

### Optional Tools

- **Rust** (if building codebook-lsp from source)
  - Install from [rustup.rs](https://rustup.rs/)

## Setting Up the Development Environment

### 1. Clone the Repository

```bash
git clone https://github.com/blopker/codebook.git
cd codebook/vscode-extension
```

### 2. Install Dependencies

```bash
npm install
```

Or use the build script:
```bash
./build.sh install
```

### 3. Verify Server Installation

```bash
./build.sh check-server
```

If the server is not found, install it:
```bash
cargo install codebook-lsp
```

## Development Workflow

### Building the Extension

```bash
# One-time build
npm run compile

# Or use the build script
./build.sh build
```

### Watch Mode (Auto-compile on changes)

```bash
npm run watch

# Or
./build.sh watch
```

### Running the Extension

#### Method 1: VS Code Launch Configuration (Recommended)

1. Open the `vscode-extension` folder in VS Code
2. Press `F5` or go to Run → Start Debugging
3. A new VS Code window will open with the extension loaded
4. Open a code file to test spell checking

#### Method 2: Build Script

```bash
./build.sh dev
```

This will:
- Compile the extension
- Open VS Code in extension development mode
- Start the TypeScript compiler in watch mode

### Testing

#### Run Unit Tests

```bash
npm test

# Or
./build.sh test
```

#### Manual Testing Checklist

When testing changes, verify:

- [ ] Extension activates without errors
- [ ] Language server starts successfully
- [ ] Spell checking works in various file types
- [ ] Commands work (add to dictionary, restart server)
- [ ] Configuration changes take effect
- [ ] Error handling for missing server
- [ ] Performance with large files

### Debugging

#### Extension Debugging

1. Set breakpoints in `src/extension.ts`
2. Press `F5` to start debugging
3. Use the Debug Console to inspect variables

#### Language Server Debugging

To debug the language server itself:

1. Set the log level to debug:
   ```json
   "codebook.logLevel": "debug"
   ```

2. View the output:
   - Command Palette → "Codebook: Show Output Channel"
   - Or Output panel → Select "Codebook" from dropdown

#### Common Issues and Solutions

**Extension not activating:**
- Check the "Extension Host" output for errors
- Ensure you're testing with a supported file type
- Verify the extension is enabled in the test window

**Language server not starting:**
- Check if codebook-lsp is in PATH: `which codebook-lsp`
- Verify server path in settings: `codebook.serverPath`
- Check the Codebook output channel for errors

**No spell checking:**
- Verify the file type is supported
- Check if the extension is enabled: `codebook.enable`
- Look for errors in the output channel

## Code Structure

```
vscode-extension/
├── src/
│   ├── extension.ts          # Main extension entry point
│   └── test/
│       ├── runTest.ts        # Test runner
│       └── suite/
│           ├── index.ts      # Test suite setup
│           └── extension.test.ts # Extension tests
├── .vscode/
│   ├── launch.json          # Debug configurations
│   └── tasks.json           # Build tasks
├── package.json             # Extension manifest
├── tsconfig.json            # TypeScript configuration
└── README.md               # User documentation
```

### Key Files

- **`src/extension.ts`**: Main extension logic, LSP client setup
- **`package.json`**: Extension metadata, commands, configuration schema
- **`tsconfig.json`**: TypeScript compiler options

## Making Changes

### Adding a New Command

1. Define the command in `package.json`:
   ```json
   {
     "command": "codebook.myNewCommand",
     "title": "My New Command",
     "category": "Codebook"
   }
   ```

2. Register the command in `src/extension.ts`:
   ```typescript
   const myCommand = vscode.commands.registerCommand(
     'codebook.myNewCommand',
     async () => {
       // Command implementation
     }
   );
   context.subscriptions.push(myCommand);
   ```

### Adding a Configuration Option

1. Add to `package.json` under `contributes.configuration.properties`:
   ```json
   "codebook.myOption": {
     "type": "string",
     "default": "value",
     "description": "My option description"
   }
   ```

2. Use in `src/extension.ts`:
   ```typescript
   const config = vscode.workspace.getConfiguration('codebook');
   const myOption = config.get<string>('myOption', 'default');
   ```

### Modifying Language Support

Edit the `activationEvents` in `package.json` to add/remove language support:
```json
"activationEvents": [
  "onLanguage:newlanguage"
]
```

## Packaging and Distribution

### Create a VSIX Package

```bash
npm run package

# Or
./build.sh package
```

This creates a `.vsix` file that can be:
- Installed locally: `code --install-extension codebook-*.vsix`
- Shared with others
- Published to the marketplace

### Publishing to VS Code Marketplace

1. Get a Personal Access Token from [Azure DevOps](https://dev.azure.com/)

2. Login to vsce:
   ```bash
   vsce login <publisher-name>
   ```

3. Publish:
   ```bash
   npm run publish

   # Or
   ./build.sh publish
   ```

## Testing Strategies

### Unit Tests

Located in `src/test/suite/`:
- Test command registration
- Configuration validation
- Error handling

### Integration Tests

Test the extension with the actual language server:
- Create test files with known spelling errors
- Verify diagnostics are reported correctly
- Test quick fixes and code actions

### Performance Testing

For large files or projects:
- Monitor memory usage
- Check response times
- Verify no blocking operations

## Contributing Guidelines

### Before Submitting a PR

1. **Run tests**: `npm test`
2. **Run linter**: `npm run lint`
3. **Test manually**: Verify your changes work as expected
4. **Update documentation**: If adding features or changing behavior
5. **Update CHANGELOG.md**: Document your changes

### Code Style

- Use TypeScript strict mode
- Follow existing code patterns
- Add JSDoc comments for public functions
- Use meaningful variable names
- Keep functions small and focused

### Commit Messages

Follow conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `test:` Test changes
- `refactor:` Code refactoring
- `chore:` Maintenance tasks

Example: `feat: add support for custom dictionary paths`

## Resources

- [VS Code Extension API](https://code.visualstudio.com/api)
- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/)
- [vscode-languageclient Documentation](https://github.com/microsoft/vscode-languageserver-node)
- [Codebook Repository](https://github.com/blopker/codebook)

## Getting Help

- Open an issue on [GitHub](https://github.com/blopker/codebook/issues)
- Check existing issues for similar problems
- Include extension version, VS Code version, and OS in bug reports
- Provide minimal reproduction steps when reporting bugs

## Advanced Topics

### Custom Language Server Arguments

Modify server arguments in `src/extension.ts`:
```typescript
const serverOptions: ServerOptions = {
  run: {
    command: serverPath,
    args: ['--custom-arg'],
    transport: TransportKind.stdio
  }
};
```

### Conditional Activation

Control when the extension activates:
```typescript
export async function activate(context: vscode.ExtensionContext) {
  // Check conditions before starting
  if (!shouldActivate()) {
    return;
  }
  // ... rest of activation
}
```

### Custom Diagnostics

Enhance diagnostics with additional information:
```typescript
client.onReady().then(() => {
  client.onNotification('custom/diagnostic', (params) => {
    // Handle custom diagnostics
  });
});
```

## Troubleshooting Development Issues

### npm install fails

- Clear npm cache: `npm cache clean --force`
- Delete `node_modules` and `package-lock.json`, then reinstall

### TypeScript compilation errors

- Ensure TypeScript version matches: `npm install typescript@latest`
- Check `tsconfig.json` for correct settings
- Verify all type definitions are installed

### Extension not loading in debug mode

- Check VS Code version compatibility
- Verify no syntax errors in `package.json`
- Look for errors in Extension Host output

### Build script permission denied

```bash
chmod +x build.sh
```

## Version Management

When releasing a new version:

1. Update version in `package.json`
2. Update `CHANGELOG.md` with release notes
3. Create a git tag: `git tag v0.1.0`
4. Build and test the package
5. Publish to marketplace

## License

This extension is MIT licensed. See the [LICENSE](LICENSE) file for details.