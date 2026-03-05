<p align="center">
  <img src="assets/banner-crab-salad.png" alt="sweetgreen-rs banner" width="100%" />
</p>

<h1 align="center">sweetgreen-rs</h1>

<p align="center">
  <em>Unofficial salad CLI.</em>
</p>

<p align="center">
  <code>cargo install --git https://github.com/LK/sweetgreen-rs --bin sg</code> &nbsp;·&nbsp; <code>sg --help</code>
</p>

## Install

From this repo:

```bash
cargo install --git https://github.com/LK/sweetgreen-rs --bin sg
```

That installs the binary as:

```bash
sg
```

## Quick Start

```bash
# check auth/cart status
sg auth status
sg cart view

# browse menu
sg menu restaurants --query "downtown brooklyn"
sg menu products --restaurant-id 2104 --search harvest --include-ingredients
sg menu product --restaurant-id 2104 --name "harvest bowl"
sg menu ingredient --restaurant-id 2104 --name "corn salsa"

# add by names (restaurant id can be inferred from active cart)
sg cart add-by-name --product-name "harvest bowl" --removals apples
```

## Authentication Modes

There are two ways to run `sg`.

### 1) Browser Session Mode (recommended)

Use this if you want CLI changes to show up in your normal Sweetgreen web session.

```bash
sg auth import-browser-cookies --browser arc --domain sweetgreen.com
```

### 2) Local Token Mode (OTP)

Use this if you want a fully CLI-driven login flow.

```bash
sg auth login --email you@example.com
```

You will be prompted for the one-time code sent to email.  
`sg` stores local auth state in `~/.sweetgreen/` by default.

## Intended Usage

The typical pattern is:

1. Authenticate once (usually browser session mode).
2. Use `sg menu ...` to resolve products and ingredient IDs.
3. Use `sg cart ...` commands to add, remove, and adjust items.
4. Review and check out in the Sweetgreen web/app experience.

This works well for personal scripts and for agent-driven flows (for example, Slack or chat assistants calling `sg` under the hood).

## Useful Commands

```bash
sg auth status
sg auth refresh
sg auth logout

sg menu restaurants --query "11201"
sg cart view
sg cart count
sg cart wanted-times
sg cart add --product-id <id> --quantity 1
sg cart add-by-name --product-name "chicken caesar wrap" --additions "corn salsa"
sg cart update --line-item-id <id> --product-id <id> --quantity 2
sg cart remove --line-item-id <id>
sg cart clear
```

## Notes

- The CLI writes debug reports to `~/.sweetgreen/debug/` when requests fail.
- This is an unofficial client and may break if Sweetgreen changes private APIs.
