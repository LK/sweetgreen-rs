use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

use crate::client::SweetgreenClient;
use crate::config::{default_cookie_path, default_state_path};
use crate::error::SweetgreenError;
use crate::models::{
    AddLineItemToCartInput, DeliveryOrderDetailInput, DressingWeight, EditLineItemInCartInput,
    IngredientModificationInput, IngredientSubstitutionModificationInput, MenuIngredient,
    MenuProduct, MenuRestaurant, MixedDressingDetailsInput,
};
use crate::state::{AuthState, StateStore};

#[derive(Debug, Parser)]
#[command(name = "sg", about = "sweetgreen cart CLI")]
pub struct Cli {
    #[arg(long, env = "SG_STATE_PATH")]
    pub state_path: Option<PathBuf>,

    #[arg(long, env = "SG_COOKIE_PATH")]
    pub cookie_path: Option<PathBuf>,

    #[arg(long, env = "SG_GRAPHQL_ENDPOINT")]
    pub graphql_endpoint: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },
    Menu {
        #[command(subcommand)]
        command: MenuCommands,
    },
    Cart {
        #[command(subcommand)]
        command: CartCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    Login(LoginArgs),
    ImportBrowserCookies(ImportBrowserCookiesArgs),
    Refresh,
    Status,
    Logout,
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    #[arg(long)]
    pub email: String,

    #[arg(long)]
    pub code: Option<String>,
}

#[derive(Debug, Args)]
pub struct ImportBrowserCookiesArgs {
    #[arg(long, value_enum, default_value_t = BrowserKind::Dia)]
    pub browser: BrowserKind,

