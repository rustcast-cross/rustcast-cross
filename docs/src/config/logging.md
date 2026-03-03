# Logger configuration

## Default

This is the only logger added by default

```toml
[log.stdout]
type = "stdout"
level = "INFO"
use_ansi = true
```

This prints all logs at the `INFO` level or above to stdout, with ansi colouring

## Format

The `log` key is a table of each logger. The keys are mostly irrelevant, it's really only used for
logging. The properties are

- `type` (`stdout` or `file`)  
  The logger type. `stdout` logs to stdout, while `file` logs to a provided location

- `level` (`ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE` case insensitively)
  The minimum level to log. The levels are (in order)

  1. `ERROR` Critical errors
  1. `WARN` General warnings
  1. `INFO` Relatively unimportant but nice info
  1. `DEBUG` Slightly noisier logs that are mostly useful for debugging
  1. `TRACE` *Really* noisy logs. You probably shouldn't ever print these to stdout.

  e.g. if you set it to `INFO`, it'll print logs with the levels `INFO`, `WARN`, and `ERROR`.

- `use_ansi` (`true` / `false`)
  If true, prints out logs with ANSI colours. You probably shouldn't use this when logging to
  files.

- `env_filter` (optional string)
  An envfilter to apply to the data. For more info, see [tracing_subscriber's docs]
  (https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)

- `path` (only when `type` is `file`)
  The path to output the logs to 