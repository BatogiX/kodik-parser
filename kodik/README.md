# kodik

ClI written in Rust for getting direct links to files from Kodik.

## Install
```sh
cargo install kodik
```

## Usage

### Example
```sh
./kodik
Usage: kodik [OPTIONS] [URL]...

Arguments:
  [URL]...                     Url(s) to parse

Options:
  -l, --lazy                   Outputs one by one (turns off parallelism)
  -p, --player <MEDIA-PLAYER>  Specify media player (implies --lazy)
  -v, --verbose                Use verbose output (-vv very verbose)
  -s, --silent                 Do not print log messages
  -q, --quality <QUALITY>      Specify video quality [possible values: 360, 480, 720] (default: 720)
  -h, --help                   Print help
```
```sh
kodik https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p
```
```sh
kodik https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p https://kodikplayer.com/video/115369/2eb2c698195c8a5020284d37dbc981a3/720p https://kodikplayer.com/video/93063/a520057b037a9d017ed53f9e4955ae85/720p
```

#### You can also pipe output in your favourite media player
```sh
kodik https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p | mpv --playlist=-
```
OR
```sh
kodik --player mpv https://kodikplayer.com/video/91873/060cab655974d46835b3f4405807acc2/720p 
```
