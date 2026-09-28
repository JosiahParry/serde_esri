// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::frf;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: frf = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

/// Determines whether icons are returned and the type of icon to use with a place or category.
/// The SVG and CIM symbols default to 15 x 15 pixels but can be scaled smoothly for display in larger UI elements or to emphasize these features on a map. The PNG icons are provided as 48 x 48 pixels but for map display the recommended size is 16 x 16 pixels.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Icon {
    Svg,
    Cim,
    Png,
}

/// A set of additional locations for the place.
///
/// This list provides alternative locations for accessing a place such as
/// `frontDoor` or `road`.
///
///
/// A set of additional locations that represent the place, for example the location of the
/// front door, or of a drop off point.
///
/// This object and child properties are part of the "Location"
/// attribute group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdditionalLocations {
    /// A location for drop-off/pick-up for a place.
    pub drop_off: Option<NullablePoint>,

    /// A location for the front door for a place.
    pub front_door: Option<NullablePoint>,

    /// A road-side location for a place.
    pub road: Option<NullablePoint>,

    /// A location in the roof centroid for a place.
    pub roof: Option<NullablePoint>,
}

/// A location for drop-off/pick-up for a place.
///
///
/// A point defined in WGS84 decimal degrees.
///
///
/// A location for the front door for a place.
///
///
/// A road-side location for a place.
///
///
/// A location in the roof centroid for a place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullablePoint {
    /// The x, or longitude, of this location in WGS84 decimal degrees.
    pub x: f64,

    /// The y, or latitude, of this location in WGS84 decimal degrees.
    pub y: f64,
}

/// The address of a place, or point of interest (POI).
///
///
/// The address of the place or point of interest (POI).
///
/// This object and child properties are part of the "Address" attribute
/// group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    /// Additional sub-division.
    ///
    /// Usually, but not always, a country sub-division (e.g., Scotland).
    pub admin_region: Option<String>,

    /// The census block Id of the place (US only).
    pub census_block_id: Option<String>,

    /// Two letter ISO country code
    pub country: Option<String>,

    /// As defined by Nielsen, signifies a region where the population can receive similar TV and
    /// radio offerings (US only).
    pub designated_market_area: Option<String>,

    /// Additional address information, including suite or apartment numbers.
    pub extended: Option<String>,

    /// The city, town, or equivalent.
    pub locality: Option<String>,

    /// The neighborhoods of the place.
    pub neighborhood: Option<Vec<String>>,

    /// Post-office box.
    pub po_box: Option<String>,

    /// Postal code or equivalent (zip code in the US).
    ///
    /// Format will be localized based on country.
    pub postcode: Option<String>,

    /// Town/place employed in postal addressing.
    pub post_town: Option<String>,

    /// The state, province, territory or equivalent.
    pub region: Option<String>,

    /// The street address for a place, for example the street name and number.
    pub street_address: Option<String>,
}

/// Represents the category of a place.
///
/// You can get more information on categories from the `places/categories`
/// endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    /// The category Id uniquely identifies this category or type of place.
    ///
    /// The name of the category can be looked up using the
    /// `places/categories` endpoint. For example, 17119 is the id for a
    /// "Bicycle Store".
    pub category_id: String,

    /// The label that describes the category.
    pub label: String,
}

/// Provides details about a category or type of place.
///
/// Categories are hierarchical so that you can filter places based on
/// specific categories such as "Provençal Restaurant", or with more generic
/// types such as "Restaurant". A category such as "Provençal Restaurant"
/// includes the details of its more generic parent, such as "French
/// Restaurant".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryDetails {
    /// A unique identifying string for a category.
    ///
    /// This matches the `categoryId` of a place's `category` property.
    pub category_id: String,

    /// The full list of labels that describe the category, including its more generic parent
    /// categories.
    pub full_label: Vec<String>,

    /// Details of an icon, suitable for depicting this place.
    ///
    /// To fetch icon details use the `icon` query parameter.
    pub icon: Option<IconDetails>,

    /// The list of parent category Ids for this category.
    pub parents: Option<Vec<String>>,
}

