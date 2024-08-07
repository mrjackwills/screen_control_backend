<p align="center">
 <img src='./.github/logo.svg' width='125px'/>
</p>

<h1 align="center">screen control backend client</h1>

<p align="center">
	See the frontend site source <a href='https://www.github.com/mrjackwills/screen_control_frontend' target='_blank' rel='noopener noreferrer'>here</a>
</p>

<p align="center">
	Built in <a href='https://www.rust-lang.org/' target='_blank' rel='noopener noreferrer'>Rust</a>.
</p>

## Required services

1) <a href='https://www.staticpi.com/' target='_blank' rel='noopener noreferrer'>staticPi</a> - the simple and secure messaging service

Suggested locations for directories required by screen_control

| directory             | reason                      |
| --------------------- | --------------------------- |
| ```~/screen_control.d/``` | Location of the application |

Files that are required by push alarm
| file            | reason                  |
| --------------- | ----------------------- |
| ```./.env```    | environmental variables |


## Required Envs

Envs that are used by `screen_control`
| name               | description         | required |
| ------------------ | ------------------- | :------: |
| `WS_ADDRESS`       | WS server URL       | ✓        |
| `WS_APIKEY`        | WS API key          | ✓        |
| `WS_PASSWORD`      | WS API password     | ✓        |
| `WS_TOKEN_ADDRESS` | WS token-server URL | ✓        |
| `LOG_LEVEL`        | Log level to print  | ❌       |


## Arguments
Command line arguments that are used by `screen_control`
| name    | description                              |
| ------- | ---------------------------------------- |
| `--on`  | Turn screen on                           |
| `--off` | Turn screen off                          |
| `-i`    | Attempt to install the systemd service   |
| `-u`    | Attempt to uninstall the systemd service |
| `-h`    | Show the help screen                     |


## Download

See <a href="https://github.com/mrjackwills/screen_control_backend/releases" target='_blank' rel='noopener noreferrer'>releases</a>

*One should always verify <a href='https://github.com/mrjackwills/screen_control_backend/blob/main/download.sh' target='_blank' rel='noopener noreferrer'>script content</a> before running in a shell*

```shell
curl https://raw.githubusercontent.com/mrjackwills/screen_control_backend/main/download.sh | bash
```

## Run

use ```./screen_control # Optional Cli Argument ```

## Tests

```bash
cargo test
```

## Build step

### x86_64

```shell
cargo build --release
```

### Cross platform
### Using Docker
requires <a href='https://www.docker.com/' target='_blank' rel='noopener noreferrer'>Docker</a> & <a href='https://github.com/cross-rs/cross' target='_blank' rel='noopener noreferrer'>cross-rs</a>


#### 64bit arm (pi 4, pi zero w 2)

```shell
cross build --target aarch64-unknown-linux-musl --release
```

#### 32bit pi (pi zero w)

```shell
cross build --target arm-unknown-linux-musleabihf --release
```
**Untested on other platforms**