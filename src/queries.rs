pub const GET_SESSION_QUERY: &str = r#"
query getSession {
  session {
    csrf
    isLoggedIn
  }
}
"#;

pub const MENU_CONTENT_RESTAURANT_QUERY: &str = r#"
query MenuContentRestaurant(
  $id: ID!
  $costChannel: CostChannel
  $caloriesVersion: CaloriesVersion
) {
  restaurant(id: $id) {
    id
    __typename
    slug
    name
    city
    state
    zipCode
    address
    phone
    isOutpost
    showDeliveryFeeDisclosure
    sanitaryGradeDisplayText
    sanitaryGradeUrl
    hours {
      formatted
    }
    utcOffset
    isAcceptingOrders
    notAcceptingOrdersReason
    deliveryFee
    flexMessage
    menu {
      id
      __typename
      categories {
        id
        __typename
        name
        isCustom
        description
        products {
          id
          __typename
          baseProduct {
            __typename
            id
            slug
            proteinG
            totalCarbsG
            totalFatG
            calories
          }
          asset {
            url
          }
          calories
          categoryId
          cost: channelCost(costChannel: $costChannel)
          description
          ingredients {
            id
            __typename
            name
            kind
            asset {
              url
            }
            calories(version: $caloriesVersion)
            totalCarbsG
            proteinG
            totalFatG
          }
          isModifiable
          isCustom
          customType
          isSalad
          label {
            id
            __typename
            name
          }
          dietaryProperties {
            id
            __typename
            name
          }
          name
          outOfStock
          restaurantId
          slug
          throttleItem
        }
      }
    }
  }
}
"#;

pub const CUSTOMIZATION_DATA_QUERY: &str = r#"
query CustomizationData(
  $productId: ID!
  $restaurantId: ID!
  $caloriesVersion: CaloriesVersion
) {
  product(id: $productId) {
    id
    slug
    name
    ingredients {
      id
      name
      kind
      calories(version: $caloriesVersion)
      totalCarbsG
      proteinG
      totalFatG
    }
    modifierGroups {
      id
      modifications {
        id
        outOfStock
        ingredient {
          id
          name
          kind
          calories(version: $caloriesVersion)
          totalCarbsG
          proteinG
          totalFatG
        }
      }
    }
  }
  restaurant(id: $restaurantId) {
    id
  }
}
"#;

pub const LOCATIONS_SEARCH_BY_STRING_QUERY: &str = r#"
query LocationsSearchBySearchString(
  $searchString: String!
  $showHidden: Boolean
) {
  searchLocationsByString(
    searchString: $searchString
    showHidden: $showHidden
  ) {
    score
    location {
      ...SearchByStringLocationDetails
    }
  }
}

fragment SearchByStringLocationDetails on StoreLocation {
  id
  name
  latitude
  longitude
  slug
  address
  city
  state
  zipCode
  isOutpost
  phone
  storeHours
  flexMessage
  enabled
  acceptingOrders
  notAcceptingOrdersReason
  hidden
  sanitaryGradeDisplayText
  sanitaryGradeUrl
}
"#;

pub const SIGN_IN_MUTATION: &str = r#"
mutation SignIn {
  signIn {
    __typename
    ... on SignInSuccess {
      customer {
        id
        __typename
      }
    }
    ... on CustomerNotFound {
      errorMessage
    }
    ... on ValidationError {
      message
      status
      fieldErrors {
        field
        message
      }
    }
    ... on SignInFailed {
      message
      status
    }
  }
}
"#;

pub const BAG_COUNT_QUERY: &str = r#"
query BagCount {
  cart {
    id
    __typename
    lineItems {
      id
      __typename
    }
  }
}
"#;

