# CopyCat
CopyCat is a pastebin service mainly for Minecraft servers written in Rust and built on top of [mclog](https://github.com/quick-898/mclog).

Thanks to the advanced [static and dynamic analyzers](https://github.com/quick-898/copycat/wiki/Analyzer) powered by [Rhai](https://rhai.rs/) embedded scripting language, you can easily get information about a server and give the users solutions on how to solve issues based on it.

## Motivation
After reading hundreds of Minecraft logs, I've seen *a lot* of repetition that I wanted to automate. Unfortunately, all alternatives that I found didn't fit our needs, mainly because of the lack of flexibility in detecting errors and solutions for them.

That's why CopyCat has a [dynamic analyzer](https://github.com/quick-898/copycat/wiki/Analyzer#Dynamic) which allows you to write custom detection scripts based on many factors like the server info, plugin info, or the log content itself.

## Features
### Static analyzer
Static analyzer gives you some basic information about the log, this includes:
- [x] Highlighting - different highlighting for info/warn/error messages, also higlighting for different specific messages without the log level (usually hosting messages) is supported.

This works with 0 lines of JavaScript and supports different platform/hosting formats, so both logs with `INFO]:` and `INFO:` formats will be highlighted properly.
- [x] Server info (version, platform and its version)
- [x] Plugin info (if present, its version)
- [x] Port info - info about the server (server, query, RCON), plugin and mod ports.

### Dynamic analyzer
You can write custom detection scripts based on information that is provided by a static analyzer. If you're curious about how it works, check [/scripts/](/scripts/).
To completely understand the dynamic analyzer, check [wiki](https://github.com/quick-898/copycat/wiki/Analyzer#Dynamic).

### API
Thanks to the developer API, you can easily integrate CopyCat with different platforms like Discord or a hosting panel.

### IP address hider
IP addresses are hidden but plugin or server related versions that matches IP addresses are not :tada: .

### Fast
CopyCat is fast. Well.. fast enough. It can be faster in some areas but thanks to Rust, there was never a need to think about optimizing the speed.

## JSON and YAML support
CopyCat also support Y(A)ML and JSON files so configuration files can be displayed properly.

## Installation
To learn how to run CopyCat check: [wiki installation page](https://github.com/quick-898/copycat/wiki/Instalation).

## TODO
- Log entry type in script with closure (callback) support

## Special thanks
- [kyngs](https://github.com/kyngs) for the name idea.
