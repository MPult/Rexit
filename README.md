<div align="center">

# Rexit

Reddit Brexit - Liberate your Reddit Chats. This tool will export your Reddit chats into a plethora of formats

![version](https://img.shields.io/github/v/tag/mpult/rexit?color=orange)
![license](https://img.shields.io/github/license/mpult/rexit?color=blue)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/mpult/rexit?color=red)
[![Ubuntu-latest](https://github.com/MPult/Rexit/actions/workflows/Ubuntu-latest.yml/badge.svg)](https://github.com/MPult/Rexit/actions/workflows/Ubuntu-latest.yml)

</div>

Tool to export Reddit chats into a variety of open formats (CSV, JSON, TXT).

```
Export your Reddit Chats

Usage: rexit.exe [OPTIONS] --formats <FORMATS>

Options:
  -f, --formats <FORMATS>  The formats to export to. Options: csv,json,txt
  -t, --token              To use the bearer token flow, instead of username and password
  -h, --help               Print help
  -V, --version            Print version
```

## Usage

Currently, you need to specify the formats, and it will ask for the username and password (or bearer token with that auth flow).

```bash
$ rexit --formats csv,json,txt
> Your Reddit Username: <USERNAME>
> Your Reddit Password: <PASSWORD>
```
It will save the files to the current directory. For CSV and TXT it is split by room; for JSON it's combined into one file. If an image (.jpg, .gif, .png, etc.) was sent the matrix URL (`mxc://<serverName>/<ID>`) will be displayed as the message content. 

## Installation
You can use the files provided in the releases' page of this repository, or install via cargo.

```BASH
$ cargo install rexit
```

## Contributing
To keep the docs focused on the user experience the contributing and technical docs were implemented through cargo doc.

To access these:
```bash
$ cargo doc --open
```

In general all contributions are welcome. I would appreciate if you'd create an issue beforehand, in order for me to plan things out nicely.
## License
[GNU General Public License, Version 3](./LICENSE)
