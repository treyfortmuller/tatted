# coldboot

As simple a flake-based Rust project I could muster, basically no batteries included but enough to start hacking.

```
nix develop

cargo run
```

Some notes:
* I use the `rust-overlay` at the moment
* `nixfmt-tree` is the nix formatter of choice
* There's some vscode settings assuming you're using `nixEnvSelector`, I'm not really a `direnv` guy

