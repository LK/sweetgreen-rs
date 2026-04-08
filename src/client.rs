use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use cookie_store::serde::json as cookie_store_json;
use regex::Regex;
use reqwest::header::{
    CONTENT_TYPE, HeaderMap, HeaderValue, LOCATION, ORIGIN, REFERER, USER_AGENT,
};
use reqwest::{Method, StatusCode, redirect::Policy};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::{Url, form_urlencoded};
use uuid::Uuid;

use crate::config::{
    AZURE_API_ACCESS_TOKEN_HEADER, AZURE_API_SCOPE, AZURE_CLIENT_ID, AZURE_OFFLINE_ACCESS_SCOPE,
    AZURE_OPENID_SCOPE, AZURE_POLICY_NAME, AZURE_PROFILE_SCOPE, AZURE_REDIRECT_URI,
    AZURE_TENANT_HOST, AZURE_TENANT_ID, AZURE_VENDOR_ACCESS_TOKEN_HEADER, AZURE_VENDOR_SCOPE,
    BROWSER_USER_AGENT, CSRF_HEADER, DEFAULT_APOLLO_CLIENT_NAME, DEFAULT_APOLLO_CLIENT_VERSION,
    DEFAULT_GRAPHQL_ENDPOINT, DEFAULT_ORDER_APP_VERSION, DEFAULT_ORDER_ORIGIN,
    DEFAULT_ORDER_REFERER, azure_authorize_endpoint, azure_token_endpoint,
};
use crate::debug::write_failure_report;
use crate::error::SweetgreenError;
use crate::graphql::{GraphqlRequest, GraphqlResponse};
use crate::models::{
    AddLineItemMutationData, AddLineItemToCartInput, BagCountQueryData, Cart, CartQueryData,
    CustomizationDataQueryData, EditLineItemInCartInput, EditLineItemMutationData,
    GiftCardBalanceQueryData, LocationSearchMatch, LocationsSearchByStringQueryData,
    MenuContentRestaurantQueryData, MenuIngredient, MenuRestaurant, RedeemGiftCardMutationData,
    RemoveFromCartInput, RemoveLineItemMutationData, RemoveRewardInput, RemoveRewardMutationData,
    Reward, RewardMutationData, RewardsQueryData, SessionInfo, SessionQueryData,
    SignInMutationData, SubmitPromoOrGiftCardCodeInput,
};
use crate::queries;
use crate::state::{AuthMode, AuthState, AuthStatePatch, StateStore};

#[derive(Debug, Clone)]
pub struct SweetgreenClient {
    http: reqwest::Client,
    graphql_endpoint: String,
    state_store: StateStore,
    cookie_store: Arc<CookieStoreMutex>,
    cookie_path: PathBuf,
}

impl SweetgreenClient {
    pub fn new(
        state_store: StateStore,
        graphql_endpoint: Option<String>,
        cookie_path: PathBuf,
    ) -> Result<Self, SweetgreenError> {
        let cookie_store = Arc::new(CookieStoreMutex::new(load_cookie_store(&cookie_path)?));
        let http = reqwest::Client::builder()
            .cookie_provider(Arc::clone(&cookie_store))
            .redirect(Policy::none())
            .build()
            .map_err(SweetgreenError::Http)?;

        Ok(Self {
            http,
            graphql_endpoint: graphql_endpoint
                .unwrap_or_else(|| DEFAULT_GRAPHQL_ENDPOINT.to_string()),
            state_store,
            cookie_store,
            cookie_path,
        })
    }

    pub fn state_store(&self) -> &StateStore {
        &self.state_store
    }

    pub fn load_state(&self) -> Result<AuthState, SweetgreenError> {
        self.state_store.load()
    }

    pub fn clear_state(&self) -> Result<(), SweetgreenError> {
        self.state_store.clear()
    }

