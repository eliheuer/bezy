# Bezy Font Editor

⚠️ This software is very raw, and is not yet suitable for use unless you want to learn the codebase. But it is getting close and PRs are welcome!

Bezy is an open-source cross-platform font editor built with the [Bevy game engine](https://bevyengine.org/), the [Rust](https://www.rust-lang.org/) programming language, and various [Linebender Crates](https://linebender.org/) like [Norad](https://github.com/linebender/norad), and [Kurbo](https://github.com/linebender/kurbo). It is designed for simplicity, customization, user empowerment, education, and [AI-agent](.cursor/rules/bezy-app.mdc) assisted programming and type design.

![Bezy Font Editor Screenshot](docs/images/bezy-screenshot-005.png)
![Bezy Font Editor Screenshot](docs/images/bezy-screenshot-006.png)

A core design principle of this editor is user empowerment. Bezy aims to be the [Emacs](https://www.gnu.org/software/emacs/) of font editors. Users should be able to shape Bezy into a custom editor that perfectly fits their needs and aesthetics, like a calligrapher of the Arabic script making their own pens from simple reeds.

Post v1.0, Bezy will have built-in AI-agent functionality using Font Garden and other models allowing for highly automated AI-assisted workflows without giving up fine-grained control and attention to detail.

[UFO](https://unifiedfontobject.org/) is the current default source format.

## Table of Contents
- [About Bezy](#about-bezy)
- [Bezy Design Principles](#bezy-design-principles)
- [Installation](#installation)
- [Using Bezy](#using-bezy)
  - [Basic Usage](#basic-usage)
  - [Command Line Arguments](#command-line-arguments)
  - [The Bezy Grotesk Test Font](#the-bezy-grotesk-test-font)
  - [Hotkeys](#hotkeys)
- [Developing with Bezy](#developing-with-bezy)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [Community](#community)
- [License](#license)

## About Bezy

Bezy is loosely inspired by and ported from [Runebender](https://github.com/linebender/runebender), a previous font editor built with [Druid](https://github.com/linebender/druid), a data-first Rust-native UI design toolkit. It uses many of the same crates like [Norad](https://github.com/linebender/norad), and [Kurbo](https://github.com/linebender/kurbo). 

It is also a spiritual successor to the font editor [RoboFont](https://robofont.com/), specifically the RoboFont [design principles](https://robofont.com/documentation/topics/robofont-design-principles/).

## Bezy Design Principles

Bezy, like the Rust programming language, is fundamentally about empowerment. We believe typeface designers and editors should be encouraged to understand and modify their tools. The idea behind Bezy is to provide a sturdy framework where everyone can add their own functionalities.

## Installation

### Prerequisites:
- Rust (1.75.0 or later)
- Cargo (included with Rust)

### Steps:

1. **Install Rust** (if you haven't already):
   ```bash
   # For macOS, Linux, or WSL
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Follow the on-screen instructions and restart your terminal
   ```
   Or visit https://www.rust-lang.org/tools/install for other platforms

2. **Clone the repository**:
   ```bash
   git clone https://github.com/eliheuer/bezy.git
   cd bezy
   ```

3. **Build and run**:
   ```bash
   cargo run
   ```

## WASM Build (for Web Browsers)

Bezy can also be built as a WebAssembly (WASM) application to run in web browsers. This is useful for web deployment or sharing your font editor in a browser environment.

### Prerequisites for WASM:
- Rust with the `wasm32-unknown-unknown` target
- `wasm-server-runner` for serving the WASM build

### Steps:

1. **Install the WASM target** (if you haven't already):
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. **Install wasm-server-runner**:
   ```bash
   cargo install wasm-server-runner
   ```

3. **Build and run the WASM version**:
   
   **Option A: Use the provided build script** (recommended):
   ```bash
   ./build-wasm.sh
   ```
   
   **Option B: Build manually**:
   ```bash
   cargo run --target wasm32-unknown-unknown
   ```

4. **Access the application**:
   - The WASM build will automatically open in your default browser
   - Alternatively, navigate to the URL shown in the terminal (typically `http://127.0.0.1:1334`)

### WASM Limitations:
- File system access is limited (no `--load-ufo` support currently)
- Some desktop-specific features may not be available
- Performance may differ from the native desktop version

The WASM build is perfect for:
- Sharing your work online
- Demonstrations and tutorials
- Web-based font editing workflows
- Deployment to platforms like itch.io

### GitHub Pages Deployment

For production deployment to your own domain, see the [GitHub Pages Deployment Guide](docs/github-pages-deployment.md). The repository includes automated GitHub Actions for deploying to GitHub Pages with custom domain support.

Quick deployment steps:
1. Enable GitHub Pages in your repository settings
2. Configure DNS for your domain  
3. Push to main branch - deployment is automatic!

## Using Bezy

### Basic Usage

Simply running `cargo run` will start Bezy with a default UFO (Bezy Grotesk Regular Latin/Arabic) and the lowercase a glyph (U+0061).

### Command Line Arguments

Bezy supports several command line arguments to customize its behavior:

#### Loading a UFO Font File

```bash
cargo run -- --load-ufo <PATH_TO_UFO>
```

Example:
```bash
cargo run -- --load-ufo assets/fonts/bezy-grotesk-regular.ufo
```

#### Testing with Specific Unicode Characters

To test with a specific Unicode character:

```bash
cargo run -- --test-unicode <HEXADECIMAL_CODEPOINT>
```

Example (displays lowercase 'a'):
```bash
cargo run -- --test-unicode 0061
```

#### Combining Arguments

You can combine multiple arguments:

```bash
cargo run -- --load-ufo assets/fonts/bezy-grotesk-regular.ufo --test-unicode 0061
```

This loads the Bezy Grotesk font and displays the lowercase 'a' character.

#### Debug Mode

Enable additional debug information:

```bash
cargo run -- --debug
```

You can also control the logging verbosity using the `--log-level` option:

```bash
cargo run -- --log-level debug    # Show debug logs
cargo run -- --log-level info     # Show only info logs (default)
cargo run -- --log-level warn     # Show only warnings and errors
```
### The Bezy Grotesk Test Font

Bezy comes with a test font called "Bezy Grotesk" located in the `assets/fonts` directory. This is a UFO font format file that you can use to explore the editor's capabilities.

#### Key Features of Bezy Grotesk:
- Basic Latin characters
- Numbers and punctuation
- Special characters for the editor's UI
- Arabic script characters

To explore the font's structure, you can examine the UFO directory:
```
assets/fonts/bezy-grotesk-regular.ufo/
```

### Hotkeys

Many of Bezy's settings can be customized by editing the `src/settings.rs` file. For example, nudge amounts (2, 8, and 32 units) can be modified to better suit your workflow preferences. This aligns with Bezy's core philosophy of user empowerment and customization.

> ⚠️ **Pre-v1.0 Alpha Notice**: In the current pre-release state, many features in Bezy are only accessible via keyboard shortcuts. UI buttons for all features will be implemented before v1.0.

#### Selection and Navigation
- **Arrow Keys**: Nudge selected points (by 2 unit)
- **Shift + Arrow Keys**: Nudge selected points (by 8 units)
- **Cmd/Ctrl + Arrow Keys**: Nudge selected points (by 32 units)
- **Shift + +**: Switch to next codepoint
- **Shift + -**: Switch to previous codepoint
- **Shift + Click**: Add to selection (multi-select)
- **Click + Drag**: Box selection
- **Shift + Click + Drag**: Add to selection with box selection
- **Esc**: Deselect all

#### Edit Operations
- **Cmd/Ctrl + Z**: Undo
- **Cmd/Ctrl + Shift + Z** or **Cmd/Ctrl + Y**: Redo
- **Cmd/Ctrl + S**: Save font
- **Delete/Backspace**: Delete selected points

#### View Controls
- **Middle Mouse/Space + Drag**: Pan the view
- **Mouse Wheel/Trackpad Gesture**: Zoom in/out
- **Cmd/Ctrl + Plus (+)**: Zoom in
- **Cmd/Ctrl + Minus (-)**: Zoom out
- **T**: Toggle zoom-to-cursor behavior

These hotkeys are based on the current implementation in the codebase and may evolve as development continues toward v1.0.

## Developing with Bezy

If you're new to Rust and want to contribute to Bezy:

1. Familiarize yourself with Bevy (the game engine): https://bevyengine.org/
2. Learn about the UFO font format: https://unifiedfontobject.org/
3. Read through the codebase, starting with `src/lib.rs` and `src/app.rs`
4. Try making small modifications to understand how things work
5. PRs are welcome!

### Adding Custom Tools

Bezy features a dynamic toolbar system that makes it incredibly easy to add new editing tools. The system uses Bevy's plugin architecture and requires minimal code changes.

#### Quick Start: Adding a Tool

1. **Create your tool file** (`src/ui/toolbars/edit_mode_toolbar/my_tool.rs`):
   ```rust
   use bevy::prelude::*;
   use crate::ui::toolbars::edit_mode_toolbar::{EditTool, ToolRegistry};

   pub struct MyTool;

   impl EditTool for MyTool {
       fn id(&self) -> crate::ui::toolbars::edit_mode_toolbar::ToolId {
           "my_tool"
       }
       
       fn name(&self) -> &'static str {
           "My Tool"
       }
       
       fn icon(&self) -> &'static str {
           "\u{E019}"  // Unicode icon
       }
       
       fn default_order(&self) -> i32 {
           50  // Lower numbers appear first
       }
       
       fn update(&self, commands: &mut Commands) {
           // Tool behavior while active
       }
   }

   pub struct MyToolPlugin;

   impl Plugin for MyToolPlugin {
       fn build(&self, app: &mut App) {
           app.add_systems(Startup, register_my_tool);
       }
   }

   fn register_my_tool(mut tool_registry: ResMut<ToolRegistry>) {
       tool_registry.register_tool(Box::new(MyTool));
   }
   ```

2. **Add module declaration** to `src/ui/toolbars/edit_mode_toolbar/mod.rs`:
   ```rust
   mod my_tool;
   pub use my_tool::MyToolPlugin;
   ```

3. **Register the plugin** in your app:
   ```rust
   app.add_plugins(MyToolPlugin);
   ```

That's it! Your tool automatically appears in the toolbar with proper ordering and functionality.

#### Advanced Features

- **Custom ordering**: Control tool order with `default_order()` or `ToolOrdering` resource
- **Keyboard shortcuts**: Add shortcuts with `shortcut_key()`
- **Lifecycle management**: Use `on_enter()` and `on_exit()` for setup/cleanup
- **Temporary modes**: Support temporary activation (like spacebar for pan)

For detailed documentation, see `src/ui/toolbars/edit_mode_toolbar/USAGE.md`.

## Documentation

Bezy has developer documentation available in the `docs/` directory:

- [Logger System](docs/logger.md) - Information about Bezy's custom logging system
- [GitHub Pages Deployment](docs/github-pages-deployment.md) - Complete guide for deploying to GitHub Pages with custom domains

## Contributing

Contributions to Bezy are welcome! Here's how you can help:

### Getting Started
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Contribution Guidelines
- Follow the existing code style (just run cargo fmt mostly)
- Write clear commit messages
- Add tests if needed
- Discuss major changes in an issue before implementation

## Community

Join the Bezy community:

- **GitHub Discussions**: For feature requests, questions, and general discussion
- **Issues Tracker**: For bugs and specific improvement suggestions

We're in the process of setting up additional community channels - check back soon!

## License

This project is licensed under the GPL.

The GNU General Public License is a free, [copyleft](https://en.wikipedia.org/wiki/Copyleft) license for software and other kinds of works.

The licenses for most software and other practical works are designed to take away your freedom to share and change the works. By contrast, the GNU General Public License is intended to guarantee your freedom to share and change all versions of a program--to make sure it remains free software for all its users. We, the Free Software Foundation, use the GNU General Public License for most of our software; it applies also to any other work released this way by its authors. You can apply it to your programs, too.

When we speak of free software, we are referring to freedom, not price. Our General Public Licenses are designed to make sure that you have the freedom to distribute copies of free software (and charge for them if you wish), that you receive source code or can get it if you want it, that you can change the software or use pieces of it in new free programs, and that you know you can do these things.

To protect your rights, we need to prevent others from denying you these rights or asking you to surrender the rights. Therefore, you have certain responsibilities if you distribute copies of the software, or if you modify it: responsibilities to respect the freedom of others.

For example, if you distribute copies of such a program, whether gratis or for a fee, you must pass on to the recipients the same freedoms that you received. You must make sure that they, too, receive or can get the source code. And you must show them these terms so they know their rights.