    #[arg(long, default_value = "sweetgreen.com")]
    pub domain: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BrowserKind {
    Dia,
    Arc,
    Chrome,
    Chromium,
    Brave,
    Edge,
    Opera,
}

impl BrowserKind {
    fn python_name(self) -> &'static str {
        match self {
            BrowserKind::Dia => "dia",
            BrowserKind::Arc => "arc",
            BrowserKind::Chrome => "chrome",
            BrowserKind::Chromium => "chromium",
            BrowserKind::Brave => "brave",
            BrowserKind::Edge => "edge",
            BrowserKind::Opera => "opera",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum CartCommands {
    View,
    Count,
    WantedTimes,
    Add(AddLineItemArgs),
    AddByName(AddLineItemByNameArgs),
    Update(UpdateLineItemArgs),
    Remove(RemoveLineItemArgs),
    Clear,
    PromoApply(PromoApplyArgs),
    Rewards,
    RewardApply(RewardApplyArgs),
    RewardRemove(RewardRemoveArgs),
    GiftBalance,
    GiftRedeem(GiftRedeemArgs),
}

#[derive(Debug, Args)]
pub struct AddLineItemArgs {
    #[arg(long)]
    pub product_id: String,

    #[arg(long, default_value_t = 1)]
    pub quantity: i64,

    #[arg(long)]
    pub custom_name: Option<String>,

    #[arg(long)]
    pub restaurant_id: Option<i64>,

    #[arg(long, value_delimiter = ',')]
    pub additions: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub removals: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub substitutions: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub mixed_dressings: Vec<String>,

    #[arg(long)]
    pub delivery_address_id: Option<String>,

    #[arg(long)]
    pub delivery_fee: Option<f64>,

    #[arg(long)]
    pub delivery_tip: Option<f64>,

    #[arg(long)]
    pub delivery_vendor: Option<String>,

    #[arg(long)]
    pub delivery_vendor_restaurant_id: Option<String>,
}

#[derive(Debug, Args)]
pub struct AddLineItemByNameArgs {
    #[arg(long)]
    pub restaurant_id: Option<String>,

    #[arg(long)]
    pub product_name: String,

    #[arg(long, default_value_t = 1)]
    pub quantity: i64,

    #[arg(long)]
    pub custom_name: Option<String>,

    #[arg(long, value_delimiter = ',')]
    pub additions: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub removals: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub substitutions: Vec<String>,

    #[arg(long)]
    pub delivery_address_id: Option<String>,

    #[arg(long)]
    pub delivery_fee: Option<f64>,

    #[arg(long)]
    pub delivery_tip: Option<f64>,

    #[arg(long)]
    pub delivery_vendor: Option<String>,

    #[arg(long)]
    pub delivery_vendor_restaurant_id: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum MenuCommands {
    Products(MenuProductsArgs),
    Product(MenuProductArgs),
    Restaurants(MenuRestaurantsArgs),
    Ingredient(MenuIngredientArgs),
}

#[derive(Debug, Args)]
pub struct MenuProductsArgs {
    #[arg(long)]
    pub restaurant_id: String,

    #[arg(long)]
    pub search: Option<String>,

    #[arg(long, default_value_t = false)]
    pub include_out_of_stock: bool,

    #[arg(long, default_value_t = false)]
    pub include_ingredients: bool,
}

#[derive(Debug, Args)]
pub struct MenuProductArgs {
    #[arg(long)]
    pub restaurant_id: String,

    #[arg(long)]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct MenuRestaurantsArgs {
    #[arg(long)]
    pub query: String,

    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct MenuIngredientArgs {
    #[arg(long)]
    pub restaurant_id: String,

    #[arg(long)]
    pub name: String,

    #[arg(long, default_value_t = 5)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct UpdateLineItemArgs {
    #[arg(long)]
    pub line_item_id: String,

    #[arg(long)]
    pub product_id: String,

    #[arg(long, default_value_t = 1)]
    pub quantity: i64,

    #[arg(long)]
    pub custom_name: Option<String>,

    #[arg(long, value_delimiter = ',')]
    pub additions: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub removals: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub substitutions: Vec<String>,

    #[arg(long, value_delimiter = ',')]
    pub mixed_dressings: Vec<String>,
}

#[derive(Debug, Args)]
pub struct RemoveLineItemArgs {
    #[arg(long)]
    pub line_item_id: String,
}

#[derive(Debug, Args)]
pub struct PromoApplyArgs {
    #[arg(long)]
    pub code: String,
}

#[derive(Debug, Args)]
pub struct RewardApplyArgs {
    #[arg(long)]
    pub order_id: String,

    #[arg(long)]
    pub reward_id: String,
}

#[derive(Debug, Args)]
pub struct RewardRemoveArgs {
    #[arg(long)]
    pub order_id: String,
}

#[derive(Debug, Args)]
pub struct GiftRedeemArgs {
    #[arg(long)]
    pub code: String,

    #[arg(long)]
    pub reg_code: Option<String>,
}

pub async fn run(cli: Cli) -> Result<(), SweetgreenError> {
    let state_path = cli.state_path.unwrap_or_else(default_state_path);
    let cookie_path = cli.cookie_path.unwrap_or_else(default_cookie_path);
    let store = StateStore::new(state_path);
    let client = SweetgreenClient::new(store, cli.graphql_endpoint, cookie_path)?;

    let result = match cli.command {
        Commands::Auth { command } => run_auth(command, &client).await,
        Commands::Menu { command } => run_menu(command, &client).await,
        Commands::Cart { command } => run_cart(command, &client).await,
    };

    let save_result = client.save_cookies();
    if result.is_ok() {
        save_result?;
    }

    result
}

async fn run_auth(command: AuthCommands, client: &SweetgreenClient) -> Result<(), SweetgreenError> {
    match command {
        AuthCommands::Login(args) => {
            let state = client.auth_login(&args.email, args.code.as_deref()).await?;
            print_json(&state.redacted());
            Ok(())
        }
        AuthCommands::ImportBrowserCookies(args) => {
            let imported =
                client.import_browser_cookies(args.browser.python_name(), &args.domain)?;
            if imported > 0 {
                println!(
                    "Imported {imported} cookie(s) from {} for domain {} and switched auth mode to browser.",
                    args.browser.python_name(),
                    args.domain
                );
            } else {
                println!(
                    "Imported 0 cookie(s) from {} for domain {}; auth mode unchanged.",
                    args.browser.python_name(),
                    args.domain
                );
            }
            Ok(())
        }
        AuthCommands::Refresh => {
            let state = client.auth_refresh().await?;
            print_json(&state.redacted());
            Ok(())
        }
        AuthCommands::Status => {
            let state = client.load_state()?;
            print_json(&state.redacted());
            Ok(())
        }
        AuthCommands::Logout => {
            client.clear_state()?;
            client.clear_cookies()?;
            println!("Logged out and cleared local state.");
            Ok(())
        }
    }
}

async fn run_menu(command: MenuCommands, client: &SweetgreenClient) -> Result<(), SweetgreenError> {
    match command {
        MenuCommands::Products(args) => {
            let restaurant = client.menu_for_restaurant(args.restaurant_id).await?;
            let mut products = flatten_menu_products(&restaurant);
            if !args.include_out_of_stock {
                products.retain(|entry| !entry.product.out_of_stock.unwrap_or(false));
            }
            if let Some(search) = args.search.as_deref() {
                let search_norm = normalize_text(search);
                products.retain(|entry| normalize_text(&entry.product.name).contains(&search_norm));
            }

            let output = MenuProductsOutput {
                restaurant_id: restaurant.id,
                restaurant_name: restaurant.name,
                products: products
                    .into_iter()
                    .map(|entry| MenuProductSummary {
                        id: entry.product.id,
                        name: entry.product.name,
                        slug: entry.product.slug,
                        category_id: entry.category_id,
                        category_name: entry.category_name,
                        out_of_stock: entry.product.out_of_stock.unwrap_or(false),
                        is_modifiable: entry.product.is_modifiable.unwrap_or(false),
                        restaurant_id: entry.product.restaurant_id,
                        ingredients: if args.include_ingredients {
                            Some(
                                entry
                                    .product
                                    .ingredients
                                    .iter()
                                    .map(|ingredient| ingredient.name.clone())
                                    .collect(),
                            )
                        } else {
                            None
                        },
                        ingredient_details: if args.include_ingredients {
                            Some(
                                entry
                                    .product
                                    .ingredients
                                    .iter()
                                    .map(|ingredient| MenuIngredientSummary {
                                        id: ingredient.id.clone(),
                                        name: ingredient.name.clone(),
                                    })
                                    .collect(),
                            )
                        } else {
                            None
                        },
                    })
                    .collect(),
            };
            print_json(&output);
            Ok(())
        }
        MenuCommands::Product(args) => {
            let restaurant = client.menu_for_restaurant(args.restaurant_id).await?;
            let products = flatten_menu_products(&restaurant);
            let resolved = resolve_product_by_name(&products, &args.name, false)?;
            let output = MenuResolvedProductOutput {
                restaurant_id: restaurant.id,
                restaurant_name: restaurant.name,
                requested_name: args.name,
                matched_product: MenuProductSummary {
                    id: resolved.entry.product.id.clone(),
                    name: resolved.entry.product.name.clone(),
                    slug: resolved.entry.product.slug.clone(),
                    category_id: resolved.entry.category_id.clone(),
                    category_name: resolved.entry.category_name.clone(),
                    out_of_stock: resolved.entry.product.out_of_stock.unwrap_or(false),
                    is_modifiable: resolved.entry.product.is_modifiable.unwrap_or(false),
                    restaurant_id: resolved.entry.product.restaurant_id.clone(),
                    ingredients: Some(
                        resolved
                            .entry
                            .product
                            .ingredients
                            .iter()
                            .map(|ingredient| ingredient.name.clone())
                            .collect(),
                    ),
                    ingredient_details: Some(
                        resolved
                            .entry
                            .product
                            .ingredients
                            .iter()
                            .map(|ingredient| MenuIngredientSummary {
                                id: ingredient.id.clone(),
                                name: ingredient.name.clone(),
                            })
                            .collect(),
                    ),
                },
                match_score: resolved.score,
            };
            print_json(&output);
            Ok(())
        }
        MenuCommands::Restaurants(args) => {
            if args.query.trim().is_empty() {
                return Err(SweetgreenError::InvalidArgument(
                    "search query cannot be empty".to_string(),
                ));
            }
            let matches = client
                .search_locations_by_string(args.query.clone(), Some(false))
                .await?;
            let output = MenuRestaurantSearchOutput {
                query: args.query,
                restaurants: matches
                    .into_iter()
                    .take(args.limit.max(1))
                    .map(|entry| MenuRestaurantSearchResult {
                        id: entry.location.id,
                        name: entry.location.name,
                        slug: entry.location.slug,
                        address: entry.location.address,
                        city: entry.location.city,
                        state: entry.location.state,
                        zip_code: entry.location.zip_code,
                        latitude: entry.location.latitude,
                        longitude: entry.location.longitude,
                        accepting_orders: entry.location.accepting_orders,
                        score: entry.score,
                    })
                    .collect(),
            };
            print_json(&output);
            Ok(())
        }
        MenuCommands::Ingredient(args) => {
            let restaurant = client.menu_for_restaurant(args.restaurant_id).await?;
            let products = flatten_menu_products(&restaurant);
            let catalog = build_restaurant_ingredient_catalog(&products);
            let mut matches = resolve_ingredient_matches(
                &catalog,
                &args.name,
                args.limit.max(1),
                IngredientMatchScope::RestaurantOnly,
            );

            if matches.is_empty() {
                let deep_catalog =
                    build_customization_ingredient_catalog(client, &restaurant, &products).await;
                if !deep_catalog.is_empty() {
                    matches = resolve_ingredient_matches(
                        &deep_catalog,
                        &args.name,
                        args.limit.max(1),
                        IngredientMatchScope::RestaurantOnly,
                    );
                }
            }

            if matches.is_empty() {
                return Err(SweetgreenError::InvalidArgument(format!(
                    "no ingredient matched '{}'",
                    args.name
                )));
            }

            let output = MenuIngredientMatchesOutput {
                restaurant_id: restaurant.id,
                restaurant_name: restaurant.name,
                requested_name: args.name,
                matches,
            };
            print_json(&output);
            Ok(())
        }
    }
}

async fn run_cart(command: CartCommands, client: &SweetgreenClient) -> Result<(), SweetgreenError> {
    let state = client.ensure_cart_state().await?;

    match command {
        CartCommands::View => {
            let cart = client.cart(&state).await?;
            print_json(&cart);
            Ok(())
        }
        CartCommands::Count => {
            let count = client.bag_count(&state).await?;
            println!("{count}");
            Ok(())
        }
        CartCommands::WantedTimes => {
            let wanted_times = client.wanted_times(&state).await?;
            print_json(&wanted_times);
            Ok(())
        }
        CartCommands::Add(args) => {
            let delivery_order_details =
                resolve_delivery_details_for_cart(client, &state, build_delivery_details(&args)?)
                    .await?;
            let input = AddLineItemToCartInput {
                additions: args
                    .additions
                    .into_iter()
                    .map(|ingredient_id| IngredientModificationInput { ingredient_id })
                    .collect(),
                custom_name: args.custom_name,
                delivery_order_details,
                mixed_dressing_details: parse_mixed_dressings(&args.mixed_dressings)?,
                product_id: args.product_id,
                quantity: args.quantity,
                removals: args
                    .removals
                    .into_iter()
                    .map(|ingredient_id| IngredientModificationInput { ingredient_id })
                    .collect(),
                restaurant_id: args.restaurant_id,
                substitutions: parse_substitutions(&args.substitutions)?,
            };

            let result = client.add_line_item(&state, input).await?;
            print_json(&result);
            Ok(())
        }
        CartCommands::AddByName(args) => {
            let restaurant_id = resolve_add_by_name_restaurant_id(client, &state, &args).await?;
            let restaurant = client.menu_for_restaurant(restaurant_id.clone()).await?;
            let products = flatten_menu_products(&restaurant);
            let ingredient_catalog = build_restaurant_ingredient_catalog(&products);
            let product_match = resolve_product_by_name(&products, &args.product_name, true)?;
            let effective_product_ingredients =
                resolve_effective_product_ingredients_for_add_by_name(
                    client,
                    &state,
                    &restaurant,
                    &product_match.entry.product,
                )
                .await;

            let mut resolved_modifications = resolve_named_modifications(
                &effective_product_ingredients,
                &ingredient_catalog,
                &args.additions,
                &args.removals,
                &args.substitutions,
                product_match.entry.product.name.as_str(),
            );

            let has_named_changes = !args.additions.is_empty()
                || !args.removals.is_empty()
                || !args.substitutions.is_empty();
            if resolved_modifications.is_err() && has_named_changes {
                let deep_catalog =
                    build_customization_ingredient_catalog(client, &restaurant, &products).await;
                if !deep_catalog.is_empty() {
                    resolved_modifications = resolve_named_modifications(
                        &effective_product_ingredients,
                        &deep_catalog,
                        &args.additions,
                        &args.removals,
                        &args.substitutions,
                        product_match.entry.product.name.as_str(),
                    );
                }
            }

            let (additions, removals, substitutions) = resolved_modifications?;

            let delivery_order_details = resolve_delivery_details_for_cart(
                client,
                &state,
                build_delivery_details_by_name(&args)?,
            )
            .await?;
            let numeric_restaurant_id = parse_optional_i64(&restaurant_id);

            let input = AddLineItemToCartInput {
                additions: additions
                    .iter()
                    .map(|item| IngredientModificationInput {
                        ingredient_id: item.ingredient_id.clone(),
                    })
                    .collect(),
                custom_name: args.custom_name.clone(),
                delivery_order_details,
                mixed_dressing_details: vec![],
                product_id: product_match.entry.product.id.clone(),
                quantity: args.quantity,
                removals: removals
                    .iter()
                    .map(|item| IngredientModificationInput {
                        ingredient_id: item.ingredient_id.clone(),
                    })
                    .collect(),
                restaurant_id: numeric_restaurant_id,
                substitutions,
            };

            let result = client.add_line_item(&state, input).await?;
            let output = AddByNameOutput {
                requested_product_name: args.product_name,
                matched_product_name: product_match.entry.product.name.clone(),
                matched_product_id: product_match.entry.product.id.clone(),
                product_match_score: product_match.score,
                additions,
                removals,
                result,
            };
            print_json(&output);
            Ok(())
        }
        CartCommands::Update(args) => {
            let input = EditLineItemInCartInput {
                additions: args
                    .additions
                    .into_iter()
                    .map(|ingredient_id| IngredientModificationInput { ingredient_id })
                    .collect(),
                custom_name: args.custom_name,
                line_item_id: args.line_item_id,
                mixed_dressing_details: parse_mixed_dressings(&args.mixed_dressings)?,
                product_id: args.product_id,
                quantity: args.quantity,
                removals: args
                    .removals
                    .into_iter()
                    .map(|ingredient_id| IngredientModificationInput { ingredient_id })
                    .collect(),
                substitutions: parse_substitutions(&args.substitutions)?,
            };

            let result = client.update_line_item(&state, input).await?;
            print_json(&result);
            Ok(())
        }
        CartCommands::Remove(args) => {
            let result = client.remove_line_item(&state, args.line_item_id).await?;
            print_json(&result);
            Ok(())
        }
        CartCommands::Clear => {
            let removed_count = client.clear_cart(&state).await?;
            println!("Removed {removed_count} line item(s).");
            Ok(())
        }
        CartCommands::PromoApply(args) => {
            let result = client.apply_promo(&state, args.code).await?;
            print_json(&result);
            Ok(())
        }
        CartCommands::Rewards => {
            let rewards = client.rewards(&state).await?;
            print_json(&rewards);
            Ok(())
        }
        CartCommands::RewardApply(args) => {
            let result = client
                .apply_reward(&state, args.order_id, args.reward_id)
                .await?;
            print_json(&result);
            Ok(())
        }
        CartCommands::RewardRemove(args) => {
            let result = client.remove_reward(&state, args.order_id).await?;
            print_json(&result);
            Ok(())
        }
        CartCommands::GiftBalance => {
            let result = client.gift_card_balance(&state).await?;
            print_json(&result);
            Ok(())
        }
        CartCommands::GiftRedeem(args) => {
            let result = client
                .redeem_gift_card(&state, args.code, args.reg_code)
                .await?;
            print_json(&result);
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct MenuProductEntry {
    category_id: String,
    category_name: String,
    product: MenuProduct,
}

#[derive(Debug, Serialize)]
struct MenuProductsOutput {
    restaurant_id: String,
    restaurant_name: String,
    products: Vec<MenuProductSummary>,
}

#[derive(Debug, Serialize)]
struct MenuResolvedProductOutput {
    restaurant_id: String,
    restaurant_name: String,
    requested_name: String,
    matched_product: MenuProductSummary,
    match_score: i32,
}

#[derive(Debug, Serialize)]
struct MenuProductSummary {
    id: String,
    name: String,
    slug: Option<String>,
    category_id: String,
    category_name: String,
    out_of_stock: bool,
    is_modifiable: bool,
    restaurant_id: Option<String>,
    ingredients: Option<Vec<String>>,
    ingredient_details: Option<Vec<MenuIngredientSummary>>,
}

#[derive(Debug, Serialize)]
struct MenuIngredientSummary {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct MenuIngredientMatchesOutput {
    restaurant_id: String,
    restaurant_name: String,
    requested_name: String,
    matches: Vec<ResolvedIngredientMatch>,
}

#[derive(Debug, Serialize)]
struct MenuRestaurantSearchOutput {
    query: String,
    restaurants: Vec<MenuRestaurantSearchResult>,
}

#[derive(Debug, Serialize)]
struct MenuRestaurantSearchResult {
    id: String,
    name: String,
    slug: Option<String>,
    address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip_code: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    accepting_orders: Option<bool>,
    score: Option<f64>,
}

#[derive(Debug, Serialize)]
struct AddByNameOutput {
    requested_product_name: String,
    matched_product_name: String,
    matched_product_id: String,
    product_match_score: i32,
    additions: Vec<ResolvedIngredientMatch>,
    removals: Vec<ResolvedIngredientMatch>,
    result: crate::models::AddLineItemMutationData,
}

#[derive(Debug, Clone, Serialize)]
struct ResolvedIngredientMatch {
    requested_name: String,
    matched_name: String,
    ingredient_id: String,
    match_score: i32,
    match_scope: String,
    product_examples: Vec<String>,
}

struct ProductMatch<'a> {
    entry: &'a MenuProductEntry,
    score: i32,
}

#[derive(Debug, Clone)]
struct RestaurantIngredientEntry {
    ingredient_id: String,
    ingredient_name: String,
    product_names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngredientMatchScope {
    ProductThenRestaurant,
    RestaurantOnly,
}

fn flatten_menu_products(restaurant: &MenuRestaurant) -> Vec<MenuProductEntry> {
    let mut out = Vec::new();
    let Some(menu) = restaurant.menu.as_ref() else {
        return out;
    };

    for category in &menu.categories {
        for product in &category.products {
            out.push(MenuProductEntry {
                category_id: category.id.clone(),
                category_name: category.name.clone(),
                product: product.clone(),
            });
        }
    }

    out
}

fn build_restaurant_ingredient_catalog(
    products: &[MenuProductEntry],
) -> Vec<RestaurantIngredientEntry> {
    let mut by_id: BTreeMap<String, (String, BTreeSet<String>)> = BTreeMap::new();
    for entry in products {
        for ingredient in &entry.product.ingredients {
            let slot = by_id
                .entry(ingredient.id.clone())
                .or_insert_with(|| (ingredient.name.clone(), BTreeSet::new()));
            if slot.0.is_empty() {
                slot.0 = ingredient.name.clone();
            }
            slot.1.insert(entry.product.name.clone());
        }
    }

    by_id
        .into_iter()
        .map(
            |(ingredient_id, (ingredient_name, product_names))| RestaurantIngredientEntry {
                ingredient_id,
                ingredient_name,
                product_names: product_names.into_iter().collect(),
            },
        )
        .collect()
}

async fn build_customization_ingredient_catalog(
    client: &SweetgreenClient,
    restaurant: &MenuRestaurant,
    products: &[MenuProductEntry],
) -> Vec<RestaurantIngredientEntry> {
    let mut by_id: BTreeMap<String, (String, BTreeSet<String>)> = BTreeMap::new();
    let state = AuthState::default();

    for entry in products {
        let customization_candidates = customization_lookup_candidates(
            entry.product.slug.as_deref(),
            &entry.product.id,
            restaurant,
        );
        let mut customization_ingredients = None;
        for (product_ref, restaurant_ref) in customization_candidates {
            let Ok(ingredients) = client
                .customization_ingredients_for_product(&state, product_ref, restaurant_ref)
                .await
            else {
                continue;
            };
            customization_ingredients = Some(ingredients);
            break;
        }

        let Some(ingredients) = customization_ingredients else {
            continue;
        };

        for ingredient in ingredients {
            let slot = by_id
                .entry(ingredient.id.clone())
                .or_insert_with(|| (ingredient.name.clone(), BTreeSet::new()));
            if slot.0.is_empty() {
                slot.0 = ingredient.name.clone();
            }
            slot.1.insert(entry.product.name.clone());
        }
    }

    by_id
        .into_iter()
        .map(
            |(ingredient_id, (ingredient_name, product_names))| RestaurantIngredientEntry {
                ingredient_id,
                ingredient_name,
                product_names: product_names.into_iter().collect(),
            },
        )
        .collect()
}

async fn resolve_effective_product_ingredients_for_add_by_name(
    client: &SweetgreenClient,
    state: &AuthState,
    restaurant: &MenuRestaurant,
    product: &MenuProduct,
) -> Vec<MenuIngredient> {
    let mut ingredients_by_id = BTreeMap::new();
    for ingredient in &product.ingredients {
        ingredients_by_id.insert(ingredient.id.clone(), ingredient.clone());
    }

    let customization_candidates =
        customization_lookup_candidates(product.slug.as_deref(), &product.id, restaurant);
    for (product_ref, restaurant_ref) in customization_candidates {
        let Ok(customization_ingredients) = client
            .customization_ingredients_for_product(state, product_ref, restaurant_ref)
            .await
        else {
            continue;
        };

        for ingredient in customization_ingredients {
            ingredients_by_id
                .entry(ingredient.id.clone())
                .or_insert(ingredient);
        }
        break;
    }

    ingredients_by_id.into_values().collect()
}

fn customization_lookup_candidates(
    product_slug: Option<&str>,
    product_id: &str,
    restaurant: &MenuRestaurant,
) -> Vec<(String, String)> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    let mut push = |product_ref: &str, restaurant_ref: &str| {
        if product_ref.trim().is_empty() || restaurant_ref.trim().is_empty() {
            return;
        }
        let key = format!("{product_ref}::{restaurant_ref}");
        if seen.insert(key) {
            candidates.push((product_ref.to_string(), restaurant_ref.to_string()));
        }
    };

    if let (Some(pslug), Some(rslug)) = (product_slug, restaurant.slug.as_deref()) {
        push(pslug, rslug);
    }
    if let Some(pslug) = product_slug {
        push(pslug, &restaurant.id);
    }
    if let Some(rslug) = restaurant.slug.as_deref() {
        push(product_id, rslug);
    }
    push(product_id, &restaurant.id);

    candidates
}

fn resolve_product_by_name<'a>(
    products: &'a [MenuProductEntry],
    query: &str,
    prefer_available: bool,
) -> Result<ProductMatch<'a>, SweetgreenError> {
    let query_norm = normalize_text(query);
    if query_norm.is_empty() {
        return Err(SweetgreenError::InvalidArgument(
            "product name cannot be empty".to_string(),
        ));
    }

    let mut scored = products
        .iter()
        .map(|entry| {
            let mut score = fuzzy_match_score(&query_norm, &normalize_text(&entry.product.name));
            if prefer_available && !entry.product.out_of_stock.unwrap_or(false) {
                score += 250;
            }
            (entry, score)
        })
        .filter(|(_, score)| *score > 0)
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| b.1.cmp(&a.1));

    let Some((top_entry, top_score)) = scored.first().copied() else {
        return Err(SweetgreenError::InvalidArgument(format!(
            "no menu product matched '{query}'"
        )));
    };

    if top_score < 500 {
        let suggestions = scored
            .iter()
            .take(5)
            .map(|(entry, _)| entry.product.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(SweetgreenError::InvalidArgument(format!(
            "unable to confidently match product '{query}'. Top candidates: {suggestions}"
        )));
    }

    if let Some((second_entry, second_score)) = scored.get(1).copied()
        && second_score >= top_score - 40
        && normalize_text(&second_entry.product.name) != normalize_text(&top_entry.product.name)
        && top_score < 10_000
    {
        return Err(SweetgreenError::InvalidArgument(format!(
            "ambiguous product '{query}'. Best matches: '{}', '{}'",
            top_entry.product.name, second_entry.product.name
        )));
    }

    Ok(ProductMatch {
        entry: top_entry,
        score: top_score,
    })
}

fn resolve_ingredient_list(
    product_ingredients: &[MenuIngredient],
    restaurant_ingredients: &[RestaurantIngredientEntry],
    requested: &[String],
    kind: &str,
    product_name: &str,
) -> Result<Vec<ResolvedIngredientMatch>, SweetgreenError> {
    requested
        .iter()
        .map(|requested_name| {
            resolve_single_ingredient(
                product_ingredients,
                restaurant_ingredients,
                requested_name,
                kind,
                product_name,
            )
        })
        .collect()
}

fn resolve_substitutions_by_name(
    product_ingredients: &[MenuIngredient],
    restaurant_ingredients: &[RestaurantIngredientEntry],
    substitutions: &[String],
    product_name: &str,
) -> Result<Vec<IngredientSubstitutionModificationInput>, SweetgreenError> {
    substitutions
        .iter()
        .map(|item| {
            let mut parts = item.split(':');
            let added_name = parts
                .next()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    SweetgreenError::InvalidArgument(format!(
                        "invalid substitution '{}', expected added_name:removed_name",
                        item
                    ))
                })?
                .to_string();
            let removed_name = parts
                .next()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    SweetgreenError::InvalidArgument(format!(
                        "invalid substitution '{}', expected added_name:removed_name",
                        item
                    ))
                })?
                .to_string();
            if parts.next().is_some() {
                return Err(SweetgreenError::InvalidArgument(format!(
                    "invalid substitution '{}', expected added_name:removed_name",
                    item
                )));
            }

            let added = resolve_single_ingredient(
                product_ingredients,
                restaurant_ingredients,
                &added_name,
                "substitution add",
                product_name,
            )?;
            let removed = resolve_single_ingredient(
                product_ingredients,
                restaurant_ingredients,
                &removed_name,
                "substitution remove",
                product_name,
            )?;

            Ok(IngredientSubstitutionModificationInput {
                added_ingredient_id: added.ingredient_id,
                removed_ingredient_id: removed.ingredient_id,
            })
        })
        .collect()
}

fn resolve_named_modifications(
    product_ingredients: &[MenuIngredient],
    restaurant_ingredients: &[RestaurantIngredientEntry],
    additions: &[String],
    removals: &[String],
    substitutions: &[String],
    product_name: &str,
) -> Result<
    (
        Vec<ResolvedIngredientMatch>,
        Vec<ResolvedIngredientMatch>,
        Vec<IngredientSubstitutionModificationInput>,
    ),
    SweetgreenError,
> {
    let additions = resolve_ingredient_list(
        product_ingredients,
        restaurant_ingredients,
        additions,
        "addition ingredient",
        product_name,
    )?;
    let removals = resolve_ingredient_list(
        product_ingredients,
        restaurant_ingredients,
        removals,
        "removal ingredient",
        product_name,
    )?;
    let substitutions = resolve_substitutions_by_name(
        product_ingredients,
        restaurant_ingredients,
        substitutions,
        product_name,
    )?;
    Ok((additions, removals, substitutions))
}

fn resolve_single_ingredient(
    product_ingredients: &[MenuIngredient],
    restaurant_ingredients: &[RestaurantIngredientEntry],
    requested_name: &str,
    kind: &str,
    product_name: &str,
) -> Result<ResolvedIngredientMatch, SweetgreenError> {
    let scored = score_ingredient_candidates(
        requested_name,
        product_name,
        product_ingredients,
        restaurant_ingredients,
        IngredientMatchScope::ProductThenRestaurant,
    );
    if let Some(top) = scored.first()
        && top.match_score >= 500
    {
        return Ok(top.clone());
    }

    let restaurant_only = score_ingredient_candidates(
        requested_name,
        product_name,
        product_ingredients,
        restaurant_ingredients,
        IngredientMatchScope::RestaurantOnly,
    );

    let suggestions = restaurant_only
        .iter()
        .take(5)
        .map(format_ingredient_suggestion)
        .collect::<Vec<_>>();
    if suggestions.is_empty() {
        return Err(SweetgreenError::InvalidArgument(format!(
            "no {kind} matched '{requested_name}'"
        )));
    }

    Err(SweetgreenError::InvalidArgument(format!(
        "unable to confidently match {kind} '{requested_name}'. Top restaurant matches: {}",
        suggestions.join(", ")
    )))
}

fn resolve_ingredient_matches(
    restaurant_ingredients: &[RestaurantIngredientEntry],
    requested_name: &str,
    limit: usize,
    scope: IngredientMatchScope,
) -> Vec<ResolvedIngredientMatch> {
    score_ingredient_candidates(requested_name, "", &[], restaurant_ingredients, scope)
        .into_iter()
        .take(limit)
        .collect()
}

fn score_ingredient_candidates(
    requested_name: &str,
    product_name: &str,
    product_ingredients: &[MenuIngredient],
    restaurant_ingredients: &[RestaurantIngredientEntry],
    scope: IngredientMatchScope,
) -> Vec<ResolvedIngredientMatch> {
    let query_norm = normalize_text(requested_name);
    if query_norm.is_empty() {
        return Vec::new();
    }

    let include_product = matches!(scope, IngredientMatchScope::ProductThenRestaurant);
    let include_restaurant = matches!(
        scope,
        IngredientMatchScope::ProductThenRestaurant | IngredientMatchScope::RestaurantOnly
    );

    let mut out = Vec::new();
    let mut product_ids = HashSet::new();

    if include_product {
        for ingredient in product_ingredients {
            let score = fuzzy_match_score(&query_norm, &normalize_text(&ingredient.name));
            if score <= 0 {
                continue;
            }
            product_ids.insert(ingredient.id.clone());
            out.push(ResolvedIngredientMatch {
                requested_name: requested_name.to_string(),
                matched_name: ingredient.name.clone(),
                ingredient_id: ingredient.id.clone(),
                match_score: score + 200,
                match_scope: "product".to_string(),
                product_examples: if product_name.is_empty() {
                    Vec::new()
                } else {
                    vec![product_name.to_string()]
                },
            });
        }
    }

    if include_restaurant {
        for ingredient in restaurant_ingredients {
            if scope != IngredientMatchScope::RestaurantOnly
                && product_ids.contains(&ingredient.ingredient_id)
            {
                continue;
            }
            let score =
                fuzzy_match_score(&query_norm, &normalize_text(&ingredient.ingredient_name));
            if score <= 0 {
                continue;
            }
            out.push(ResolvedIngredientMatch {
                requested_name: requested_name.to_string(),
                matched_name: ingredient.ingredient_name.clone(),
                ingredient_id: ingredient.ingredient_id.clone(),
                match_score: score,
                match_scope: "restaurant".to_string(),
                product_examples: ingredient.product_names.clone(),
            });
        }
    }

    out.sort_by(|a, b| b.match_score.cmp(&a.match_score));
    let mut seen = HashSet::new();
    out.into_iter()
        .filter(|item| seen.insert(item.ingredient_id.clone()))
        .collect()
}

fn format_ingredient_suggestion(item: &ResolvedIngredientMatch) -> String {
    if item.product_examples.is_empty() {
        return format!("{} (id={})", item.matched_name, item.ingredient_id);
    }
    format!(
        "{} (id={}, products={})",
        item.matched_name,
        item.ingredient_id,
        item.product_examples.join("|")
    )
}

fn normalize_text(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn fuzzy_match_score(query_norm: &str, candidate_norm: &str) -> i32 {
    if query_norm.is_empty() || candidate_norm.is_empty() {
        return 0;
    }
    if query_norm == candidate_norm {
        return 10_000;
    }
    if candidate_norm.starts_with(query_norm) {
        return 9_000 - (candidate_norm.len() as i32 - query_norm.len() as i32);
    }
    if candidate_norm.contains(query_norm) {
        return 8_000 - (candidate_norm.len() as i32 - query_norm.len() as i32);
    }

    let query_tokens = query_norm.split_whitespace().collect::<Vec<_>>();
    let candidate_tokens = candidate_norm.split_whitespace().collect::<Vec<_>>();
    let common = query_tokens
        .iter()
        .filter(|token| candidate_tokens.contains(token))
        .count() as i32;
    if common == 0 {
        return 0;
    }

    let token_penalty = (query_tokens.len() as i32 - candidate_tokens.len() as i32).abs() * 75;
    (common * 1_000) - token_penalty
}

fn parse_optional_i64(raw: &str) -> Option<i64> {
    raw.parse::<i64>().ok()
}

async fn resolve_add_by_name_restaurant_id(
    client: &SweetgreenClient,
    state: &AuthState,
    args: &AddLineItemByNameArgs,
) -> Result<String, SweetgreenError> {
    if let Some(restaurant_id) = args.restaurant_id.as_ref()
        && !restaurant_id.trim().is_empty()
    {
        return Ok(restaurant_id.clone());
    }

    if let Some(cart) = client.cart(state).await?
        && let Some(restaurant) = cart.restaurant
    {
        return Ok(restaurant.id);
    }

    Err(SweetgreenError::InvalidArgument(
        "--restaurant-id is required when there is no active cart restaurant".to_string(),
    ))
}

fn build_delivery_details(
    args: &AddLineItemArgs,
) -> Result<Option<DeliveryOrderDetailInput>, SweetgreenError> {
    build_delivery_details_from_parts(
        args.delivery_address_id.clone(),
        args.delivery_fee,
        args.delivery_tip,
        args.delivery_vendor.clone(),
        args.delivery_vendor_restaurant_id.clone(),
    )
}

fn build_delivery_details_by_name(
    args: &AddLineItemByNameArgs,
) -> Result<Option<DeliveryOrderDetailInput>, SweetgreenError> {
    build_delivery_details_from_parts(
        args.delivery_address_id.clone(),
        args.delivery_fee,
        args.delivery_tip,
        args.delivery_vendor.clone(),
        args.delivery_vendor_restaurant_id.clone(),
    )
}

async fn resolve_delivery_details_for_cart(
    client: &SweetgreenClient,
    state: &AuthState,
    provided: Option<DeliveryOrderDetailInput>,
) -> Result<Option<DeliveryOrderDetailInput>, SweetgreenError> {
    if provided.is_some() {
        return Ok(provided);
    }

    let Some(cart) = client.cart(state).await? else {
        return Ok(None);
    };
    let Some(details) = cart.delivery_order_detail else {
        return Ok(None);
    };
    let Some(address) = details.address else {
        return Ok(None);
    };
    let Some(vendor) = details.vendor else {
        return Ok(None);
    };
    let Some(vendor_restaurant_id) = details.vendor_restaurant_id else {
        return Ok(None);
    };

    Ok(Some(DeliveryOrderDetailInput {
        address_id: address.id,
        delivery_fee: details.delivery_fee.unwrap_or(0.0),
        tip: details.tip.unwrap_or(0.0),
        vendor,
        vendor_restaurant_id,
    }))
}

fn build_delivery_details_from_parts(
    address_id: Option<String>,
    delivery_fee: Option<f64>,
    tip: Option<f64>,
    vendor: Option<String>,
    vendor_restaurant_id: Option<String>,
) -> Result<Option<DeliveryOrderDetailInput>, SweetgreenError> {
    let Some(address_id) = address_id else {
        return Ok(None);
    };

    let delivery_fee = delivery_fee.ok_or_else(|| {
        SweetgreenError::InvalidArgument(
            "--delivery-fee is required when --delivery-address-id is set".to_string(),
        )
    })?;
    let tip = tip.ok_or_else(|| {
        SweetgreenError::InvalidArgument(
            "--delivery-tip is required when --delivery-address-id is set".to_string(),
        )
    })?;
    let vendor = vendor.ok_or_else(|| {
        SweetgreenError::InvalidArgument(
            "--delivery-vendor is required when --delivery-address-id is set".to_string(),
        )
    })?;
    let vendor_restaurant_id = vendor_restaurant_id.ok_or_else(|| {
        SweetgreenError::InvalidArgument(
            "--delivery-vendor-restaurant-id is required when --delivery-address-id is set"
                .to_string(),
        )
    })?;

    Ok(Some(DeliveryOrderDetailInput {
        address_id,
        delivery_fee,
        tip,
        vendor,
        vendor_restaurant_id,
    }))
}

fn parse_substitutions(
    substitutions: &[String],
) -> Result<Vec<IngredientSubstitutionModificationInput>, SweetgreenError> {
    substitutions
        .iter()
        .map(|item| {
            let mut parts = item.split(':');
            let added_ingredient_id = parts
                .next()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    SweetgreenError::InvalidArgument(format!(
                        "invalid substitution '{}', expected added_id:removed_id",
                        item
                    ))
                })?
                .to_string();
            let removed_ingredient_id = parts
                .next()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    SweetgreenError::InvalidArgument(format!(
                        "invalid substitution '{}', expected added_id:removed_id",
                        item
                    ))
                })?
                .to_string();

