# dsc (Data Serialization Language Converter)

This is a tool to convert data from one format to another `blazingly fast`.  
It uses [serde](https://crates.io/crates/serde), [serde_json](https://crates.io/crates/serde_json) and [serde_yaml](https://crates.io/crates/serde_yaml), [toml](https://crates.io/crates/toml) and [quick-xml](https://crates.io/crates/quick-xml) to do the heavy lifting.  
Null values are not supported in toml, at least for now.

## Examples

```ps1
dsc --help # to see all the options
```

Reading from stdin and writing to a file:

```ps1
cat input.json | dsc --from json out.yaml # target format is inferred from the file extension 
```

Writing to stdout and reading from a file:

```ps1
dsc input.toml --to xml > out.xml # source format is inferred from the file extension 
```

Reading from stdin and writing to stdout:

```ps1
# here target format needs to be explicitly specified
curl -s https://api.github.com/users/lucascompython | dsc --from json --to yaml | cat 
```

Reading from a file and writing to another file:

```ps1
# the -o flags enables whitespace removal and the -r flag sets the root tag for xml
dsc input.toml out.xml -r roottag -o # formats are inferred from the file extensions
```