pub const BAG_CART_FRAGMENT: &str = r#"
fragment BagCartData on Order {
  id
  __typename
  orderType
  canTrackOrderStatus
  availableWantedTimes {
    time
    deliveryOffset
  }
  restaurant {
    id
    __typename
    slug
    name
    city
    state
    address
    zipCode
    isOutpost
    deliveryMinSubtotal
    isOutpost
    showOutpostPriceDifferenciationDisclosure
    showDeliveryPriceDifferenciationDisclosure
    showDeliveryFeeDisclosure
    deliveryFee
    availableDropOffLocations {
      id
      __typename
      name
    }
    asset {
      url
    }
  }
  deliveryOrderDetail {
    id
    __typename
    tip
    deliveryFee
    vendor
    orderId
    vendorRestaurantId
    estimatedDeliveryTime
    address {
      id
      __typename
      street
      secondaryStreet
      city
      state
      country
      zipCode
      deliveryPreference
      googlePlaceId
      latitude
      longitude
      name
      notes
    }
  }
  lineItems {
    id
    __typename
    slug
    quantity
    cost
    customName
    perItemCost
    favorited
    isCustom
    addedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    removedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    product {
      id
      __typename
      name
      slug
      calories
      isCustom
      isModifiable
      enableDressingDetails
      baseProduct {
        __typename
        id
        slug
        proteinG
        totalCarbsG
        totalFatG
        calories
      }
      asset {
        url
      }
      ingredients {
        id
        __typename
        name
        kind
        proteinG
        totalCarbsG
        totalFatG
      }
    }
    mixedDressingDetails {
      ingredientId
      weight
    }
    dressingMode
  }
  ledger {
    tax
    subtotal
    feesTotal
    discountsTotal
    creditsTotal
    tip
    discounts {
      id
      __typename
      name
      amount
      description
    }
    credits {
      id
      __typename
      name
      amount
      description
    }
    fees {
      id
      __typename
      name
      amount
      description
    }
  }
}
"#;

pub const BAG_CART_QUERY: &str = r#"
query BagCart {
  cart {
    ...BagCartData
  }
}

fragment BagCartData on Order {
  id
  __typename
  orderType
  canTrackOrderStatus
  availableWantedTimes {
    time
    deliveryOffset
  }
  restaurant {
    id
    __typename
    slug
    name
    city
    state
    address
    zipCode
    isOutpost
    deliveryMinSubtotal
    isOutpost
    showOutpostPriceDifferenciationDisclosure
    showDeliveryPriceDifferenciationDisclosure
    showDeliveryFeeDisclosure
    deliveryFee
    availableDropOffLocations {
      id
      __typename
      name
    }
    asset {
      url
    }
  }
  deliveryOrderDetail {
    id
    __typename
    tip
    deliveryFee
    vendor
    orderId
    vendorRestaurantId
    estimatedDeliveryTime
    address {
      id
      __typename
      street
      secondaryStreet
      city
      state
      country
      zipCode
      deliveryPreference
      googlePlaceId
      latitude
      longitude
      name
      notes
    }
  }
  lineItems {
    id
    __typename
    slug
    quantity
    cost
    customName
    perItemCost
    favorited
    isCustom
    addedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    removedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    product {
      id
      __typename
      name
      slug
      calories
      isCustom
      isModifiable
      enableDressingDetails
      baseProduct {
        __typename
        id
        slug
        proteinG
        totalCarbsG
        totalFatG
        calories
      }
      asset {
        url
      }
      ingredients {
        id
        __typename
        name
        kind
        proteinG
        totalCarbsG
        totalFatG
      }
    }
    mixedDressingDetails {
      ingredientId
      weight
    }
    dressingMode
  }
  ledger {
    tax
    subtotal
    feesTotal
    discountsTotal
    creditsTotal
    tip
    discounts {
      id
      __typename
      name
      amount
      description
    }
    credits {
      id
      __typename
      name
      amount
      description
    }
    fees {
      id
      __typename
      name
      amount
      description
    }
  }
}
"#;

pub const BAG_TIMES_POLLING_QUERY: &str = r#"
query BagTimesPolling {
  cart {
    id
    __typename
    availableWantedTimes {
      time
      deliveryOffset
    }
  }
}
"#;

pub const ADD_LINE_ITEM_MUTATION: &str = r#"
mutation AddLineItemToCart($input: AddLineItemToCartInput!) {
  addLineItemToCart(input: $input) {
    __typename
    ... on AddLineItemToCartSuccess {
      cart {
        ...AddLineItemCartData
      }
    }
    ... on RestaurantMaxQuantityExceeded {
      message
      status
    }
    ... on RestaurantMaxDeliveryQuantityExceeded {
      message
      status
    }
    ... on ProductOutOfStock {
      message
      status
    }
    ... on MaxModificationsExceeded {
      message
      status
    }
    ... on ValidationError {
      message
      status
      fieldErrors {
        field
        message
      }
    }
  }
}

