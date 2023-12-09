# bootleg-editor3000

## Description
This program is a CLI app that allows user read a CSV file of fixed dimensions (dimensions x,y) and operate on it.

These functionalities include:
1. Display entire file.
2. Paginate (display from row xa to xb).
3. Delete and modify a row/field.
4. Output struct to new csv file or update existing one.

## Usage
You can run the program by running the following command in the root directory of the project:
```bash
cargo run -- -h # or --help to see all the available options
```

- To paginate each record per page (default 10 records per page), you can run the following command:
```bash
cargo run testdata.csv -r 5 -dd
```

- To test the whole program, you can run the following command:
```bash
cargo test
```