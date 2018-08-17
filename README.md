# mysqldiff
A MySQL database structure compare tool implemented in Rust

## Usage
``` shell
$ # cargo run <imitator_uri> <template_uri>
$ # ie
$ cargo run mysql://root:zaq1xsw2@localhost:3306/imitator mysql://root:zaq1xsw2@localhost:3306/template
```