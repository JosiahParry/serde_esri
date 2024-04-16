use crate::places::Icon;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Request parameters used to query the [`/places/near-point`](https://developers.arcgis.com/rest/places/near-point-get/) endpoint
#[derive(Debug, Clone, Deserialize, Serialize, Default, Builder)]
pub struct NearPointQueryParams {
    /// The x, or longitude, of this location in WGS84 decimal degrees.
    pub x: f64,
    /// The y, or latitude, of this location in WGS84 decimal degrees.
    pub y: f64,
    ///The radius in meters to search for places - measured from the supplied x and y coordinate.
    ///
    /// A radius of 100 meters will result in a circular search area that is 200 meters in diameter. If omitted, the default search radius will be 1,000 meters.
    ///
    /// Default value : 1000.0
    #[builder(setter(into, strip_option), default)]
    pub radius: Option<f64>,
    /// Filters places to those that match the category Ids.

    /// Places will be returned if they match any of the category Ids. If this property is not set, places will be returned regardless of their category.

    /// You can obtain information on category Ids from the places/categories endpoint. For example, to filter for places where the category is "Bicycle Store", include the categoryId 17117 in this property.
    /// You can search up to a maximum of 10 category Ids.
    #[builder(setter(into, strip_option), default)]
    pub category_id: Option<Vec<String>>,
    /// Free search text for places against names, categories etc. Maximum of 255 characters.
    #[builder(setter(into, strip_option), default)]
    pub search_text: Option<String>,
    /// The icon format to return. The default is svg. Use the [`Icon`] enum.
    #[builder(setter(into, strip_option), default)]
    pub icon: Option<Icon>,
}

/// Prepared version of NearPointQueryParams which concatenates the category_ids
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NearPointPreparedParams {
    x: f64,
    y: f64,
    radius: Option<f64>,
    category_id: Option<String>,
    search_text: Option<String>,
    icon: Option<Icon>,
}

#[cfg(feature = "places-client")]
impl NearPointQueryParams {
    pub(crate) fn prepare(self) -> NearPointPreparedParams {
        NearPointPreparedParams {
            x: self.x,
            y: self.y,
            radius: self.radius,
            category_id: self.category_id.map_or_else(|| None, |f| Some(f.join(","))),
            search_text: self.search_text,
            icon: self.icon,
        }
    }
}

/// Request parameters used to query the [`/places/within-extent`](https://developers.arcgis.com/rest/places/within-extent-get/) endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Builder, Default)]
#[serde(rename_all = "camelCase")]
pub struct WithinExtentQueryParams {
    /// The minimum x coordinate, or longitude, of the search extent.
    /// This is the furthest west that will be searched.
    pub xmin: f64,
    ///The minimum y coordinate, or latitude, of the search extent.
    ///  This is the furthest south that will be searched.
    pub ymin: f64,

    /// The maximum x coordinate, or longitude, of the search extent.
    /// This is the furthest east that will be searched.
    pub xmax: f64,

    /// The maximum y coordinate, or latitude, of the search extent.
    /// This is the furthest north that will be searched.
    pub ymax: f64,
    /// Filters places to those that match the category Ids.

    /// Places will be returned if they match any of the category Ids. If this property is not set, places will be returned regardless of their category.

    /// You can obtain information on category Ids from the places/categories endpoint. For example, to filter for places where the category is "Bicycle Store", include the categoryId 17117 in this property.
    /// You can search up to a maximum of 10 category Ids.
    #[builder(setter(into, strip_option), default)]
    pub category_ids: Option<Vec<String>>,
    /// Free search text for places against names, categories etc. Maximum of 255 characters.
    #[builder(setter(into, strip_option), default)]
    pub search_text: Option<String>,
    /// The icon format to return. The default is svg. Use the [`Icon`] enum.
    #[builder(setter(into, strip_option), default)]
    pub icon: Option<Icon>,
}

/// Prepared version of NearPointQueryParams which concatenates the category_ids
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WithinExtentPreparedParams {
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
    category_ids: Option<String>,
    search_text: Option<String>,
    icon: Option<Icon>,
}

#[cfg(feature = "places-client")]
impl WithinExtentQueryParams {
    pub(crate) fn prepare(self) -> WithinExtentPreparedParams {
        WithinExtentPreparedParams {
            xmin: self.xmin,
            ymin: self.ymin,
            xmax: self.xmax,
            ymax: self.ymax,
            category_ids: self
                .category_ids
                .map_or_else(|| None, |f| Some(f.join(","))),
            search_text: self.search_text,
            icon: self.icon,
        }
    }
}

/// Request parameters used to query the [`/places/{placeId}`](https://developers.arcgis.com/rest/places/place-id-get/) endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
#[serde(rename_all = "camelCase")]
pub struct PlaceQueryParams {
    /// The ID of the place for which you want to fetch additional details.
    pub place_id: String,
    /// The array of fields that define the attributes to return for a place.
    ///
    /// Use this parameter to define the attributes you would like returned, for example requestedFields=name,address:streetAddress. However, you can also set this value to requestedFields=all to return all of the attributes available for a place.
    ///
    /// The placeId attribute is always returned in addition to the other attributes you requested. If a valid attribute value is not available, null, or an empty collection, is returned and you are not charged for it.
    /// Available values : all, additionalLocations, additionalLocations:dropOff, additionalLocations:frontDoor, additionalLocations:road, additionalLocations:roof, address, address:adminRegion, address:censusBlockId, address:country, address:designatedMarketArea, address:extended, address:locality, address:neighborhood, address:poBox, address:postcode, address:postTown, address:region, address:streetAddress, categories, contactInfo, contactInfo:email, contactInfo:fax, contactInfo:telephone, contactInfo:website, chains, description, hours, hours:opening, hours:openingText, hours:popular, location, name, rating, rating:price, rating:user, socialMedia, socialMedia:facebookId, socialMedia:instagram, socialMedia:twitter
    pub requested_fields: Vec<String>,
}

/// Request parameters used to query the [`/categories`](https://developers.arcgis.com/rest/places/categories-get/) endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
pub struct CategoriesQueryParams {
    /// A text filter that will be used for searching categories.
    ///
    /// The text must be at least three characters and will be applied as a partial match. For example, using the filter "off" would return categories using the word "Office" as well as those using the word "Coffee".
    #[builder(setter(into, strip_option), default)]
    pub filter: Option<String>,
    /// Determines whether icons are returned and the type of icon to use with a place or category.
    #[builder(setter(into, strip_option), default)]
    pub icon: Option<Icon>,
    /// Optional case-sensitive parameter to specify the preferred language to use for category names.
    ///
    /// This query parameter uses language codes to specify the preferred language. If not set, or if no translation is available, the default behavior is to return category names in English.
    ///
    /// The language codes use the CLDR (Common Locale Data Repository) format string that uses a two letter language code (e.g. "fr" for French) optionally followed by a two letter country code (e.g. "fr-CA" for French in Canada).
    ///
    /// If an unsupported language code is used, then the service will attempt to fall-back to the closest available language. This is done by stripping regional and extension subtags to find a known language code. For example, French Canadian (fr-CA) is unsupported so this falls back to French fr.
    ///
    /// Should the fallback fail, then the service will return category names in the default language en for English
    #[builder(setter(into, strip_option), default)]
    pub language: Option<String>,
}

/// Request parameters used to query the [`/categories/{categoryId}`](https://developers.arcgis.com/rest/places/categories-category-id-get/) endpoint
#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
pub struct CategoryQueryParams {
    pub category_id: String,
    #[builder(setter(into, strip_option), default)]
    pub icon: Option<Icon>,
    #[builder(setter(into, strip_option), default)]
    pub language: Option<String>,
}
