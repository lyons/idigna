> The twins, Idigna and Buranun, have the body of a snake.
> They desired to travel far.
> This desire took the form of the Twin Statue and its gate.
> The Twin Statue calls forth the gate's power.
> The hero Hermes used the statues to release the twins.
>
> *La-Mulana, Twin Labyrinths, D-1*

## What

Idigna is a little server for the [Gemini protocol](https://gemini.circumlunar.space/) that supports defining regex-based rules for rewriting URIs, setting up redirects, and autoindexing folders.

## Why

There's something charmingly quixotic about a protocol that has more server implementations than active users. Also I needed a toy project for learning asynchronous Rust.

## How

`cargo build --release` to buid the executable. The executable expects a configuration file (default `config.json`) in its working directory, as well as an SSL cert/key pair at a location specified by the config file. Files are served from the folder specified in `server_root` in the config file.

See `config-and-init/config.json` and `config-and-init/idigna.service` for example configuration and init scripts.