/// Details of an icon, suitable for depicting this place.
///
/// To fetch icon details use the `icon` query parameter.
///
///
/// Information about an icon for depicting a place or category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconDetails {
    /// Url for an icon for this place or category in either `svg`, `cim` or `png` format.
    pub url: Option<String>,
}

/// Information about a chain that a place belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    /// The name of the chain.
    pub name: Option<String>,
}

/// The contact information for a place.
///
///
/// Contact information for the place, such as the telephone number or email address.
///
/// This object and child properties are part of the "Details" attribute
/// group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// Email address.
    pub email: Option<String>,

    /// Fax number.
    pub fax: Option<String>,

    /// The telephone number of the place.
    pub telephone: Option<String>,

    /// The website address of the place.
    pub website: Option<String>,
}

/// Error JSON response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    /// Error information
    pub error: ErrorClass,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorClass {
    /// A code identifying the type of error.
    ///
    /// This can be an HTTP status code, `498` (invalid or expired
    /// token), or `499` (missing token).
    pub code: usize,

    /// List of details about the error.
    pub details: Option<Vec<String>>,

    /// A message describing the error.
    pub message: String,

    /// A link to general information about the service, such as the owning system and token
    /// service URL.
    ///
    /// This property is only
    /// present for errors relating to tokens.
    pub rest_info_url: Option<String>,
}

/// Lists the opening hours of this place or POI along with the popular or busy hours.
///
/// A string is also provided that can be used for display.
///
///
/// The operating hours for the place, including hours of opening and popular times.
///
/// This object and child properties are part of the "Details" attribute
/// group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hours {
    /// The opening hours for a place.
    ///
    /// Opening hours are shown by day of the week. Each day can have
    /// several pairs of from and to times. For example, if a coffee shop is
    /// open from 9:00 until 12:00 and then again from 13:00 until 17:00, it
    /// would contain two pairs of from/to times: 09:00 paired with 12:00
    /// and 13:00 with 17:00. Opening hours are shown in 24-hour time in the
    /// local timezone of the place or POI.
    pub opening: Option<HoursByDay>,

    /// The opening hours for this place, formatted for display.
    pub opening_text: Option<String>,

    /// The popular or busy hours for a place.
    ///
    /// Popular hours are shown by day of the week. Each day can have
    /// several pairs of from and to times. For example, if a coffee shop is
    /// popular from 9:00 until 10:00 and then again from 14:00 until 15:00,
    /// it would contain two pairs of from/to times: 09:00 paired with 10:00
    /// and 14:00 with 15:00. Popular hours are shown in 24-hour time in the
    /// local timezone of the place or POI.
    pub popular: Option<HoursByDay>,
}

/// The opening hours for a place.
///
/// Opening hours are shown by day of the week. Each day can have
/// several pairs of from and to times. For example, if a coffee shop is
/// open from 9:00 until 12:00 and then again from 13:00 until 17:00, it
/// would contain two pairs of from/to times: 09:00 paired with 12:00
/// and 13:00 with 17:00. Opening hours are shown in 24-hour time in the
/// local timezone of the place or POI.
///
///
/// The opening or popular hours for a place.
///
/// Opening hours are shown by day of the week. Each day can have several
/// pairs of from and to times. For example, if a coffee shop is open from
/// 9:00 until 12:00 and then again from 13:00 until 17:00, it would contain
/// two pairs of opening/closing times: 9:00 paired with 12:00 and 13:00
/// with 17:00. Hours are shown in 24-hour time in the local timezone of the
/// place or POI.
///
///
/// The popular or busy hours for a place.
///
/// Popular hours are shown by day of the week. Each day can have
/// several pairs of from and to times. For example, if a coffee shop is
/// popular from 9:00 until 10:00 and then again from 14:00 until 15:00,
/// it would contain two pairs of from/to times: 09:00 paired with 10:00
/// and 14:00 with 15:00. Popular hours are shown in 24-hour time in the
/// local timezone of the place or POI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoursByDay {
    pub friday: Option<Vec<TimeRange>>,

    pub monday: Option<Vec<TimeRange>>,

    pub saturday: Option<Vec<TimeRange>>,

    pub sunday: Option<Vec<TimeRange>>,

    pub thursday: Option<Vec<TimeRange>>,

    pub tuesday: Option<Vec<TimeRange>>,

    pub wednesday: Option<Vec<TimeRange>>,
}