fragment AddLineItemCartData on Order {
  id
  __typename
  orderType
  canTrackOrderStatus
  restaurant {
    id
    __typename
    slug
    name
    deliveryMinSubtotal
    deliveryFee
  }
  deliveryOrderDetail {
    tip
    deliveryFee
    vendor
    vendorRestaurantId
    address {
      id
    }
  }
  lineItems {
    id
    __typename
    slug
    quantity
    cost
    customName
    perItemCost
    favorited
    isCustom
    addedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
    }
    removedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
    }
    product {
      id
      __typename
      name
      slug
      calories
      isCustom
      isModifiable
      enableDressingDetails
      baseProduct {
        __typename
        id
        slug
        proteinG
        totalCarbsG
        totalFatG
        calories
      }
      asset {
        url
      }
      ingredients {
        id
        __typename
        name
        kind
        proteinG
        totalCarbsG
        totalFatG
      }
    }
  }
  ledger {
    tax
    subtotal
    feesTotal
    discountsTotal
    creditsTotal
    tip
  }
}
"#;

pub const UPDATE_LINE_ITEM_MUTATION: &str = r#"
mutation UpdateLineItem($input: EditLineItemInCartInput!) {
  editLineItemInCart(input: $input) {
    __typename
    ... on EditLineItemInCartSuccess {
      cart {
        ...BagCartData
      }
    }
    ... on RestaurantMaxQuantityExceeded {
      message
      status
    }
    ... on MaxModificationsExceeded {
      message
      status
    }
    ... on RestaurantMaxDeliveryQuantityExceeded {
      message
      status
    }
    ... on ValidationError {
      message
      status
      fieldErrors {
        field
        message
      }
    }
    ... on LocationClosed {
      message
      status
    }
  }
}

fragment BagCartData on Order {
  id
  __typename
  orderType
  canTrackOrderStatus
  availableWantedTimes {
    time
    deliveryOffset
  }
  restaurant {
    id
    __typename
    slug
    name
    city
    state
    address
    zipCode
    isOutpost
    deliveryMinSubtotal
    isOutpost
    showOutpostPriceDifferenciationDisclosure
    showDeliveryPriceDifferenciationDisclosure
    showDeliveryFeeDisclosure
    deliveryFee
    availableDropOffLocations {
      id
      __typename
      name
    }
    asset {
      url
    }
  }
  deliveryOrderDetail {
    id
    __typename
    tip
    deliveryFee
    vendor
    orderId
    vendorRestaurantId
    estimatedDeliveryTime
    address {
      id
      __typename
      street
      secondaryStreet
      city
      state
      country
      zipCode
      deliveryPreference
      googlePlaceId
      latitude
      longitude
      name
      notes
    }
  }
  lineItems {
    id
    __typename
    slug
    quantity
    cost
    customName
    perItemCost
    favorited
    isCustom
    addedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    removedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    product {
      id
      __typename
      name
      slug
      calories
      isCustom
      isModifiable
      enableDressingDetails
      baseProduct {
        __typename
        id
        slug
        proteinG
        totalCarbsG
        totalFatG
        calories
      }
      asset {
        url
      }
      ingredients {
        id
        __typename
        name
        kind
        proteinG
        totalCarbsG
        totalFatG
      }
    }
    mixedDressingDetails {
      ingredientId
      weight
    }
    dressingMode
  }
  ledger {
    tax
    subtotal
    feesTotal
    discountsTotal
    creditsTotal
    tip
    discounts {
      id
      __typename
      name
      amount
      description
    }
    credits {
      id
      __typename
      name
      amount
      description
    }
    fees {
      id
      __typename
      name
      amount
      description
    }
  }
}
"#;

pub const REMOVE_LINE_ITEM_MUTATION: &str = r#"
mutation RemoveLineItem($input: RemoveFromCartInput!) {
  removeFromCart(input: $input) {
    __typename
    ... on RemoveFromCartSuccess {
      cart {
        ...BagCartData
      }
    }
    ... on ValidationError {
      message
      status
      fieldErrors {
        field
        message
      }
    }
    ... on LocationClosed {
      message
      status
    }
  }
}

