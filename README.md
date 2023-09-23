# CopyCat
CopyCat is an pastebin service mainly for Minecraft servers written in Rust built on top of [mclog](https://github.com/quick-898/mclog).

Thanks to advanced [static and dynamic analyzer](https://github.com/quick-898/copycat/wiki/Analyzer) powered by [Rhai](https://rhai.rs/) embedded scripting language, you can easily get information about server and give users solution how to solve issues based on it.

## Motivation
After reading hundreds of Minecraft logs, I've seen *a lot* of repetetion that I wanted to automate. Unfortunately, all alternatives I could find didn't fit our needs, mainly because of the lack of flexibility in detecting errors and solutions for them.

That's why CopyCat has [dynamic analyzer](https://github.com/quick-898/copycat/wiki/Analyzer#Dynamic) which allows you to write custom detection scripts based on many factors like server info, plugin info, or log content itself.

## Features
### Static analyzer
Static analyzer gives you basic information about the log, this includes:
- [x] Highlighting - different highlighting for info/warn/error messages, also higlighting for specific messages without log level (usually hosting messages) is supported

  This works without 0 lines of JavaScript and supports different platform/hosting formats, so both logs with `INFO]:` and `INFO:` formats will be highlighted properly.
- [x] Server info (version, platform and its version)
- [x] Plugin info (if present, its version)
- [x] Port info - info about server (server, query, RCON), plugin and mod ports

### Dynamic analyzer
You can write custom detection scripts based on information provided by static analyzer. If you're curious how it works, check [/scripts/](/scripts/).
To completely understand dynamic analyzer, check [wiki](https://github.com/quick-898/copycat/wiki/Analyzer#Dynamic).

### API
Thanks to developer API you can easily integrate CopyCat with different platforms like Discord or hosting panel

### IP address hider
IP addresses are hidden but plugin versions that matches IP addresses not :tada:

### Fast
CopyCat is fast. Well.. fast enough. It can be faster in some areas but thanks to Rust, there was never a need to think about optimizing the speed.

## Installation
To learn how to run CopyCat check [wiki installation page](https://github.com/quick-898/copycat/wiki/Instalation).

## Special thanks
- [kyngs](https://github.com/kyngs) for the name idea
