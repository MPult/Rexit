<div align="center">

# Rexit

Reddit Brexit - Liberate your Reddit Chats. This tool will export your reddit chats into a plethora of formats

![version](https://img.shields.io/github/v/tag/mpult/rexit?color=orange)
![license](https://img.shields.io/github/license/mpult/rexit?color=blue)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/mpult/rexit?color=red)
[![Ubuntu-latest](https://github.com/MPult/Rexit/actions/workflows/Ubuntu-latest.yml/badge.svg)](https://github.com/MPult/Rexit/actions/workflows/Ubuntu-latest.yml)

</div>

Tool to export reddit chats into a varaity of open formats (CSV, JSON, TXT).

```
Export your Reddit Chats

Usage: rexit.exe --formats <FORMATS>

Options:
  -f, --formats <FORMATS>  The formats to export to. Options: csv,json,txt
  -h, --help               Print help
  -V, --version            Print version
```

## Usage

Currently you need to specify the formats and it will as for the Authorization bearer token.

```bash
$ rexit --formats csv,json,txt
Your Bearer Token: <YOUR BEARER TOKEN>
```
It will save the files to the current directory. For CSV and TXT it is split by room; for JSON it`s combined into one file.