    pub fn save_cookies(&self) -> Result<(), SweetgreenError> {
        if let Some(parent) = self.cookie_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SweetgreenError::Auth(format!(
                    "failed to create cookie dir {}: {e}",
                    parent.display()
                ))
            })?;
        }

        let file = File::create(&self.cookie_path).map_err(|e| {
            SweetgreenError::Auth(format!(
                "failed to create cookie file {}: {e}",
                self.cookie_path.display()
            ))
        })?;
        let mut writer = BufWriter::new(file);

        let store = self
            .cookie_store
            .lock()
            .map_err(|_| SweetgreenError::Auth("cookie store lock poisoned".to_string()))?;
        cookie_store_json::save(&store, &mut writer).map_err(|e| {
            SweetgreenError::Auth(format!(
                "failed to serialize cookies to {}: {e}",
                self.cookie_path.display()
            ))
        })?;
        writer.flush().map_err(|e| {
            SweetgreenError::Auth(format!(
                "failed to flush cookie file {}: {e}",
                self.cookie_path.display()
            ))
        })?;

        Ok(())
    }

    pub fn clear_cookies(&self) -> Result<(), SweetgreenError> {
        let mut store = self
            .cookie_store
            .lock()
            .map_err(|_| SweetgreenError::Auth("cookie store lock poisoned".to_string()))?;
        store.clear();
        drop(store);

        if self.cookie_path.exists() {
            fs::remove_file(&self.cookie_path).map_err(|e| {
                SweetgreenError::Auth(format!(
                    "failed to remove cookie file {}: {e}",
                    self.cookie_path.display()
                ))
            })?;
        }

        Ok(())
    }

    pub fn import_browser_cookies(
        &self,
        browser: &str,
        domain: &str,
    ) -> Result<usize, SweetgreenError> {
        let imported = fetch_browser_cookies(browser, domain)?;
        if imported.is_empty() {
            return Ok(0);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut imported_count = 0usize;
        let mut store = self
            .cookie_store
            .lock()
            .map_err(|_| SweetgreenError::Auth("cookie store lock poisoned".to_string()))?;

        for cookie in imported {
            if cookie.name.is_empty() || cookie.domain.is_empty() {
                continue;
            }

            let host = cookie.domain.trim_start_matches('.');
            if host.is_empty() {
                continue;
            }

            let request_url = Url::parse(&format!("https://{host}/")).map_err(|e| {
                SweetgreenError::Auth(format!(
                    "failed to build request URL for imported cookie domain {}: {e}",
                    cookie.domain
                ))
            })?;

            let mut cookie_str = format!("{}={}", cookie.name, cookie.value);
            cookie_str.push_str(&format!("; Domain={}", cookie.domain));
            cookie_str.push_str(&format!("; Path={}", normalize_cookie_path(&cookie.path)));
            if cookie.secure {
                cookie_str.push_str("; Secure");
            }
            if cookie.http_only {
                cookie_str.push_str("; HttpOnly");
            }
            if let Some(expires) = cookie.expires {
                if expires > now {
                    cookie_str.push_str(&format!("; Max-Age={}", expires - now));
                }
            }

            if store.parse(&cookie_str, &request_url).is_ok() {
                imported_count += 1;
            }
        }

        drop(store);
        self.save_cookies()?;
        if imported_count > 0 {
            self.state_store.update(|state| {
                state.auth_mode = AuthMode::Browser;
            })?;
        }
        Ok(imported_count)
    }

    pub async fn auth_refresh(&self) -> Result<AuthState, SweetgreenError> {
        let state = self.state_store.load()?;
        let refresh_token = state
            .refresh_token
            .clone()
            .ok_or(SweetgreenError::MissingAuthState("refresh_token"))?;

        let refreshed = self.refresh_tokens(&refresh_token).await?;
        let mut next = state;
        next.merge(AuthStatePatch {
            auth_mode: Some(AuthMode::Local),
            refresh_token: Some(Some(refreshed.refresh_token.clone())),
            api_authorization_token: Some(Some(refreshed.api_access_token.clone())),
            vendor_authorization_token: Some(Some(refreshed.vendor_access_token.clone())),
            ..AuthStatePatch::default()
        });

        self.state_store.save(&next)?;
        Ok(next)
    }

    pub async fn auth_login(
        &self,
        email: &str,
        code: Option<&str>,
    ) -> Result<AuthState, SweetgreenError> {
        let login_ctx = self.begin_email_code_flow(email).await?;
        self.send_login_code(email, &login_ctx).await?;

        let code = match code {
            Some(code) => code.to_string(),
            None => {
                print!("Enter email login code: ");
                io::stdout()
                    .flush()
                    .map_err(|e| SweetgreenError::Auth(format!("failed to flush stdout: {e}")))?;
                let mut code = String::new();
                io::stdin().read_line(&mut code).map_err(|e| {
                    SweetgreenError::Auth(format!("failed to read login code: {e}"))
                })?;
                code.trim().to_string()
            }
        };

        self.verify_login_code(email, &code, &login_ctx).await?;

        let auth_code = self
            .complete_email_code_flow(email, &code, &login_ctx)
            .await?;
        let api_tokens = self
            .exchange_authorization_code(&auth_code, &login_ctx)
            .await?;
        let vendor_tokens = self
            .exchange_vendor_access_token(&api_tokens.refresh_token)
            .await?;

        let mut state = AuthState {
            auth_mode: AuthMode::Local,
            email: Some(email.to_string()),
            refresh_token: Some(api_tokens.refresh_token.clone()),
            api_authorization_token: Some(api_tokens.access_token.clone()),
            vendor_authorization_token: Some(vendor_tokens.access_token.clone()),
            ..AuthState::default()
        };

        let guest_session = self.get_session(&state).await?;
        state.csrf_token = Some(guest_session.csrf);

        let sign_in_result = self.sign_in(&state).await?;
        if sign_in_result.typename != "SignInSuccess" {
            return Err(SweetgreenError::Auth(format!(
                "sign-in failed with type {}",
                sign_in_result.typename
            )));
        }

        let active_session = self.get_session(&state).await?;
        state.csrf_token = Some(active_session.csrf);
        state.auth_mode = AuthMode::Local;

        self.state_store.save(&state)?;
        Ok(state)
    }

    pub async fn ensure_authenticated_state(&self) -> Result<AuthState, SweetgreenError> {
        let mut state = self.state_store.load()?;

        if let Some(refresh_token) = state.refresh_token.clone() {
            let refreshed = self.refresh_tokens(&refresh_token).await?;
            state.merge(AuthStatePatch {
                refresh_token: Some(Some(refreshed.refresh_token.clone())),
                api_authorization_token: Some(Some(refreshed.api_access_token)),
                vendor_authorization_token: Some(Some(refreshed.vendor_access_token)),
                ..AuthStatePatch::default()
            });
        } else if state.api_authorization_token.is_none()
            || state.vendor_authorization_token.is_none()
        {
            let refresh_token = state
                .refresh_token
                .clone()
                .ok_or(SweetgreenError::MissingAuthState("refresh_token"))?;
            let refreshed = self.refresh_tokens(&refresh_token).await?;
            state.merge(AuthStatePatch {
                refresh_token: Some(Some(refreshed.refresh_token.clone())),
                api_authorization_token: Some(Some(refreshed.api_access_token)),
                vendor_authorization_token: Some(Some(refreshed.vendor_access_token)),
                ..AuthStatePatch::default()
            });
        }

        let guest_session = self.get_session(&state).await?;
        state.csrf_token = Some(guest_session.csrf);

        let sign_in_result = self.sign_in(&state).await?;
        if sign_in_result.typename != "SignInSuccess" {
            return Err(SweetgreenError::Auth(format!(
                "sign-in failed with type {}",
                sign_in_result.typename
            )));
        }

        let active_session = self.get_session(&state).await?;
        state.csrf_token = Some(active_session.csrf);
        state.auth_mode = AuthMode::Local;

        self.state_store.save(&state)?;
        Ok(state)
    }

    pub async fn ensure_cart_state(&self) -> Result<AuthState, SweetgreenError> {
        let state = self.state_store.load()?;
        match state.auth_mode {
            AuthMode::Browser => self.browser_session_state().await,
            AuthMode::Local => self.ensure_authenticated_state().await,
        }
    }

    pub async fn browser_session_state(&self) -> Result<AuthState, SweetgreenError> {
        let session = self.get_session(&AuthState::default()).await?;
        if !session.is_logged_in {
            return Err(SweetgreenError::Auth(
                "browser session is not logged in; import browser cookies first".to_string(),
            ));
        }

        let mut state = self
            .state_store
            .load()
            .unwrap_or_else(|_| AuthState::default());
        state.auth_mode = AuthMode::Browser;
        state.csrf_token = Some(session.csrf);
        state.api_authorization_token = None;
        state.vendor_authorization_token = None;
        self.state_store.save(&state)?;
        Ok(state)
    }

    pub async fn get_session(&self, state: &AuthState) -> Result<SessionInfo, SweetgreenError> {
        let data: SessionQueryData = self
            .graphql(
                state,
                "getSession",
                queries::GET_SESSION_QUERY,
                json!({}),
                false,
            )
            .await?;
        Ok(data.session)
    }

    pub async fn bag_count(&self, state: &AuthState) -> Result<usize, SweetgreenError> {
        let data: BagCountQueryData = self
            .graphql(state, "BagCount", queries::BAG_COUNT_QUERY, json!({}), true)
            .await?;
        Ok(data.cart.map(|cart| cart.line_items.len()).unwrap_or(0))
    }

    pub async fn cart(&self, state: &AuthState) -> Result<Option<Cart>, SweetgreenError> {
        let data: CartQueryData = self
            .graphql(state, "BagCart", queries::BAG_CART_QUERY, json!({}), true)
            .await?;
        Ok(data.cart)
    }

    pub async fn wanted_times(&self, state: &AuthState) -> Result<Vec<String>, SweetgreenError> {
        let data: CartQueryData = self
            .graphql(
                state,
                "BagTimesPolling",
                queries::BAG_TIMES_POLLING_QUERY,
                json!({}),
                true,
            )
            .await?;

        let times = data
            .cart
            .map(|cart| {
                cart.available_wanted_times
                    .into_iter()
                    .map(|item| item.time)
                    .collect()
            })
            .unwrap_or_default();

        Ok(times)
    }

    pub async fn menu_for_restaurant(
        &self,
        restaurant_id: impl Into<String>,
    ) -> Result<MenuRestaurant, SweetgreenError> {
        let id = restaurant_id.into();
        let data: MenuContentRestaurantQueryData = self
            .graphql(
                &AuthState::default(),
                "MenuContentRestaurant",
                queries::MENU_CONTENT_RESTAURANT_QUERY,
                json!({
                    "id": id,
                    "caloriesVersion": "v2",
                }),
                false,
            )
            .await?;

        data.restaurant.ok_or_else(|| {
            SweetgreenError::Auth("menu lookup returned no restaurant for given id".to_string())
        })
    }

    pub async fn search_locations_by_string(
        &self,
        search_string: impl Into<String>,
        show_hidden: Option<bool>,
    ) -> Result<Vec<LocationSearchMatch>, SweetgreenError> {
        let search_string = search_string.into();
        let data: LocationsSearchByStringQueryData = self
            .graphql(
                &AuthState::default(),
                "LocationsSearchBySearchString",
                queries::LOCATIONS_SEARCH_BY_STRING_QUERY,
                json!({
                    "searchString": search_string,
                    "showHidden": show_hidden,
                }),
                false,
            )
            .await?;
        Ok(data.search_locations_by_string)
    }

    pub async fn customization_ingredients_for_product(
        &self,
        state: &AuthState,
        product_id: impl Into<String>,
        restaurant_id: impl Into<String>,
    ) -> Result<Vec<MenuIngredient>, SweetgreenError> {
        let data: CustomizationDataQueryData = self
            .graphql(
                state,
                "CustomizationData",
                queries::CUSTOMIZATION_DATA_QUERY,
                json!({
                    "productId": product_id.into(),
                    "restaurantId": restaurant_id.into(),
                    "caloriesVersion": "v2",
                }),
                true,
            )
            .await?;

        let product = data.product.ok_or_else(|| {
            SweetgreenError::Auth(
                "customization lookup returned no product for given identifiers".to_string(),
            )
        })?;

        let mut ingredients_by_id = BTreeMap::new();
        for ingredient in product.ingredients {
            ingredients_by_id.insert(ingredient.id.clone(), ingredient);
        }

        for group in product.modifier_groups {
            for modification in group.modifications {
                if modification.out_of_stock.unwrap_or(false) {
                    continue;
                }
                if let Some(ingredient) = modification.ingredient {
                    ingredients_by_id
                        .entry(ingredient.id.clone())
                        .or_insert(ingredient);
                }
            }
        }

        Ok(ingredients_by_id.into_values().collect())
    }

    pub async fn add_line_item(
        &self,
        state: &AuthState,
        input: AddLineItemToCartInput,
    ) -> Result<AddLineItemMutationData, SweetgreenError> {
        self.graphql(
            state,
            "AddLineItemToCart",
            queries::ADD_LINE_ITEM_MUTATION,
            json!({ "input": input }),
            true,
        )
        .await
    }

    pub async fn update_line_item(
        &self,
        state: &AuthState,
        input: EditLineItemInCartInput,
    ) -> Result<EditLineItemMutationData, SweetgreenError> {
        self.graphql(
            state,
            "UpdateLineItem",
            queries::UPDATE_LINE_ITEM_MUTATION,
            json!({ "input": input }),
            true,
        )
        .await
    }

    pub async fn remove_line_item(
        &self,
        state: &AuthState,
        line_item_id: impl Into<String>,
    ) -> Result<RemoveLineItemMutationData, SweetgreenError> {
        let input = RemoveFromCartInput {
            line_item_id: line_item_id.into(),
        };

        self.graphql(
            state,
            "RemoveLineItem",
            queries::REMOVE_LINE_ITEM_MUTATION,
            json!({ "input": input }),
            true,
        )
        .await
    }

    pub async fn clear_cart(&self, state: &AuthState) -> Result<usize, SweetgreenError> {
        let cart = self.cart(state).await?;
        let Some(cart) = cart else {
            return Ok(0);
        };

        let mut removed = 0usize;
        for line_item in cart.line_items {
            let _ = self.remove_line_item(state, line_item.id).await?;
            removed += 1;
        }

        Ok(removed)
    }

    pub async fn apply_promo(
        &self,
        state: &AuthState,
        code: impl Into<String>,
    ) -> Result<crate::models::PromoMutationData, SweetgreenError> {
        let input = SubmitPromoOrGiftCardCodeInput { code: code.into() };
        self.graphql(
            state,
            "applyPromoCode",
            queries::APPLY_PROMO_MUTATION,
            json!({ "input": input }),
            true,
        )
        .await
    }

    pub async fn rewards(&self, state: &AuthState) -> Result<Vec<Reward>, SweetgreenError> {
        let data: RewardsQueryData = self
            .graphql(
                state,
                "BagRewards",
                queries::BAG_REWARDS_QUERY,
                json!({}),
                true,
            )
            .await?;
        Ok(data.rewards)
    }

    pub async fn apply_reward(
        &self,
        state: &AuthState,
        order_id: impl Into<String>,
        reward_id: impl Into<String>,
    ) -> Result<RewardMutationData, SweetgreenError> {
        self.graphql(
            state,
            "applyBagReward",
            queries::APPLY_REWARD_MUTATION,
            json!({
                "input": {
                    "orderId": order_id.into(),
                    "rewardId": reward_id.into()
                }
            }),
            true,
        )
        .await
    }

    pub async fn remove_reward(
        &self,
        state: &AuthState,
        order_id: impl Into<String>,
    ) -> Result<RemoveRewardMutationData, SweetgreenError> {
        let input = RemoveRewardInput {
            order_id: order_id.into(),
        };

        self.graphql(
            state,
            "removeBagReward",
            queries::REMOVE_REWARD_MUTATION,
            json!({ "input": input }),
            true,
        )
        .await
    }

    pub async fn gift_card_balance(
        &self,
        state: &AuthState,
    ) -> Result<GiftCardBalanceQueryData, SweetgreenError> {
        self.graphql(
            state,
            "BagGiftCardBalance",
            queries::GIFT_CARD_BALANCE_QUERY,
            json!({}),
            true,
        )
        .await
    }

    pub async fn redeem_gift_card(
        &self,
        state: &AuthState,
        code: impl Into<String>,
        reg_code: Option<String>,
    ) -> Result<RedeemGiftCardMutationData, SweetgreenError> {
        self.graphql(
            state,
            "RedeemGiftCardInBag",
            queries::REDEEM_GIFT_CARD_MUTATION,
            json!({ "code": code.into(), "regCode": reg_code }),
            true,
        )
        .await
    }

    async fn sign_in(
        &self,
        state: &AuthState,
    ) -> Result<crate::models::SignInUnionResult, SweetgreenError> {
        let data: SignInMutationData = self
            .graphql(state, "SignIn", queries::SIGN_IN_MUTATION, json!({}), true)
            .await?;
        Ok(data.sign_in)
    }

    async fn graphql<V, T>(
        &self,
        state: &AuthState,
        operation_name: &str,
        query: &str,
        variables: V,
        include_auth_headers: bool,
    ) -> Result<T, SweetgreenError>
    where
        V: Serialize,
        T: DeserializeOwned,
    {
        let request = GraphqlRequest {
            operation_name,
            query,
            variables,
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ORIGIN, HeaderValue::from_static(DEFAULT_ORDER_ORIGIN));
        headers.insert(REFERER, HeaderValue::from_static(DEFAULT_ORDER_REFERER));
        headers.insert(USER_AGENT, HeaderValue::from_static(BROWSER_USER_AGENT));
        headers.insert(
            "apollographql-client-name",
            HeaderValue::from_static(DEFAULT_APOLLO_CLIENT_NAME),
        );
        headers.insert(
            "apollographql-client-version",
            HeaderValue::from_static(DEFAULT_APOLLO_CLIENT_VERSION),
        );

        if include_auth_headers {
            if let Some(csrf) = state.csrf_token.as_deref() {
                headers.insert(
                    CSRF_HEADER,
                    HeaderValue::from_str(csrf).map_err(|e| {
                        SweetgreenError::InvalidArgument(format!("invalid csrf header: {e}"))
                    })?,
                );
            }

            if let Some(token) = state.api_authorization_token.as_deref() {
                headers.insert(
                    AZURE_API_ACCESS_TOKEN_HEADER,
                    HeaderValue::from_str(token).map_err(|e| {
                        SweetgreenError::InvalidArgument(format!("invalid api token header: {e}"))
                    })?,
                );
            }

            if let Some(token) = state.vendor_authorization_token.as_deref() {
                headers.insert(
                    AZURE_VENDOR_ACCESS_TOKEN_HEADER,
                    HeaderValue::from_str(token).map_err(|e| {
                        SweetgreenError::InvalidArgument(format!(
                            "invalid vendor token header: {e}"
                        ))
                    })?,
                );
            }
        }

        let response = self
            .http
            .post(&self.graphql_endpoint)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let response = ensure_success_status(response, "graphql_request").await?;

        let payload: GraphqlResponse<T> = parse_json_response(
            response,
            "failed to parse GraphQL response",
            "graphql_response_parse",
        )
        .await?;

        if !payload.errors.is_empty() {
            let joined = payload
                .errors
                .iter()
                .map(|err| err.summary())
                .collect::<Vec<_>>()
                .join("; ");
            let report = write_failure_report(
                "graphql_application_errors",
                &json!({
                    "operation_name": operation_name,
                    "errors": payload.errors,
                    "has_data": payload.data.is_some(),
                }),
            );

            let report_hint = report
                .as_ref()
                .map(|path| format!("; report={}", path.display()))
                .unwrap_or_default();

            return Err(SweetgreenError::GraphqlErrors(format!(
                "{joined}{report_hint}"
            )));
        }

        payload.data.ok_or(SweetgreenError::MissingGraphqlData)
    }

    async fn begin_email_code_flow(
        &self,
        email: &str,
    ) -> Result<LoginFlowContext, SweetgreenError> {
        let code_verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let code_challenge = pkce_challenge(&code_verifier);
        let client_request_id = Uuid::new_v4().to_string();
        let nonce = Uuid::new_v4().to_string();

        let state_json = json!({
            "id": Uuid::new_v4().to_string(),
            "meta": { "interactionType": "redirect" }
        });
        let state = URL_SAFE_NO_PAD.encode(state_json.to_string().as_bytes());

        let scope = format!(
            "{} {} {} {}",
            AZURE_OPENID_SCOPE, AZURE_API_SCOPE, AZURE_PROFILE_SCOPE, AZURE_OFFLINE_ACCESS_SCOPE
        );

        let authorize_url = {
            let mut url = Url::parse(&azure_authorize_endpoint()).map_err(|e| {
                SweetgreenError::InvalidArgument(format!("failed to build authorize URL: {e}"))
            })?;

            url.query_pairs_mut()
                .append_pair("client_id", AZURE_CLIENT_ID)
                .append_pair("scope", &scope)
                .append_pair("redirect_uri", AZURE_REDIRECT_URI)
                .append_pair("client-request-id", &client_request_id)
                .append_pair("response_mode", "fragment")
                .append_pair("response_type", "code")
                .append_pair("x-client-SKU", "msal.js.browser")
                .append_pair("x-client-VER", "3.6.0")
                .append_pair("client_info", "1")
                .append_pair("code_challenge", &code_challenge)
                .append_pair("code_challenge_method", "S256")
                .append_pair("nonce", &nonce)
                .append_pair("state", &state)
                .append_pair("orderAppVersion", DEFAULT_ORDER_APP_VERSION);

            url
        };

        let response = self
            .http
            .request(Method::GET, authorize_url.clone())
            .header(REFERER, DEFAULT_ORDER_REFERER)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let response = ensure_status(response, StatusCode::OK, "azure_authorize").await?;

        let html = response.text().await.map_err(SweetgreenError::Http)?;
        let settings = extract_azure_settings(&html)?;

        let self_asserted_url = format!(
            "https://{}/{}/{}/SelfAsserted?tx={}&p={}",
            AZURE_TENANT_HOST,
            AZURE_TENANT_ID,
            AZURE_POLICY_NAME,
            settings.trans_id,
            AZURE_POLICY_NAME
        );

        let initial_form = form_encode(&[("request_type", "RESPONSE"), ("signInName", email)]);

        let response = self
            .http
            .request(Method::POST, &self_asserted_url)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(ORIGIN, format!("https://{}", AZURE_TENANT_HOST))
            .header(REFERER, authorize_url.to_string())
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .header(CSRF_HEADER, settings.csrf.clone())
            .header("x-requested-with", "XMLHttpRequest")
            .body(initial_form)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let _ = ensure_status(response, StatusCode::OK, "azure_self_asserted_init").await?;

        let confirmed_combined_url = build_confirmed_url(
            "CombinedSigninAndSignup",
            &settings.csrf,
            &settings.trans_id,
            &settings.page_view_id,
        )?;

        let response = self
            .http
            .request(Method::GET, &confirmed_combined_url)
            .header(REFERER, authorize_url.to_string())
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let response = ensure_status(
            response,
            StatusCode::OK,
            "azure_combined_signin_signup_confirmed",
        )
        .await?;

        let html = response.text().await.map_err(SweetgreenError::Http)?;
        let verification_settings = extract_azure_settings(&html)?;

        Ok(LoginFlowContext {
            code_verifier,
            client_request_id,
            scope,
            trans_id: verification_settings.trans_id,
            verification_csrf: verification_settings.csrf,
            verification_page_view_id: verification_settings.page_view_id,
            combined_confirmed_url: confirmed_combined_url,
        })
    }

    async fn send_login_code(
        &self,
        email: &str,
        ctx: &LoginFlowContext,
    ) -> Result<(), SweetgreenError> {
        let url = format!(
            "https://{}/{}/{}/SelfAsserted/DisplayControlAction/vbeta/emailVerificationSSPRControl/SendCode?tx={}&p={}",
            AZURE_TENANT_HOST, AZURE_TENANT_ID, AZURE_POLICY_NAME, ctx.trans_id, AZURE_POLICY_NAME
        );

        let body = form_encode(&[("emailAddress", email)]);

        let response = self
            .http
            .request(Method::POST, &url)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(ORIGIN, format!("https://{}", AZURE_TENANT_HOST))
            .header(REFERER, &ctx.combined_confirmed_url)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .header(CSRF_HEADER, ctx.verification_csrf.clone())
            .header("x-requested-with", "XMLHttpRequest")
            .body(body)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let _ = ensure_status(response, StatusCode::OK, "azure_send_code").await?;

        Ok(())
    }

    async fn verify_login_code(
        &self,
        email: &str,
        code: &str,
        ctx: &LoginFlowContext,
    ) -> Result<(), SweetgreenError> {
        let url = format!(
            "https://{}/{}/{}/SelfAsserted/DisplayControlAction/vbeta/emailVerificationSSPRControl/VerifyCode?tx={}&p={}",
            AZURE_TENANT_HOST, AZURE_TENANT_ID, AZURE_POLICY_NAME, ctx.trans_id, AZURE_POLICY_NAME
        );

        let body = form_encode(&[("emailAddress", email), ("EmailVerificationCode", code)]);

        let response = self
            .http
            .request(Method::POST, &url)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(ORIGIN, format!("https://{}", AZURE_TENANT_HOST))
            .header(REFERER, &ctx.combined_confirmed_url)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .header(CSRF_HEADER, ctx.verification_csrf.clone())
            .header("x-requested-with", "XMLHttpRequest")
            .body(body)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let _ = ensure_status(response, StatusCode::OK, "azure_verify_code").await?;

        Ok(())
    }

    async fn complete_email_code_flow(
        &self,
        email: &str,
        code: &str,
        ctx: &LoginFlowContext,
    ) -> Result<String, SweetgreenError> {
        let self_asserted_url = format!(
            "https://{}/{}/{}/SelfAsserted?tx={}&p={}",
            AZURE_TENANT_HOST, AZURE_TENANT_ID, AZURE_POLICY_NAME, ctx.trans_id, AZURE_POLICY_NAME
        );

        let body = form_encode(&[
            ("emailAddress", email),
            ("EmailVerificationCode", code),
            ("request_type", "RESPONSE"),
        ]);

        let response = self
            .http
            .request(Method::POST, &self_asserted_url)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(ORIGIN, format!("https://{}", AZURE_TENANT_HOST))
            .header(REFERER, &ctx.combined_confirmed_url)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .header(CSRF_HEADER, ctx.verification_csrf.clone())
            .header("x-requested-with", "XMLHttpRequest")
            .body(body)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let _ = ensure_status(response, StatusCode::OK, "azure_self_asserted_complete").await?;

        let confirmed_self_asserted_url = build_confirmed_url(
            "SelfAsserted",
            &ctx.verification_csrf,
            &ctx.trans_id,
            &ctx.verification_page_view_id,
        )?;

        let response = self
            .http
            .request(Method::GET, &confirmed_self_asserted_url)
            .header(REFERER, &ctx.combined_confirmed_url)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let response =
            ensure_status(response, StatusCode::FOUND, "azure_self_asserted_confirmed").await?;

        let location = response
            .headers()
            .get(LOCATION)
            .ok_or_else(|| SweetgreenError::Auth("missing redirect location".to_string()))?
            .to_str()
            .map_err(|e| SweetgreenError::Auth(format!("invalid redirect location: {e}")))?
            .to_string();

        extract_auth_code_from_redirect(&location)
    }

    async fn exchange_authorization_code(
        &self,
        auth_code: &str,
        ctx: &LoginFlowContext,
    ) -> Result<TokenResponse, SweetgreenError> {
        let response = self
            .http
            .post(azure_token_endpoint())
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded;charset=utf-8",
            )
            .header(ORIGIN, DEFAULT_ORDER_ORIGIN)
            .header(REFERER, DEFAULT_ORDER_REFERER)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .body(form_encode(&[
                ("client_id", AZURE_CLIENT_ID),
                ("redirect_uri", AZURE_REDIRECT_URI),
                ("scope", &ctx.scope),
                ("code", auth_code),
                ("x-client-SKU", "msal.js.browser"),
                ("x-client-VER", "3.6.0"),
                ("code_verifier", &ctx.code_verifier),
                ("grant_type", "authorization_code"),
                ("client_info", "1"),
                ("client-request-id", &ctx.client_request_id),
            ]))
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let response = ensure_status(
            response,
            StatusCode::OK,
            "azure_token_exchange_authorization_code",
        )
        .await?;

        let token: TokenResponse = parse_json_response(
            response,
            "failed to parse authorization_code token response",
            "azure_token_exchange_authorization_code_parse",
        )
        .await?;
        if token.access_token.is_empty() || token.refresh_token.is_empty() {
            return Err(SweetgreenError::Auth(
                "authorization_code token response missing access or refresh token".to_string(),
            ));
        }

        Ok(token)
    }

    async fn exchange_vendor_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse, SweetgreenError> {
        let scope = format!(
            "{} {} {} {}",
            AZURE_OPENID_SCOPE, AZURE_VENDOR_SCOPE, AZURE_PROFILE_SCOPE, AZURE_OFFLINE_ACCESS_SCOPE
        );

        let response = self
            .http
            .post(azure_token_endpoint())
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded;charset=utf-8",
            )
            .header(ORIGIN, DEFAULT_ORDER_ORIGIN)
            .header(REFERER, DEFAULT_ORDER_REFERER)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .body(form_encode(&[
                ("client_id", AZURE_CLIENT_ID),
                ("scope", &scope),
                ("grant_type", "refresh_token"),
                ("client_info", "1"),
                ("x-client-SKU", "msal.js.browser"),
                ("x-client-VER", "3.6.0"),
                ("refresh_token", refresh_token),
            ]))
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let response =
            ensure_status(response, StatusCode::OK, "azure_token_exchange_vendor").await?;

        let token: TokenResponse = parse_json_response(
            response,
            "failed to parse vendor token response",
            "azure_token_exchange_vendor_parse",
        )
        .await?;
        if token.access_token.is_empty() {
            return Err(SweetgreenError::Auth(
                "vendor token response missing access token".to_string(),
            ));
        }

        Ok(token)
    }

    async fn refresh_tokens(
        &self,
        refresh_token: &str,
    ) -> Result<RefreshedTokens, SweetgreenError> {
        let api_scope = format!(
            "{} {} {} {}",
            AZURE_OPENID_SCOPE, AZURE_API_SCOPE, AZURE_PROFILE_SCOPE, AZURE_OFFLINE_ACCESS_SCOPE
        );
        let vendor_scope = format!(
            "{} {} {} {}",
            AZURE_OPENID_SCOPE, AZURE_VENDOR_SCOPE, AZURE_PROFILE_SCOPE, AZURE_OFFLINE_ACCESS_SCOPE
        );

        let api = self
            .token_refresh_request(refresh_token, &api_scope)
            .await?;
        let vendor = self
            .token_refresh_request(refresh_token, &vendor_scope)
            .await?;

        Ok(RefreshedTokens {
            refresh_token: api.refresh_token,
            api_access_token: api.access_token,
            vendor_access_token: vendor.access_token,
        })
    }

    async fn token_refresh_request(
        &self,
        refresh_token: &str,
        scope: &str,
    ) -> Result<TokenResponse, SweetgreenError> {
        let response = self
            .http
            .post(azure_token_endpoint())
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded;charset=utf-8",
            )
            .header(ORIGIN, DEFAULT_ORDER_ORIGIN)
            .header(REFERER, DEFAULT_ORDER_REFERER)
            .header(USER_AGENT, BROWSER_USER_AGENT)
            .body(form_encode(&[
                ("client_id", AZURE_CLIENT_ID),
                ("scope", scope),
                ("grant_type", "refresh_token"),
                ("client_info", "1"),
                ("x-client-SKU", "msal.js.browser"),
                ("x-client-VER", "3.6.0"),
                ("refresh_token", refresh_token),
            ]))
            .send()
            .await
            .map_err(SweetgreenError::Http)?;

        let response = ensure_status(response, StatusCode::OK, "azure_token_refresh").await?;

        let token: TokenResponse = parse_json_response(
            response,
            "failed to parse refresh token response",
            "azure_token_refresh_parse",
        )
        .await?;
        if token.access_token.is_empty() || token.refresh_token.is_empty() {
            return Err(SweetgreenError::Auth(
                "refresh token response missing access or refresh token".to_string(),
            ));
        }

        Ok(token)
    }
}

