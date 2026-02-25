# User Guide

## Installation

```shell
cargo install mdbook-pagecrypt
```

## Usage

Add the following to your `book.toml`:

```toml
[output.pagecrypt]
password = "secret"
rounds = 600_000 # optional, rounds in password hashing
```

And then run `mdbook build` to encrypt the site.

## Configuration

- `password`: The password to encrypt the site with. **Required.**
- `rounds`: The number of rounds to use for password hashing. Default is 600_000. Bigger numbers are safer but slower.

### HTML Configuration

You can also configure HTML rendering options under `[output.pagecrypt]`. Any option valid for `[output.html]` can be used:

```toml
[output.pagecrypt]
password = "secret"
default-theme = "ayu"
preferred-dark-theme = "ayu"

[output.pagecrypt.fold]
enable = true
level = 1
```

See the [mdBook documentation](https://rust-lang.github.io/mdBook/format/configuration/renderers.html#html-renderer-options) for all available HTML options.

## Security

The encryption is powered by AES symmetric encryption and salted password hashing. It is hard to crack using brute force.

- Encryption is performed with the [AES-GCM](https://crates.io/crates/aes-gcm) crate.
- Password hashing is performed with the [pbkdf2](https://crates.io/crates/pbkdf2) crate.
