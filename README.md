# jarrs
Very specific app for extracting elements from a json file which is one array of a lot of objects. This does not actually care if it is processing valid json.

I made this because I tech-debt-ed myself into a corner and now have to deal with 15GB-json files. Orjson and other python packages shit themselves whenever they try to parse the mess that i created.

## Usage
```bash
$ jarrs [OPTIONS] --target <TARGET> --out-dir <OUT_DIR>
Options:
  -t, --target <TARGET>
  -c, --chunksize <CHUNKSIZE>  [default: 536870912]
  -o, --out-dir <OUT_DIR>
  -h, --help                   Print help
  -V, --version                Print version
```

The input for this tool must look something like this:
```json
// in.json
[
  {
    "test": 1,
    "test1": 2
  },
  {
    "test": 1,
    "test2": 2
  },
  {
    "test": 1,
    "test3": 2
  },
  {
    "test": 1,
    "test4": 2
  }
]
```
If you use this exact case, and the command: 
```
$ jarrs --target <in.json> --out-dir out --chunksize 50
```
it will produce two output files in a directory called `out` containing the first and second half of `in.json`.
```json
// out/chunk_0.json
[
  {
    "test": 1,
    "test1": 2
  },
  {
    "test": 1,
    "test2": 2
  }
]
```

```json
// out/chunk_1.json
[
  {
    "test": 1,
    "test3": 2
  },
  {
    "test": 1,
    "test4": 2
  }
]
```

The size of each of the files will be somewhere around the ballpack of the chunk size. The tool actually only moves on to the next file if the file it's currently writing to is exceeded. This means that the chunks will be slightly larger than the given size, and at least the size of one object.



## Roadmap
- [ ] maybe add multithreading, but it's good enough for now.
