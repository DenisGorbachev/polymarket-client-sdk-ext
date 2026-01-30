# Merge the CacheDownloadCommand and CacheTestCommand

## Files

* src/command/cache_test_command.rs
* src/command/cache_download_command.rs

## Tasks

* Move the round-trip check to CacheDownloadCommand
  * Note that you don't need to deserialize from fjall::Slice, since you already have an `input: T` (deserialized earlier by the client)
* Remove CacheTestCommand
