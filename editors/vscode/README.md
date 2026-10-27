# Ridu Language Support for Visual Studio Code

<p align="center">
  <img src="icon.png" width="160" alt="Ridu Logo" />
</p>

Official Visual Studio Code extension providing language support for the **Ridu Programming Language** (`.ridu`).

## Features
- **Syntax Highlighting**: Comprehensive TextMate grammar for keywords (`val`, `var`, `fun`, `print`, `println`, `if`, `else`, `while`), literals, types, operators, and comments.
- **Language Configuration**: Automatic bracket matching, smart indenting, comment toggling (`//` and `/* ... */`), and quote pairs.
- **Code Snippets**: Quick templates for variable declarations (`val`, `var`, `valt`, `vart`), functions (`fun`, `main`), conditionals (`if`, `ifelse`), loops (`while`), and print statements (`println`, `print`).
- **File Association**: Automatic association with `.ridu` source files.

## Installation

### From Source
1. Copy the `editors/vscode` folder into your VS Code extensions directory:
   ```bash
   cp -r editors/vscode ~/.vscode/extensions/ratanazen.ridu-vscode-0.1.0
   ```
2. Restart or reload Visual Studio Code.