#[derive(Debug, Clone)]
struct LoginFlowContext {
    code_verifier: String,
    client_request_id: String,
    scope: String,
    trans_id: String,
    verification_csrf: String,
    verification_page_view_id: String,
    combined_confirmed_url: String,
}

#[derive(Debug, Clone)]
struct RefreshedTokens {
    refresh_token: String,
    api_access_token: String,
    vendor_access_token: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(rename = "access_token", alias = "accessToken")]
    access_token: String,
    #[serde(default, rename = "refresh_token", alias = "refreshToken")]
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzureSettings {
    csrf: String,
    trans_id: String,
    page_view_id: String,
}

fn extract_azure_settings(html: &str) -> Result<AzureSettings, SweetgreenError> {
    let re = Regex::new(r"(?s)var SETTINGS = (\{.*?\});")
        .map_err(|e| SweetgreenError::Auth(format!("failed to compile settings regex: {e}")))?;

    let settings_json = re
        .captures(html)
        .and_then(|captures| captures.get(1).map(|m| m.as_str()))
        .ok_or_else(|| SweetgreenError::Auth("failed to extract SETTINGS JSON".to_string()))?;

    serde_json::from_str(settings_json)
        .map_err(|e| SweetgreenError::Auth(format!("failed to parse SETTINGS JSON: {e}")))
}

