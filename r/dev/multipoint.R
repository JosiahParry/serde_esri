# pak::pak("extendr/rextendr")
# pak::pak("devtools", dependencies = NA)
rextendr::document()
devtools::load_all()

library(sf)

x <- st_multipoint(matrix(rnorm(10), ncol = 2), dim = "XY")
as_multipoint(x) |>
  jsonify::pretty_json()

z <- st_multipoint(matrix(rnorm(12), ncol = 3), dim = "XYZ")
as_multipoint(z) |>
  jsonify::pretty_json()

m <- st_multipoint(matrix(rnorm(12), ncol = 3), dim = "XYM")
as_multipoint(m) |>
  jsonify::pretty_json()

zm <- st_multipoint(matrix(rnorm(12), ncol = 4), dim = "XYZM")
as_multipoint(zm) |>
  jsonify::pretty_json()
