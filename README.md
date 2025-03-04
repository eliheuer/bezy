# Bezy Font Editor

Bezy is an open-source cross-platform font editor built with the Bevy game engine and Rust. It is designed for simplicity, customizability, user empowerment and learning, and AI-agent assisted vibe-coding. 

![Bezy Font Editor Screenshot](docs/images/bezy-screenshot-005.png)
![Bezy Font Editor Screenshot](docs/images/bezy-screenshot-006.png)

## About Bezy

Bezy is loosely inspired by Runebender, a previous font editor built with the Druid GUI framework.

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
   cargo run -- --load-ufo assets/fonts/bezy-grotesk-regular.ufo --test-unicode 0061
   ```

## Using Bezy

### Basic Usage

Simply running `cargo run` will start Bezy with a default empty state. The UI includes:

- Edit mode toolbar (top left): Select, Pen, Hyper, Knife, Pan, Measure, Primitives, and Text tools
- Main editing area (center): Where font glyphs are displayed and edited
- Debug information (bottom): Font metrics and other details

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

This is particularly useful when debugging font metrics and other technical details.

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

## Edit Modes

Bezy offers several edit modes accessible from the toolbar:

- **Select**: Select and manipulate points and paths
- **Pen**: Draw Bézier curves
- **Hyper**: Advanced curve editing tool
- **Knife**: Cut existing paths
- **Pan**: Navigate the viewport
- **Measure**: Measure distances and angles
- **Primitives**: Create basic shapes
- **Text**: Add and edit text annotations

## Developing with Bezy

If you're new to Rust and want to contribute to Bezy:

1. Familiarize yourself with Bevy (the game engine): https://bevyengine.org/
2. Learn about the UFO font format: https://unifiedfontobject.org/
3. Read through the codebase, starting with `src/lib.rs` and `src/app.rs`
4. Try making small modifications to understand how things work

## License

This project is licensed under the GPL.

```
  The licenses for most software and other practical works are designed
to take away your freedom to share and change the works.  By contrast,
the GNU General Public License is intended to guarantee your freedom to
share and change all versions of a program--to make sure it remains free
software for all its users.  We, the Free Software Foundation, use the
GNU General Public License for most of our software; it applies also to
any other work released this way by its authors.  You can apply it to
your programs, too.

  When we speak of free software, we are referring to freedom, not
price.  Our General Public Licenses are designed to make sure that you
have the freedom to distribute copies of free software (and charge for
them if you wish), that you receive source code or can get it if you
want it, that you can change the software or use pieces of it in new
free programs, and that you know you can do these things.

  To protect your rights, we need to prevent others from denying you
these rights or asking you to surrender the rights.  Therefore, you have
certain responsibilities if you distribute copies of the software, or if
you modify it: responsibilities to respect the freedom of others.

  For example, if you distribute copies of such a program, whether
gratis or for a fee, you must pass on to the recipients the same
freedoms that you received.  You must make sure that they, too, receive
or can get the source code.  And you must show them these terms so they
know their rights.
```