fn build_confirmed_url(
    page_id: &str,
    csrf: &str,
    tx: &str,
    page_view_id: &str,
) -> Result<String, SweetgreenError> {
    let diags = json!({
        "pageViewId": page_view_id,
        "pageId": page_id,
        "trace": []
    })
    .to_string();

    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("csrf_token", csrf);
    serializer.append_pair("tx", tx);
    serializer.append_pair("p", AZURE_POLICY_NAME);
    serializer.append_pair("diags", &diags);

    let query = serializer.finish();

    Ok(format!(
        "https://{}/{}/{}/api/{}/confirmed?{}",
        AZURE_TENANT_HOST, AZURE_TENANT_ID, AZURE_POLICY_NAME, page_id, query
    ))
}

fn extract_auth_code_from_redirect(location: &str) -> Result<String, SweetgreenError> {
    let url = Url::parse(location)
        .map_err(|e| SweetgreenError::Auth(format!("invalid redirect URL: {e}")))?;

    let fragment = url
        .fragment()
        .ok_or_else(|| SweetgreenError::Auth("redirect URL missing fragment".to_string()))?;

    let params: HashMap<_, _> = form_urlencoded::parse(fragment.as_bytes())
        .into_owned()
        .collect();

    params
        .get("code")
        .cloned()
        .ok_or_else(|| SweetgreenError::Auth("redirect fragment missing code".to_string()))
}