/// A pair of times defining the start and end of a time period.
///
/// For example, this could define opening hours or popular hours. Hours are
/// shown in 24-hour time in the local timezone of the place or POI.
///
/// Where a time range is 24-hours (for example a venue that is open 24-hours),
/// the `from` property will be 00:00 and the `to` property will be 23:59.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// The start of a time range in the format "HH:MM".
    pub from: String,

    /// The end of a time range in the format "HH:MM".
    pub to: String,
}

/// A result of searching for places using either `places/near-point` or `places/within-extent`.
///
/// The result object includes a single place that satisfied the search. For `places/near-point`,
/// the distance from the search point, in meters, is included.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceResult {
    /// A list of category objects for a place.
    ///
    /// Categories are uniquely identified by a `categoryId`. For example,
    /// `17119` identifies a "Bicycle Store" and `10051` identifies a
    /// "Stadium". Note that a single place can belong to multiple
    /// categories (for example, a petrol station could also have a
    /// super-market).
    pub categories: Vec<Category>,

    /// The distance, in meters, from the place to the search point of a 'places/near-point`
    /// query.
    pub distance: Option<f64>,

    /// Details of an icon, suitable for depicting this place.
    ///
    /// To fetch icon details use the `icon` query parameter.
    pub icon: Option<IconDetails>,

    /// The location of this place as a WGS84 point.
    pub location: Point,

    /// The name of the place, or point of interest.
    /// You can search for places by name using the `searchText` property in
    /// a `places/near-point` or `places/within-extent` request.
    pub name: String,

    /// The unique Id of this place.
    ///
    /// This place Id can be passed to the `places/{placeId}` endpoint to
    /// retrieve additional details.
    pub place_id: String,
}

/// The location of this place as a WGS84 point.
///
///
/// A point defined in WGS84 decimal degrees.
///
///
/// The location of this place as a WGS84 point.
///
/// This property is part of the "Location" attribute group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    /// The x, or longitude, of this location in WGS84 decimal degrees.
    pub x: f64,

    /// The y, or latitude, of this location in WGS84 decimal degrees.
    pub y: f64,
}

/// Provides pagination links for accessing more results for the current request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    /// A url for fetching the next page of results.
    ///
    /// Use this property to request the next page of results if available. If
    /// this property is omitted then there are no more pages of results
    /// available. You must also supply authentication details, such as a
    /// `token`, to make a next page request.
    pub next_url: Option<String>,

    /// A url for fetching the previous page of results.
    ///
    /// Use this property to request the previous page of results if
    /// available. If this property is omitted then there are no previous
    /// pages of results. You must also supply authentication details, such
    /// as a `token`, to make a previous page request.
    pub previous_url: Option<String>,
}

