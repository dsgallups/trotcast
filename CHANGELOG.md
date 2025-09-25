# 0.3.0
- `Sender` has been renamed to `Channel`.
- `trotcast::channel` has been removed. use `Channel::new` instead.
- implement `Error` for the returned error types

# 0.2.0
- `channel` now only returns a `Sender`.
- added example for reconnecting