fn form_encode(pairs: &[(&str, &str)]) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(key, value);
    }
    serializer.finish()
}

fn pkce_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let digest = hasher.finalize();
    URL_SAFE_NO_PAD.encode(digest)
}

async fn parse_json_response<T: DeserializeOwned>(
    response: reqwest::Response,
    context: &str,
    stage: &str,
) -> Result<T, SweetgreenError> {
    let url = response.url().to_string();
    let status = response.status();
    let headers = headers_to_json(response.headers());

    let body = match response.bytes().await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => {
            let report = write_failure_report(
                stage,
                &json!({
                    "context": context,
                    "url": url,
                    "status": status.as_u16(),
                    "status_text": status.canonical_reason().unwrap_or(""),
                    "headers": headers,
                    "body_read_error": e.to_string(),
                }),
            );

            let report_hint = report
                .as_ref()
                .map(|path| format!("; report={}", path.display()))
                .unwrap_or_default();

            return Err(SweetgreenError::Auth(format!(
                "{context}: failed to read response body: {e}{report_hint}"
            )));
        }
    };

    serde_json::from_str(&body).map_err(|e| {
        let report = write_failure_report(
            stage,
            &json!({
                "context": context,
                "url": url,
                "status": status.as_u16(),
                "status_text": status.canonical_reason().unwrap_or(""),
                "headers": headers,
                "parse_error": e.to_string(),
                "body": body,
            }),
        );

        let report_hint = report
            .as_ref()
            .map(|path| format!("; report={}", path.display()))
            .unwrap_or_default();

        SweetgreenError::Auth(format!("{context}: {e}{report_hint}"))
    })
}