/// The additional details for a `Place`, including address, contact details, opening hours,
/// and rating.
///
/// You can request additional details for a place by using the `placeId` in
/// a `places/{placeId}` request. Use the `requestedFields` query parameter
/// to choose the fields or attributes that are included in the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceDetails {
    /// A set of additional locations that represent the place, for example the location of the
    /// front door, or of a drop off point.
    ///
    /// This object and child properties are part of the "Location"
    /// attribute group.
    pub additional_locations: Option<AdditionalLocations>,

    /// The address of the place or point of interest (POI).
    ///
    /// This object and child properties are part of the "Address" attribute
    /// group.
    pub address: Option<Address>,

    /// A list of category objects for a place.
    ///
    /// Categories are uniquely identified by a `categoryId`. For example,
    /// `17119` identifies a "Bicycle Store" and `10051` identifies a
    /// "Stadium". Note that a single place can belong to multiple
    /// categories (for example, a petrol station could also have a
    /// super-market).
    ///
    /// This property is part of the "Place" attribute group.
    pub categories: Option<Vec<Category>>,

    /// Information about all the chains the place belongs to.
    ///
    /// This object and child properties are part of the "Details" attribute
    /// group.
    pub chains: Option<Vec<ChainInfo>>,

    /// Contact information for the place, such as the telephone number or email address.
    ///
    /// This object and child properties are part of the "Details" attribute
    /// group.
    pub contact_info: Option<ContactInfo>,

    /// A text description of the place.
    ///
    /// This property is part of the "Details" attribute group.
    pub description: Option<String>,

    /// The operating hours for the place, including hours of opening and popular times.
    ///
    /// This object and child properties are part of the "Details" attribute
    /// group.
    pub hours: Option<Hours>,

    /// Details of an icon, suitable for depicting this place.
    ///
    /// To fetch icon details use the `icon` query parameter.
    pub icon: Option<IconDetails>,

    /// The location of this place as a WGS84 point.
    ///
    /// This property is part of the "Location" attribute group.
    pub location: Option<Point>,

    /// The name of the place, or point of interest.
    ///
    /// This property is part of the "Place" attribute group.
    pub name: Option<String>,

    /// The unique Id of this place.
    pub place_id: String,

    /// Rating information supplied by users of the place.
    ///
    /// This object and child properties are part of the "Details" attribute
    /// group.
    pub rating: Option<Rating>,

    /// Social media information associated with the place.
    ///
    /// This object and child properties are part of the "Details" attribute
    /// group.
    pub social_media: Option<SocialMedia>,
}

/// Rating information supplied by users of the place.
///
/// This object and child properties are part of the "Details" attribute
/// group.
///
///
/// Rating information about the price and user rating of the place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    /// An indication of the overall price of a place based on user reviews.
    pub price: Option<Price>,

    /// A rating for the place based on user-reviews from 0 to 5, where 5 is the best rating.
    pub user: Option<f64>,
}

/// An indication of the overall price of a place based on user reviews.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Price {
    Cheap,

    Expensive,

    Moderate,

    #[serde(rename = "veryExpensive")]
    VeryExpensive,
}

/// Social media information associated with the place.
///
/// This object and child properties are part of the "Details" attribute
/// group.
///
///
/// The social media details for a place.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialMedia {
    /// The facebook Id of the place.
    pub facebook_id: Option<String>,

    /// The instagram ID of the place.
    pub instagram: Option<String>,

    /// The twitter handle of the place.
    pub twitter: Option<String>,
}

/// A result of searching for places using a `places/within-extent` request.
///
/// The result object includes a single place that satisfied the search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithinExtentResult {
    /// A list of category objects for a place.
    ///
    /// Categories are uniquely identified by a `categoryId`. For example,
    /// `17119` identifies a "Bicycle Store" and `10051` identifies a
    /// "Stadium". Note that a single place can belong to multiple
    /// categories (for example, a petrol station could also have a
    /// super-market).
    pub categories: Vec<Category>,

    /// Details of an icon, suitable for depicting this place.
    ///
    /// To fetch icon details use the `icon` query parameter.
    pub icon: Option<IconDetails>,

    /// The location of this place as a WGS84 point.
    pub location: Point,

    /// The name of the place, or point of interest.
    /// You can search for places by name using the `searchText` property in
    /// a `places/near-point` or `places/within-extent` request.
    pub name: String,

    /// The unique Id of this place.
    ///
    /// This place Id can be passed to the `places/{placeId}` endpoint to
    /// retrieve additional details.
    pub place_id: String,
}
