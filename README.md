# Bezy Font Editor

⚠️ This software is very raw, and is not yet suitable for use unless you want to learn the codebase.

Bezy is an open-source cross-platform font editor built with the [Bevy game engine](https://bevyengine.org/), the [Rust](https://www.rust-lang.org/) programming language, and various [Linebender Crates](https://linebender.org/). It is designed for simplicity, customization, user empowerment, education, and [AI-agent](.cursor/rules/bezy-app.mdc) assisted programming and type design.

![Bezy Font Editor Screenshot](docs/images/bezy-screenshot-005.png)
![Bezy Font Editor Screenshot](docs/images/bezy-screenshot-006.png)

A core design principle of this editor is user empowerment. Bezy aims to be the [Emacs](https://www.gnu.org/software/emacs/) of font editors. Users should be able to shape Bezy into a custom editor that perfectly fits their needs and aesthetics, like a calligrapher of the Arabic script making their own pens from simple reeds. Designers not being fully in control of their tools is a vulnerable position. Bezy wants to challenge the paternalism and lack of customizability prevalent in other font editing applications.

Post v1.0, Bezy will have built-in AI-agent functionality using the Font Garden and other models allowing for highly automated AI-assisted workflows without giving up fine-grained control and attention to detail. Users will be able to sign in with [Ethereum](https://ethereum.org/)/[Base](https://www.base.org/) and pay-as-they-go with stable coins and the $BEZY memecoin for remote model access, or run AI-agents locally and completely ignore the sign-in-with Ethereum features. This avoids subscription models used for AI-agent tools like [Cursor](https://www.cursor.com/) and allows us to build payments and monetization on top of free and open-source software.

The $BEZY memecoin is an experiment in diffuse value capture and incentive design for free and open-source software. We believe that top-down corporate funding in FOSS tends to create coordination failures and can work against incentives that encourage collaboration towards [commons based peer production](https://en.wikipedia.org/wiki/Commons-based_peer_production). A project memecoin allows users and contributors to diffusely capture value from an open source project with no bureaucracy and funding drama.

[UFO](https://unifiedfontobject.org/) is the current default source format. We believe that human readable source formats are a non-negotiable requirement for AI-agent assisted type design.

## Table of Contents
- [About Bezy](#about-bezy)
- [Bezy Design Principles](#bezy-design-principles)
- [Installation](#installation)
- [Using Bezy](#using-bezy)
- [Developing with Bezy](#developing-with-bezy)
- [Contributing](#contributing)
- [Community](#community)
- [License](#license)

## About Bezy

Bezy is loosely inspired by and ported from [Runebender](https://github.com/linebender/runebender), a previous font editor built with [Druid](https://github.com/linebender/druid), a data-first Rust-native UI design toolkit.

It is also a spiritual successor to the font editor [RoboFont](https://robofont.com/), specifically the RoboFont [design principles](https://robofont.com/documentation/topics/robofont-design-principles/). Bezy takes the RoboFont design principles further, believing designers in the AI vibe-coding era need to be fully, not partly, in control of their tools. Programming and the Unix CLI are basic literacy in the 21st century. The difference between a proprietary font editor and an editor like Bezy is like the difference between learning to play a real stringed guitar and playing the game Guitar Hero.

## Bezy Design Principles

Bezy, like the Rust programming language, is fundamentally about empowerment. We believe typeface designers and editors should be encouraged to understand and modify their tools.

The idea behind Bezy is to provide a sturdy framework where everyone can add their own functionalities, rather than a program with hundreds of (often little needed) functions. This allows competition between ideas and technologies.

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

Simply running `cargo run` will start Bezy with a default empty state.

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

## Developing with Bezy

If you're new to Rust and want to contribute to Bezy:

1. Familiarize yourself with Bevy (the game engine): https://bevyengine.org/
2. Learn about the UFO font format: https://unifiedfontobject.org/
3. Read through the codebase, starting with `src/lib.rs` and `src/app.rs`
4. Try making small modifications to understand how things work
5. PRs are welcome!

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