async fn ensure_status(
    response: reqwest::Response,
    expected: StatusCode,
    stage: &str,
) -> Result<reqwest::Response, SweetgreenError> {
    let status = response.status();
    if status == expected {
        return Ok(response);
    }

    let url = response.url().to_string();
    let headers = headers_to_json(response.headers());
    let body = match response.bytes().await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => {
            let report = write_failure_report(
                stage,
                &json!({
                    "expected_status": expected.as_u16(),
                    "status": status.as_u16(),
                    "status_text": status.canonical_reason().unwrap_or(""),
                    "url": url,
                    "headers": headers,
                    "body_read_error": e.to_string(),
                }),
            );

            let report_hint = report
                .as_ref()
                .map(|path| format!("; report={}", path.display()))
                .unwrap_or_default();

            return Err(SweetgreenError::Auth(format!(
                "{stage}: expected status {expected}, got {status}; failed to read body: {e}{report_hint}"
            )));
        }
    };

    let report = write_failure_report(
        stage,
        &json!({
            "expected_status": expected.as_u16(),
            "status": status.as_u16(),
            "status_text": status.canonical_reason().unwrap_or(""),
            "url": url,
            "headers": headers,
            "body": body,
        }),
    );

    let report_hint = report
        .as_ref()
        .map(|path| format!("; report={}", path.display()))
        .unwrap_or_default();

    Err(SweetgreenError::Auth(format!(
        "{stage}: expected status {expected}, got {status}{report_hint}"
    )))
}

