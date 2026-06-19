# Configuration Files

_Miasma_ can be configured via a configuration file rather than specifying each setting as a CLI flag.

```sh
miasma -f <path_to_config_file>
```

The following file formats are supported:

- **YAML** - [example](./config.yaml)
- **TOML** - [example](./config.toml)
- **JSON** - [example](./config.json)

All fields are optional and can be overridden by providing CLI flags.
