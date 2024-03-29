# `serde_esri`

Esri JSON parsing library.

This crate provides representations of Esri JSON objects with [`serde::Deserialize`](https://docs.rs/serde/1.0.192/serde/de/trait.Deserialize.html) and [`serde::Serialize`](https://docs.rs/serde/1.0.192/serde/de/trait.Serialize.html) trait implementations.

`serde_esri` has two additional features `geo` and `geoarrow`. 

- `geo` implements `From` for the Esri JSON objects.
- `geoarrow` provides compatibility with arrow and geoarrow by implementing geoarrow geometry traits as well as providing a utility function `featureset_to_arrow()` which converts a `FeatureSet` to an arrow `RecordBatch`.

## Example usage: 

In this example, we query a feature service and convert the response to an Arrow `RecordBatch`. This requires the `geoarrow` feature to be enabled.

```rust
use serde_esri::features::FeatureSet;
use serde_esri::arrow_compat::featureset_to_arrow;

#[tokio::main]
async fn main() {
    
    // query url 
    let furl = "https://services.arcgis.com/P3ePLMYs2RVChkJx/ArcGIS/rest/services/USA_Counties_Generalized_Boundaries/FeatureServer/0/query?where=1=1&outFields=*&f=json&resultRecordCount=10";
    
    // make the request
    let resp = reqwest::get(furl)
        .await.unwrap()
        .text()
        .await.unwrap();
    
    // parse the response into a FeatureSet
    let fset: FeatureSet<2> = serde_json::from_str(&resp).unwrap();

    // convert the FeatureSet to an Arrow RecordBatch
    let rb = featureset_to_arrow(fset).unwrap();

    println!("{:#?}", rb.schema());
}
```

## Supported Esri JSON objects:

### Geometry 

[Esri Geometries Objects](https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm#CURVE) are encoded by the following structs: 

- `EsriPoint`
- `EsriMultiPoint<N>`
- `EsriPolyline<N>`
- `EsriPolygon<N>`

They are encapsulated by the `EsriGeometry` enum:

```rust
enum EsriGeometry<const N: usize> {
    Point(EsriPoint),
    MultiPoint(EsriMultiPoint<N>),
    Polygon(EsriPolygon<N>),
    Polyline(EsriPolyline<N>),
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

## Example usage: 

This example reads a single feature from a feature service and returns a `FeatureSet` struct. 

```rust
use serde_esri::features::FeatureSet;
use reqwest::Error;
use std::io::Read;

fn main() -> Result<(), Error> {
    let flayer_url = "https://services.arcgis.com/P3ePLMYs2RVChkJx/ArcGIS/rest/services/USA_Counties_Generalized_Boundaries/FeatureServer/0/query?where=1%3D1&outFields=*&returnGeometry=true&resultRecordCount=1&f=json";
    let mut res = reqwest::blocking::get(flayer_url)?;
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let fset: FeatureSet<2> = serde_json::from_str(&body).unwrap(); 
    println!("{:#?}", fset);
    Ok(())
}
```