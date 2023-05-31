# Acolors [Experimental] [Read-only Mirror]
Proxy Profile Manager Server with gRPC API

## Build
```
git clone https://github.com/ArkToria/Acolors
cd Acolors
cargo build --release
```
## Usage
```
    acolors serve [OPTIONS]

OPTIONS:
    -c, --config <config>          Config path (default: "./config/acolors.json")
    -d, --dbpath <dbpath>          Database path (default: "./config/acolors.db")
    -h, --help                     Print help information
    -i, --interface <interface>    Interface to bind on (default: 127.0.0.1)
    -k, --corepath <corepath>      Core path (default: "v2ray")
    -p, --port <port>              Which port to use (default: 19198)
    -t, --corename <corename>      Core name (default: "v2ray")
```
## Build Status

### Linux

[![ArchLinux Package Build Test](https://github.com/ArkToria/Acolors/actions/workflows/arch-build.yaml/badge.svg?branch=master)](https://github.com/ArkToria/Acolors/actions/workflows/arch-build.yaml)

[![Linux Run Build and Tests](https://github.com/ArkToria/Acolors/actions/workflows/linux-build.yaml/badge.svg)](https://github.com/ArkToria/Acolors/actions/workflows/linux-build.yaml)

### Windows

[![Windows Run Build and Tests](https://github.com/ArkToria/Acolors/actions/workflows/windows-build.yaml/badge.svg)](https://github.com/ArkToria/Acolors/actions/workflows/windows-build.yaml)

## Config Example
```json
{
  "inbounds": {
    "socks5": {
      "enable": true,
      "listen": "127.0.0.1",
      "port": 4444,
      "udp_enable": true
    },
    "http": {
      "enable": true,
      "listen": "127.0.0.1",
      "port": 4445
    }
  },
  "cores": [
    {
      "tag": "v2tag",
      "name": "v2ray",
      "path": "/usr/bin/v2ray"
    },
    {
      "tag": "sstag",
      "name": "shadowsocks",
      "path": "/usr/bin/sslocal-rust"
    },
    {
      "tag": "trojantag",
      "name": "trojan",
      "path": "/usr/bin/trojan"
    },
    {
      "tag": "trojan-gotag",
      "name": "trojan-go",
      "path": "/usr/bin/trojan-go"
    },
    {
      "tag": "v2rayv5",
      "name": "v2ray",
      "path": "~/compile/v2rayv5/v2ray"
    },
    {
      "tag": "naive",
      "name": "naiveproxy",
      "path": "/usr/bin/naiveproxy"
    }
  ]
}
```
## API
[Acolors/proto/acolors.proto](https://github.com/ArkToria/Acolors/blob/master/proto/acolors.proto)