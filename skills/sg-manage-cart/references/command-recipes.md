# Command Recipes

## Discover Restaurant IDs

```bash
sg menu restaurants --query "brooklyn"
sg menu restaurants --query "11201"
```

## Find a Product and Ingredients

```bash
sg menu products --restaurant-id 2104 --search "harvest" --include-ingredients
sg menu product --restaurant-id 2104 --name "harvest bowl"
sg menu ingredient --restaurant-id 2104 --name "corn salsa"
```

## Add an Item by Name

```bash
sg cart add-by-name --product-name "harvest bowl" --removals apples
sg cart add-by-name --product-name "chicken caesar wrap" --additions "corn salsa"
```

## Edit Existing Cart Items

```bash
sg cart view
sg cart update --line-item-id <line_item_id> --product-id <product_id> --quantity 2
sg cart remove --line-item-id <line_item_id>
```

## Authentication

```bash
sg auth status
sg auth import-browser-cookies --browser arc --domain sweetgreen.com
sg auth login --email you@example.com
```

## Debugging

```bash
sg cart view
```

If a command fails, read the report path printed by the CLI:

`~/.sweetgreen/debug/<timestamp>-<stage>.json`
