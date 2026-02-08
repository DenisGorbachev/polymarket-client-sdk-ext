# Try to switch to postcard

## Tasks

* Find the usages of `bitcode`
* Switch to `postcard`

## Tests

* Run the following commands:
  ```shell
  cargo run --quiet -- cache download --offset 0 --page-limit 3
  cargo run --quiet -- cache check
  ```
* If you need to clear the database, use `rm -r .cache/db`
