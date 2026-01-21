# fwdp

A simple and efficient TCP traffic forwarding tool written in Rust.

## Usage

```bash
revp <target_address> -L <listen_address>
```

### Parameters

- `target_address`: The destination address to forward traffic to (format: `ip:port`)
- `-L, --listen`: The address and port to listen on

### Listen Address Format

The listen address can be specified in two ways:

1. **Port only**: `8080` - Listens on all interfaces (0.0.0.0:8080)
2. **IP and port**: `127.0.0.1:8080` - Listens on specific interface

## Examples

Forward traffic from local port 8080 to a remote server:

```bash
revp 1.2.3.4:5678 -L 8080
```

This will:

- Listen on `0.0.0.0:8080`
- Forward all incoming connections to `1.2.3.4:5678`

## License

Copyright (c) Cnily03. All rights reserved.

Licensed under the [MIT License](LICENSE).
