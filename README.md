# `serde_esri`

Esri JSON parsing library.

This crate provides representations of Esri JSON objects with [`serde::Deserialize`](https://docs.rs/serde/1.0.192/serde/de/trait.Deserialize.html) and [`serde::Serialize`](https://docs.rs/serde/1.0.192/serde/de/trait.Serialize.html) trait implementations.

`serde_esri` has additional features:

- `geo` implements `From` for the Esri JSON objects.
- `geoarrow` provides compatibility with arrow and geoarrow by implementing geoarrow geometry traits as well as providing a utility function `featureset_to_geoarrow()` which converts a `FeatureSet` to an arrow `GeoTable`.
- `places-client` provides an API client for the Places Service REST API. 


## Example usage: 

This example reads a few features from a feature service and returns a `FeatureSet` struct. It illustrates the use of the geo and geoarrow features. 

```toml
[dependencies]
geo = "0.28.0"
geoarrow = "0.2.0"
reqwest = { version = "0.12.3", features = ["blocking"] }
serde_esri = { version = "0.2.0", features = ["geo", "geoarrow"] }
serde_json = "1.0.115"
```

```rust
use geo::{GeodesicArea, Polygon};
use serde_esri::arrow_compat::featureset_to_geoarrow;
use serde_esri::features::FeatureSet;
use std::io::Read;

fn main() {
    let flayer_url = "https://services.arcgis.com/P3ePLMYs2RVChkJx/ArcGIS/rest/services/USA_Counties_Generalized_Boundaries/FeatureServer/0/query?where=1%3D1&outFields=*&returnGeometry=true&resultRecordCount=5&f=json";
    let mut res = reqwest::blocking::get(flayer_url).unwrap();
    let mut body = String::new();

    // Read the request into a String
    res.read_to_string(&mut body).unwrap();

    // Parse into a 2D FeatureSet
    let fset: FeatureSet<2> = serde_json::from_str(&body).unwrap();

    // Utilize the `geo` feature by converting to Polygon
    // and calculate geodesic area
    // iterate through the features
    let area = fset
        .features
        .clone()
        .into_iter()
        .map(|feat| {
            // convert to a geo_types::Polygon
            let poly = Polygon::from(feat.geometry.unwrap().as_polygon().unwrap());
            // calculate geodesic area
            poly.geodesic_area_unsigned()
        })
        .collect::<Vec<_>>();

    // print areas
    println!("{:?}", area);

    // convert to a geoarrow GeoTable
    println!("{:?}", featureset_to_geoarrow(fset).unwrap());
}
```

## Supported Esri JSON objects:

### Geometry 

[Esri Geometries Objects](https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm) are encoded by the following structs: 

- `EsriPoint`
- `EsriMultiPoint<N>`
- `EsriPolyline<N>`
- `EsriPolygon<N>`
- `EsriEnvelope`

They are encapsulated by the `EsriGeometry` enum:

```rust
enum EsriGeometry<const N: usize> {
    Point(EsriPoint),
    MultiPoint(EsriMultiPoint<N>),
    Polygon(EsriPolygon<N>),
    Polyline(EsriPolyline<N>),
    Envelope(EsriEnvelope),
}
```
The parameter `N` is used to specify the dimension of the geometries. Use `<2>` for 2 dimensional data, `<3>` for Z values and `<4>` when `M` and `Z` are present. 

### FeatureSets 

An Esri JSON [`FeatureSet`](https://developers.arcgis.com/documentation/common-data-types/featureset-object.htm) is what is most commonly returned from a [Feature Service](https://developers.arcgis.com/rest/services-reference/enterprise/feature-service.htm). It is comprised of a number of optional fields and most importantly, a vector of `Feature`s.

Features are a struct with a `geometry` and an `attributes` field. The geometry must be one of the possible geometry types and attributes can be an key-value pair. 

```rust
struct Feature<const N: usize> {
    geometry: Option<EsriGeometry<N>>,
    attributes: Option<Map<String, Value>>,
}
```

FeatureSets are defined as 

```rust
pub struct FeatureSet<const N: usize> {
    // ... other optional fields 
    features: Vec<Feature<N>>,
    geometryType: Option<String>,
    spatialReference: SpatialReference,
}
```

### Spatial References

[Esri Spatial Reference Objects](https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm#GUID-DFF0E738-5A42-40BC-A811-ACCB5814BABC) are represented by the `SpatialReference` struct. Note that while all fields are optional, one should always be provided. 

```rust
struct SpatialReference {
    wkid: Option<u32>,
    latest_wkid: Option<u32>,
    vcs_wkid: Option<u32>,
    latest_vcs_wkid: Option<u32>,
    wkt: Option<String>,
}
```

## Places Service API Client

Activate the PlaceAPI client in your Cargo.toml


```toml
[dependencies]
serde_esri = { version = "0.3.0", features = ["places-client"] }
```

```rust
fn main() {

    let client = PlacesClient::new(
        PLACES_API_URL,
        "your-developer-credential",
    );

    // Use the query within extent query builder to create query parameters
    let params = WithinExtentQueryParamsBuilder::default()
        .xmin(139.74)
        .ymin(35.65)
        .xmax(139.75)
        .ymax(35.66)
        .build()
        .unwrap();

    // Call the within_extent method with the query parameters
    let res = client.within_extent(params).unwrap();

    // use the automatic pagination for the iterator method
    res.into_iter()
        .for_each(|r| println!("{:?}", r.unwrap().name));
}

