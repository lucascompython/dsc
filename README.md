# dsc (Data Serialization Converter)

This is a tool to convert data from one format to another.  
It uses [serde](https://crates.io/crates/serde), [serde_json](https://crates.io/crates/serde_json) and [serde_yaml](https://crates.io/crates/serde_yaml), [toml](https://crates.io/crates/toml) and [quick-xml](https://crates.io/crates/quick-xml) to do the heavy lifting.  

## Example

```bash
dsc --help
```

Input (input.json):  

```json
{
    "menu": {
        "id": "file",
        "value": "File",
        "popup": {
            "menuitem": [
                {
                    "value": "New",
                    "onclick": "CreateNewDoc()"
                },
                {
                    "value": "Open",
                    "onclick": "OpenDoc()"
                },
                {
                    "value": "Close",
                    "onclick": "CloseDoc()"
                }
            ]
        }
    }
}
```

Run:

```bash
dsc input.json output.yaml
```

Output (output.yaml):  

```yaml
menu:
  id: file
  popup:
    menuitem:
    - onclick: CreateNewDoc()
      value: New
    - onclick: OpenDoc()
      value: Open
    - onclick: CloseDoc()
      value: Close
  value: File
```
