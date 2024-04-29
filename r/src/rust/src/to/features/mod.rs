mod linestring;
mod multilinestring;
mod multipoint;
mod multipolygon;
mod point;
mod polygon;

use extendr_api::prelude::*;

extendr_module! {
    mod features;
    use linestring;
    use multilinestring;
    use multipoint;
    use multipolygon;
    use point;
    use polygon;
}