async fn ensure_success_status(
    response: reqwest::Response,
    stage: &str,
) -> Result<reqwest::Response, SweetgreenError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let url = response.url().to_string();
    let headers = headers_to_json(response.headers());
    let body = match response.bytes().await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(e) => {
            let report = write_failure_report(
                stage,
                &json!({
                    "status": status.as_u16(),
                    "status_text": status.canonical_reason().unwrap_or(""),
                    "url": url,
                    "headers": headers,
                    "body_read_error": e.to_string(),
                }),
            );

            let report_hint = report
                .as_ref()
                .map(|path| format!("; report={}", path.display()))
                .unwrap_or_default();

            return Err(SweetgreenError::Auth(format!(
                "{stage}: request failed with status {status}; failed to read body: {e}{report_hint}"
            )));
        }
    };

    let report = write_failure_report(
        stage,
        &json!({
            "status": status.as_u16(),
            "status_text": status.canonical_reason().unwrap_or(""),
            "url": url,
            "headers": headers,
            "body": body,
        }),
    );

    let report_hint = report
        .as_ref()
        .map(|path| format!("; report={}", path.display()))
        .unwrap_or_default();

    Err(SweetgreenError::Auth(format!(
        "{stage}: request failed with status {status}{report_hint}"
    )))
}

fn headers_to_json(headers: &HeaderMap) -> Value {
    let mut map = serde_json::Map::new();
    for (name, value) in headers {
        let raw = value
            .to_str()
            .map(|text| text.to_string())
            .unwrap_or_else(|_| format!("{:?}", value.as_bytes()));
        map.insert(name.as_str().to_string(), Value::String(raw));
    }
    Value::Object(map)
}

fn load_cookie_store(path: &Path) -> Result<CookieStore, SweetgreenError> {
    if !path.exists() {
        return Ok(CookieStore::default());
    }

    let file = File::open(path).map_err(|e| {
        SweetgreenError::Auth(format!(
            "failed to open cookie file {}: {e}",
            path.display()
        ))
    })?;
    let reader = BufReader::new(file);

    cookie_store_json::load(reader).map_err(|e| {
        SweetgreenError::Auth(format!(
            "failed to parse cookie file {}: {e}",
            path.display()
        ))
    })
}

fn fetch_browser_cookies(
    browser: &str,
    domain: &str,
) -> Result<Vec<ImportedBrowserCookie>, SweetgreenError> {
    let mut failures = Vec::new();

    match fetch_browser_cookies_with_python("python3", browser, domain) {
        Ok(cookies) => return Ok(cookies),
        Err(err) => failures.push(format!("python3: {}", describe_browser_import_error(err))),
    }

    if command_available("uv") {
        match fetch_browser_cookies_with_uv(browser, domain) {
            Ok(cookies) => return Ok(cookies),
            Err(err) => failures.push(format!("uv: {}", describe_browser_import_error(err))),
        }
    }

    match ensure_browser_cookie_helper_python() {
        Ok(helper_python) => {
            match fetch_browser_cookies_with_python(helper_python.as_os_str(), browser, domain) {
                Ok(cookies) => return Ok(cookies),
                Err(err) => failures.push(format!(
                    "helper python ({}): {}",
                    helper_python.display(),
                    describe_browser_import_error(err)
                )),
            }
        }
        Err(err) => failures.push(format!("helper setup failed: {err}")),
    }

    Err(SweetgreenError::Auth(format!(
        "browser cookie import failed for browser={} domain={}; attempts={}",
        browser,
        domain,
        failures.join(" | ")
    )))
}

