---
name: sg-manage-cart
description: Manage Sweetgreen carts using the local `sg` CLI. Use when asked to find restaurants, inspect menu data, resolve product or ingredient IDs, add/update/remove cart items, check auth/cart state, or troubleshoot Sweetgreen CLI cart operations.
---

# SG Manage Cart

Use this skill to operate Sweetgreen cart flows through `sg` on the local machine.

## Workflow

1. Verify CLI availability and auth state.
2. Resolve restaurant and menu information.
3. Apply cart mutations.
4. Re-check cart state and report exact outcomes.

## Step 1: Verify CLI and Session

Run:

```bash
command -v sg
sg auth status
```

Prefer browser-cookie mode when cart changes should match the web checkout session:

```bash
sg auth import-browser-cookies --browser arc --domain sweetgreen.com
```

Use OTP mode when browser import is unavailable:

```bash
sg auth login --email <email>
```

## Step 2: Resolve IDs and Menu Details

Find restaurant IDs:

```bash
sg menu restaurants --query "<city|neighborhood|zip|name>"
```

Inspect products and ingredients:

```bash
sg menu products --restaurant-id <id> --search "<text>" --include-ingredients
sg menu product --restaurant-id <id> --name "<product name>"
sg menu ingredient --restaurant-id <id> --name "<ingredient name>"
```

## Step 3: Mutate Cart

Prefer name-based cart edits:

```bash
sg cart add-by-name --product-name "<product>" --custom-name "<user>'s <base item name>" --additions "<ingredient>" --removals "<ingredient>"
```

Use ID-based add/update when explicit IDs are provided:

```bash
sg cart add --product-id <product_id> --quantity 1 --custom-name "<user>'s <base item name>" --additions <ingredient_id>
sg cart update --line-item-id <line_item_id> --product-id <product_id> --quantity <n> --custom-name "<user>'s <base item name>"
sg cart remove --line-item-id <line_item_id>
sg cart clear
```

## Step 4: Verify and Report

Always verify final cart state:

```bash
sg cart view
sg cart count
```

Summarize:
- Matched restaurant/product/ingredient IDs
- Actions completed
- Final cart line items. Include totals only when explicitly requested by the user.

### Owner-Aware Naming

- When an item belongs to a specific person, set `--custom-name` to `<user>'s <base item name>`.
- Keep the original base item name in the custom name (for example, `davide's harvest bowl`).
- Do not include modifications (add/remove/substitute details) in the custom name.
- For follow-up requests like "update mine", match the sender to their custom-named line item first.

## Guardrails

- Confirm destructive actions (`remove`, `clear`) when user intent is ambiguous.
- Do not claim checkout is complete; this CLI manages cart state only.
- Do not place or submit orders, and do not offer to place orders.
- On failures, include debug report paths from `~/.sweetgreen/debug/`.

## Reference

Use [references/command-recipes.md](references/command-recipes.md) for common task-to-command mappings.
