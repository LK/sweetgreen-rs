use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionQueryData {
    pub session: SessionInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub csrf: String,
    pub is_logged_in: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BagCountQueryData {
    pub cart: Option<CartCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartCount {
    pub id: String,
    #[serde(default)]
    pub line_items: Vec<LineItemCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineItemCount {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartQueryData {
    pub cart: Option<Cart>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cart {
    pub id: String,
    pub order_type: Option<String>,
    pub can_track_order_status: Option<bool>,
    pub restaurant: Option<Restaurant>,
    pub ledger: Option<Ledger>,
    pub delivery_order_detail: Option<CartDeliveryOrderDetail>,
    #[serde(default)]
    pub available_wanted_times: Vec<AvailableWantedTime>,
    #[serde(default)]
    pub line_items: Vec<CartLineItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Restaurant {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub entity_id: Option<i64>,
    pub delivery_fee: Option<i64>,
    pub delivery_min_subtotal: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ledger {
    pub subtotal: Option<f64>,
    pub tax: Option<f64>,
    pub tip: Option<f64>,
    pub fees_total: Option<f64>,
    pub discounts_total: Option<f64>,
    pub credits_total: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartDeliveryOrderDetail {
    pub tip: Option<f64>,
    pub delivery_fee: Option<f64>,
    pub vendor: Option<String>,
    pub vendor_restaurant_id: Option<String>,
    pub address: Option<CartDeliveryAddress>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartDeliveryAddress {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableWantedTime {
    pub time: String,
    pub delivery_offset: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartLineItem {
    pub id: String,
    pub slug: Option<String>,
    pub quantity: i64,
    pub custom_name: Option<String>,
    pub cost: Option<f64>,
    pub per_item_cost: Option<f64>,
    pub product: ProductSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSummary {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericUnionResult {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub message: Option<String>,
    pub status: Option<i64>,
    #[serde(default)]
    pub field_errors: Vec<FieldValidationError>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartMutationResult {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub message: Option<String>,
    pub status: Option<i64>,
    #[serde(default)]
    pub field_errors: Vec<FieldValidationError>,
    pub cart: Option<Cart>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLineItemMutationData {
    pub add_line_item_to_cart: CartMutationResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditLineItemMutationData {
    pub edit_line_item_in_cart: CartMutationResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveLineItemMutationData {
    pub remove_from_cart: CartMutationResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoMutationData {
    pub submit_promo_or_gift_card_code: GenericUnionResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardMutationData {
    pub apply_reward: RewardUnionResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveRewardMutationData {
    pub remove_reward: RewardUnionResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardUnionResult {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub message: Option<String>,
    pub status: Option<i64>,
    #[serde(default)]
    pub field_errors: Vec<FieldValidationError>,
    pub failure_code: Option<String>,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    pub order: Option<Cart>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRef {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewardsQueryData {
    #[serde(default)]
    pub rewards: Vec<Reward>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub id: String,
    pub name: String,
    pub expiration_date: Option<String>,
    pub reward_type: Option<String>,
    pub redeemable: Option<bool>,
    pub redeemable_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInMutationData {
    pub sign_in: SignInUnionResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInUnionResult {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub message: Option<String>,
    pub status: Option<i64>,
    pub error_message: Option<String>,
    #[serde(default)]
    pub field_errors: Vec<FieldValidationError>,
    pub customer: Option<OrderRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftCardBalanceQueryData {
    pub gift_card_balance: GiftCardUnionResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedeemGiftCardMutationData {
    pub redeem_gift_card: GiftCardUnionResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GiftCardUnionResult {
    #[serde(rename = "__typename")]
    pub typename: String,
    pub error_message: Option<String>,
    pub gift_card_balance: Option<f64>,
    pub customer_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuContentRestaurantQueryData {
    pub restaurant: Option<MenuRestaurant>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationDataQueryData {
    pub product: Option<CustomizationProduct>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationProduct {
    pub id: String,
    pub slug: Option<String>,
    pub name: String,
    #[serde(default)]
    pub ingredients: Vec<MenuIngredient>,
    #[serde(default)]
    pub modifier_groups: Vec<CustomizationModifierGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationModifierGroup {
    pub id: String,
    #[serde(default)]
    pub modifications: Vec<CustomizationModification>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizationModification {
    pub id: String,
    pub out_of_stock: Option<bool>,
    pub ingredient: Option<MenuIngredient>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuRestaurant {
    pub id: String,
    pub slug: Option<String>,
    pub name: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub is_outpost: Option<bool>,
    pub show_delivery_fee_disclosure: Option<bool>,
    pub sanitary_grade_display_text: Option<String>,
    pub sanitary_grade_url: Option<String>,
    pub utc_offset: Option<i64>,
    pub is_accepting_orders: Option<bool>,
    pub not_accepting_orders_reason: Option<String>,
    pub delivery_fee: Option<i64>,
    pub flex_message: Option<String>,
    pub menu: Option<MenuData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuData {
    pub id: String,
    #[serde(default)]
    pub categories: Vec<MenuCategory>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuCategory {
    pub id: String,
    pub name: String,
    pub is_custom: Option<bool>,
    pub description: Option<String>,
    #[serde(default)]
    pub products: Vec<MenuProduct>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuProduct {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub calories: Option<f64>,
    pub cost: Option<f64>,
    pub category_id: Option<String>,
    pub is_modifiable: Option<bool>,
    pub is_custom: Option<bool>,
    pub custom_type: Option<String>,
    pub is_salad: Option<bool>,
    pub out_of_stock: Option<bool>,
    pub restaurant_id: Option<String>,
    pub throttle_item: Option<bool>,
    pub base_product: Option<MenuBaseProduct>,
    #[serde(default)]
    pub ingredients: Vec<MenuIngredient>,
    pub label: Option<MenuLabel>,
    #[serde(default)]
    pub dietary_properties: Vec<MenuLabel>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuBaseProduct {
    pub id: String,
    pub slug: String,
    pub protein_g: Option<f64>,
    pub total_carbs_g: Option<f64>,
    pub total_fat_g: Option<f64>,
    pub calories: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuLabel {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuIngredient {
    pub id: String,
    pub name: String,
    pub protein_g: Option<f64>,
    pub total_carbs_g: Option<f64>,
    pub total_fat_g: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationsSearchByStringQueryData {
    #[serde(default)]
    pub search_locations_by_string: Vec<LocationSearchMatch>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationSearchMatch {
    pub score: Option<f64>,
    pub location: SearchLocation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchLocation {
    pub id: String,
    pub name: String,
    pub slug: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub is_outpost: Option<bool>,
    pub phone: Option<String>,
    pub store_hours: Option<String>,
    pub flex_message: Option<String>,
    pub enabled: Option<bool>,
    pub accepting_orders: Option<bool>,
    pub not_accepting_orders_reason: Option<String>,
    pub hidden: Option<bool>,
    pub sanitary_grade_display_text: Option<String>,
    pub sanitary_grade_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientModificationInput {
    pub ingredient_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientSubstitutionModificationInput {
    pub added_ingredient_id: String,
    pub removed_ingredient_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DressingWeight {
    Light,
    Medium,
    Heavy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MixedDressingDetailsInput {
    pub ingredient_id: String,
    pub weight: DressingWeight,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryOrderDetailInput {
    pub address_id: String,
    pub delivery_fee: f64,
    pub tip: f64,
    pub vendor: String,
    pub vendor_restaurant_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLineItemToCartInput {
    pub additions: Vec<IngredientModificationInput>,
    pub custom_name: Option<String>,
    pub delivery_order_details: Option<DeliveryOrderDetailInput>,
    pub mixed_dressing_details: Vec<MixedDressingDetailsInput>,
    pub product_id: String,
    pub quantity: i64,
    pub removals: Vec<IngredientModificationInput>,
    pub restaurant_id: Option<i64>,
    pub substitutions: Vec<IngredientSubstitutionModificationInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditLineItemInCartInput {
    pub additions: Vec<IngredientModificationInput>,
    pub custom_name: Option<String>,
    pub line_item_id: String,
    pub mixed_dressing_details: Vec<MixedDressingDetailsInput>,
    pub product_id: String,
    pub quantity: i64,
    pub removals: Vec<IngredientModificationInput>,
    pub substitutions: Vec<IngredientSubstitutionModificationInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveFromCartInput {
    pub line_item_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitPromoOrGiftCardCodeInput {
    pub code: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyRewardInput {
    pub order_id: String,
    pub reward_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveRewardInput {
    pub order_id: String,
}
