mod linestring;
mod multipoint;
mod point;

use extendr_api::prelude::*;

extendr_module! {
    mod features;
    use linestring;
    use multipoint;
    use point;
}
