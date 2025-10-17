# 0.5.0
- fix: inaccurate channel reporting when closed
- feat: added `Channel::close` and `Receiver::close`

# 0.4.0
- `no_std` support :)

# 0.3.0
- `Sender` has been renamed to `Channel`.
- `trotcast::channel` has been removed. use `Channel::new` instead.
- implement `Error` for the returned error types

# 0.2.0
- `channel` now only returns a `Sender`.
- added example for reconnecting