fragment BagCartData on Order {
  id
  __typename
  orderType
  canTrackOrderStatus
  availableWantedTimes {
    time
    deliveryOffset
  }
  restaurant {
    id
    __typename
    slug
    name
    city
    state
    address
    zipCode
    isOutpost
    deliveryMinSubtotal
    isOutpost
    showOutpostPriceDifferenciationDisclosure
    showDeliveryPriceDifferenciationDisclosure
    showDeliveryFeeDisclosure
    deliveryFee
    availableDropOffLocations {
      id
      __typename
      name
    }
    asset {
      url
    }
  }
  deliveryOrderDetail {
    id
    __typename
    tip
    deliveryFee
    vendor
    orderId
    vendorRestaurantId
    estimatedDeliveryTime
    address {
      id
      __typename
      street
      secondaryStreet
      city
      state
      country
      zipCode
      deliveryPreference
      googlePlaceId
      latitude
      longitude
      name
      notes
    }
  }
  lineItems {
    id
    __typename
    slug
    quantity
    cost
    customName
    perItemCost
    favorited
    isCustom
    addedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    removedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    product {
      id
      __typename
      name
      slug
      calories
      isCustom
      isModifiable
      enableDressingDetails
      baseProduct {
        __typename
        id
        slug
        proteinG
        totalCarbsG
        totalFatG
        calories
      }
      asset {
        url
      }
      ingredients {
        id
        __typename
        name
        kind
        proteinG
        totalCarbsG
        totalFatG
      }
    }
    mixedDressingDetails {
      ingredientId
      weight
    }
    dressingMode
  }
  ledger {
    tax
    subtotal
    feesTotal
    discountsTotal
    creditsTotal
    tip
    discounts {
      id
      __typename
      name
      amount
      description
    }
    credits {
      id
      __typename
      name
      amount
      description
    }
    fees {
      id
      __typename
      name
      amount
      description
    }
  }
}
"#;

pub const APPLY_PROMO_MUTATION: &str = r#"
mutation applyPromoCode($input: SubmitPromoOrGiftCardCodeInput!) {
  submitPromoOrGiftCardCode(input: $input) {
    __typename
    ... on InvalidPromoCode {
      message
      status
    }
    ... on ValidationError {
      message
      status
      fieldErrors {
        field
        message
      }
    }
  }
}
"#;

pub const BAG_REWARDS_QUERY: &str = r#"
query BagRewards {
  rewards {
    id
    __typename
    name
    expirationDate
    rewardType
    redeemable
    redeemableAt
  }
}
"#;

pub const APPLY_REWARD_MUTATION: &str = r#"
mutation applyBagReward($input: ApplyRewardInput!) {
  applyReward(input: $input) {
    __typename
    ... on ApplyRewardSuccess {
      order {
        ...BagCartData
      }
    }
    ... on RewardNotApplied {
      message
      status
      failureCode
      failureReasons
      failureMetadata {
        requiredChannel
        requiredDays
      }
    }
    ... on ValidationError {
      message
      status
      fieldErrors {
        field
        message
      }
    }
  }
}

fragment BagCartData on Order {
  id
  __typename
  orderType
  canTrackOrderStatus
  availableWantedTimes {
    time
    deliveryOffset
  }
  restaurant {
    id
    __typename
    slug
    name
    city
    state
    address
    zipCode
    isOutpost
    deliveryMinSubtotal
    isOutpost
    showOutpostPriceDifferenciationDisclosure
    showDeliveryPriceDifferenciationDisclosure
    showDeliveryFeeDisclosure
    deliveryFee
    availableDropOffLocations {
      id
      __typename
      name
    }
    asset {
      url
    }
  }
  deliveryOrderDetail {
    id
    __typename
    tip
    deliveryFee
    vendor
    orderId
    vendorRestaurantId
    estimatedDeliveryTime
    address {
      id
      __typename
      street
      secondaryStreet
      city
      state
      country
      zipCode
      deliveryPreference
      googlePlaceId
      latitude
      longitude
      name
      notes
    }
  }
  lineItems {
    id
    __typename
    slug
    quantity
    cost
    customName
    perItemCost
    favorited
    isCustom
    addedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    removedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    product {
      id
      __typename
      name
      slug
      calories
      isCustom
      isModifiable
      enableDressingDetails
      baseProduct {
        __typename
        id
        slug
        proteinG
        totalCarbsG
        totalFatG
        calories
      }
      asset {
        url
      }
      ingredients {
        id
        __typename
        name
        kind
        proteinG
        totalCarbsG
        totalFatG
      }
    }
    mixedDressingDetails {
      ingredientId
      weight
    }
    dressingMode
  }
  ledger {
    tax
    subtotal
    feesTotal
    discountsTotal
    creditsTotal
    tip
    discounts {
      id
      __typename
      name
      amount
      description
    }
    credits {
      id
      __typename
      name
      amount
      description
    }
    fees {
      id
      __typename
      name
      amount
      description
    }
  }
}
"#;