            if parts.next().is_some() {
                return Err(SweetgreenError::InvalidArgument(format!(
                    "invalid substitution '{}', expected added_id:removed_id",
                    item
                )));
            }

            Ok(IngredientSubstitutionModificationInput {
                added_ingredient_id,
                removed_ingredient_id,
            })
        })
        .collect()
}

fn parse_mixed_dressings(
    mixed_dressings: &[String],
) -> Result<Vec<MixedDressingDetailsInput>, SweetgreenError> {
    mixed_dressings
        .iter()
        .map(|item| {
            let mut parts = item.split(':');
            let ingredient_id = parts
                .next()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| {
                    SweetgreenError::InvalidArgument(format!(
                        "invalid mixed dressing '{}', expected ingredient_id:weight",
                        item
                    ))
                })?
                .to_string();

            let weight = match parts
                .next()
                .ok_or_else(|| {
                    SweetgreenError::InvalidArgument(format!(
                        "invalid mixed dressing '{}', expected ingredient_id:weight",
                        item
                    ))
                })?
                .to_ascii_uppercase()
                .as_str()
            {
                "LIGHT" => DressingWeight::Light,
                "MEDIUM" => DressingWeight::Medium,
                "HEAVY" => DressingWeight::Heavy,
                value => {
                    return Err(SweetgreenError::InvalidArgument(format!(
                        "invalid dressing weight '{}' (expected LIGHT, MEDIUM, HEAVY)",
                        value
                    )));
                }
            };

            if parts.next().is_some() {
                return Err(SweetgreenError::InvalidArgument(format!(
                    "invalid mixed dressing '{}', expected ingredient_id:weight",
                    item
                )));
            }

            Ok(MixedDressingDetailsInput {
                ingredient_id,
                weight,
            })
        })
        .collect()
}

fn print_json<T>(value: &T)
where
    T: Serialize,
{
    let rendered = serde_json::to_string_pretty(value)
        .unwrap_or_else(|_| "{\"error\":\"failed to serialize output\"}".to_string());
    println!("{rendered}");
}