fn fetch_browser_cookies_with_python(
    python_bin: impl AsRef<std::ffi::OsStr>,
    browser: &str,
    domain: &str,
) -> Result<Vec<ImportedBrowserCookie>, BrowserCookieImportError> {
    let output = Command::new(python_bin)
        .arg("-c")
        .arg(BROWSER_COOKIE_IMPORT_SCRIPT)
        .arg(browser)
        .arg(domain)
        .output()
        .map_err(|e| BrowserCookieImportError::Failed(format!("failed to execute python: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stderr.contains("missing dependency browser-cookie3") {
            return Err(BrowserCookieImportError::MissingDependency);
        }

        let mut msg = format!(
            "browser cookie import failed for browser={} domain={}",
            browser, domain
        );
        if !stderr.is_empty() {
            msg.push_str(&format!("; stderr={stderr}"));
        }
        if !stdout.is_empty() {
            msg.push_str(&format!("; stdout={stdout}"));
        }
        return Err(BrowserCookieImportError::Failed(msg));
    }

    serde_json::from_slice::<Vec<ImportedBrowserCookie>>(&output.stdout).map_err(|e| {
        BrowserCookieImportError::Failed(format!(
            "failed to parse imported browser cookie JSON: {e}"
        ))
    })
}

fn fetch_browser_cookies_with_uv(
    browser: &str,
    domain: &str,
) -> Result<Vec<ImportedBrowserCookie>, BrowserCookieImportError> {
    let output = Command::new("uv")
        .arg("run")
        .arg("--with")
        .arg("browser-cookie3")
        .arg("python")
        .arg("-c")
        .arg(BROWSER_COOKIE_IMPORT_SCRIPT)
        .arg(browser)
        .arg(domain)
        .output()
        .map_err(|e| BrowserCookieImportError::Failed(format!("failed to execute uv: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut msg = format!(
            "browser cookie import via uv failed for browser={} domain={}",
            browser, domain
        );
        if !stderr.is_empty() {
            msg.push_str(&format!("; stderr={stderr}"));
        }
        if !stdout.is_empty() {
            msg.push_str(&format!("; stdout={stdout}"));
        }
        return Err(BrowserCookieImportError::Failed(msg));
    }

    serde_json::from_slice::<Vec<ImportedBrowserCookie>>(&output.stdout).map_err(|e| {
        BrowserCookieImportError::Failed(format!(
            "failed to parse imported browser cookie JSON from uv: {e}"
        ))
    })
}

fn ensure_browser_cookie_helper_python() -> Result<PathBuf, SweetgreenError> {
    let helper_root = if let Some(home_dir) = std::env::var_os("HOME") {
        PathBuf::from(home_dir).join(".sweetgreen").join("pycookie")
    } else {
        PathBuf::from(".sweetgreen").join("pycookie")
    };
    let helper_python = helper_root.join("bin").join("python");

    if python_has_browser_cookie3(&helper_python) {
        return Ok(helper_python);
    }

    if let Some(parent) = helper_root.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            SweetgreenError::Auth(format!(
                "failed to create helper parent dir {}: {e}",
                parent.display()
            ))
        })?;
    }

    let venv_status = Command::new("python3")
        .arg("-m")
        .arg("venv")
        .arg(&helper_root)
        .status()
        .map_err(|e| SweetgreenError::Auth(format!("failed to create helper venv: {e}")))?;
    if !venv_status.success() {
        return Err(SweetgreenError::Auth(
            "failed to create helper venv for browser cookie import".to_string(),
        ));
    }

    let pip_status = Command::new(&helper_python)
        .arg("-m")
        .arg("pip")
        .arg("install")
        .arg("--disable-pip-version-check")
        .arg("-q")
        .arg("browser-cookie3")
        .status()
        .map_err(|e| SweetgreenError::Auth(format!("failed to install browser-cookie3: {e}")))?;
    if !pip_status.success() {
        return Err(SweetgreenError::Auth(
            "failed to install browser-cookie3 into helper venv".to_string(),
        ));
    }

    if !python_has_browser_cookie3(&helper_python) {
        return Err(SweetgreenError::Auth(
            "browser-cookie3 still unavailable after helper venv setup".to_string(),
        ));
    }

    Ok(helper_python)
}

fn python_has_browser_cookie3(python_bin: &Path) -> bool {
    let output = Command::new(python_bin)
        .arg("-c")
        .arg("import browser_cookie3")
        .output();

    matches!(output, Ok(output) if output.status.success())
}

fn command_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn normalize_cookie_path(path: &str) -> &str {
    if path.is_empty() { "/" } else { path }
}

#[derive(Debug, Deserialize)]
struct ImportedBrowserCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    secure: bool,
    http_only: bool,
    expires: Option<i64>,
}

enum BrowserCookieImportError {
    MissingDependency,
    Failed(String),
}

fn describe_browser_import_error(err: BrowserCookieImportError) -> String {
    match err {
        BrowserCookieImportError::MissingDependency => {
            "missing dependency browser-cookie3".to_string()
        }
        BrowserCookieImportError::Failed(message) => message,
    }
}

const BROWSER_COOKIE_IMPORT_SCRIPT: &str = r#"
import json
import sys

browser = sys.argv[1]
domain = sys.argv[2].lstrip('.').lower()

try:
    import browser_cookie3
except Exception as exc:
    print(f"missing dependency browser-cookie3: {exc}", file=sys.stderr)
    sys.exit(2)

class Dia(browser_cookie3.ChromiumBased):
    def __init__(self, cookie_file=None, domain_name="", key_file=None):
        args = {
            'osx_cookies': [
                '~/Library/Application Support/Dia/User Data/Default/Cookies',
                '~/Library/Application Support/Dia/User Data/Profile */Cookies',
            ],
            'os_crypt_name': 'chrome',
            'osx_key_service': 'Dia Safe Storage',
            'osx_key_user': 'Dia',
        }
        super().__init__(
            browser='Dia',
            cookie_file=cookie_file,
            domain_name=domain_name,
            key_file=key_file,
            **args,
        )

def dia(cookie_file=None, domain_name="", key_file=None):
    return Dia(cookie_file, domain_name, key_file).load()

if browser == 'dia':
    extractor = dia
elif hasattr(browser_cookie3, browser):
    extractor = getattr(browser_cookie3, browser)
else:
    print(f"unsupported browser '{browser}'", file=sys.stderr)
    sys.exit(3)

try:
    jar = extractor(domain_name=domain)
except TypeError:
    jar = extractor()

cookies = []
for cookie in jar:
    raw_domain = (cookie.domain or '')
    cookie_domain = raw_domain.lstrip('.').lower()
    if domain and domain not in cookie_domain:
        continue

    rest = getattr(cookie, '_rest', {}) or {}
    http_only = bool(rest.get('HttpOnly')) or bool(rest.get('httponly'))
    if hasattr(cookie, 'has_nonstandard_attr'):
        try:
            http_only = http_only or bool(cookie.has_nonstandard_attr('HttpOnly'))
        except Exception:
            pass

    expires = None
    if cookie.expires:
        try:
            expires = int(cookie.expires)
        except Exception:
            expires = None

    cookies.append({
        'name': cookie.name or '',
        'value': cookie.value or '',
        'domain': raw_domain,
        'path': cookie.path or '/',
        'secure': bool(cookie.secure),
        'http_only': http_only,
        'expires': expires,
    })

print(json.dumps(cookies))
"#;