pub const REMOVE_REWARD_MUTATION: &str = r#"
mutation removeBagReward($input: RemoveRewardInput!) {
  removeReward(input: $input) {
    __typename
    ... on RemoveRewardSuccess {
      order {
        ...BagCartData
      }
    }
    ... on RewardNotRemoved {
      message
      status
    }
    ... on ValidationError {
      message
      status
      fieldErrors {
        field
        message
      }
    }
  }
}

fragment BagCartData on Order {
  id
  __typename
  orderType
  canTrackOrderStatus
  availableWantedTimes {
    time
    deliveryOffset
  }
  restaurant {
    id
    __typename
    slug
    name
    city
    state
    address
    zipCode
    isOutpost
    deliveryMinSubtotal
    isOutpost
    showOutpostPriceDifferenciationDisclosure
    showDeliveryPriceDifferenciationDisclosure
    showDeliveryFeeDisclosure
    deliveryFee
    availableDropOffLocations {
      id
      __typename
      name
    }
    asset {
      url
    }
  }
  deliveryOrderDetail {
    id
    __typename
    tip
    deliveryFee
    vendor
    orderId
    vendorRestaurantId
    estimatedDeliveryTime
    address {
      id
      __typename
      street
      secondaryStreet
      city
      state
      country
      zipCode
      deliveryPreference
      googlePlaceId
      latitude
      longitude
      name
      notes
    }
  }
  lineItems {
    id
    __typename
    slug
    quantity
    cost
    customName
    perItemCost
    favorited
    isCustom
    addedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    removedIngredients {
      id
      __typename
      name
      kind
      proteinG
      totalCarbsG
      totalFatG
      asset {
        url
      }
    }
    product {
      id
      __typename
      name
      slug
      calories
      isCustom
      isModifiable
      enableDressingDetails
      baseProduct {
        __typename
        id
        slug
        proteinG
        totalCarbsG
        totalFatG
        calories
      }
      asset {
        url
      }
      ingredients {
        id
        __typename
        name
        kind
        proteinG
        totalCarbsG
        totalFatG
      }
    }
    mixedDressingDetails {
      ingredientId
      weight
    }
    dressingMode
  }
  ledger {
    tax
    subtotal
    feesTotal
    discountsTotal
    creditsTotal
    tip
    discounts {
      id
      __typename
      name
      amount
      description
    }
    credits {
      id
      __typename
      name
      amount
      description
    }
    fees {
      id
      __typename
      name
      amount
      description
    }
  }
}
"#;

pub const GIFT_CARD_BALANCE_QUERY: &str = r#"
query BagGiftCardBalance {
  giftCardBalance {
    __typename
    ... on GiftCardBalance {
      customerId
      giftCardBalance
    }
    ... on UnableToGetGiftCardBalanceError {
      errorMessage
    }
  }
}
"#;

pub const REDEEM_GIFT_CARD_MUTATION: &str = r#"
mutation RedeemGiftCardInBag($code: String!, $regCode: String) {
  redeemGiftCard(code: $code, regCode: $regCode) {
    __typename
    ... on GiftCardBalance {
      customerId
      giftCardBalance
    }
    ... on InvalidGiftCardError {
      errorMessage
    }
    ... on GiftCardAssociatedWithAnotherAccountError {
      errorMessage
    }
    ... on UnableToRedeemGiftCardError {
      errorMessage
    }
    ... on NoBalanceGiftCardError {
      errorMessage
    }
  }
}
"#;
