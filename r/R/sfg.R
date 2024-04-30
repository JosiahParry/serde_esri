# library(arcgisutils)

# POINT
# create sfg points
xy <- st_point(c(1, 2))
xyz <- st_point(c(1, 2, 3))
xym <- st_point(c(1, 2, 3), dim = "XYM")
xyzm <- st_point(c(1, 2, 3, 4))

as_esri_geometry(xy)
as_esri_geometry(xyz)
as_esri_geometry(xym)
as_esri_geometry(xyzym)



as_esri_geometry <- function(x, crs = NULL, call = rlang::caller_env()) {
  # TODO handle CRS
  sr <- crs %||% list()

  valid_sfg_classes <- c(
    "POINT",
    "MULTIPOINT",
    "LINESTRING",
    "MULTILINESTRING",
    "POLYGON",
    "MULTIPOLYGON"
  )

  if (!rlang::inherits_any(x, valid_sfg_classes)) {
    cli::cli_abort(
      "{.arg x} must inherit one of the following classes {.cls {valid_sfg_classes}}",
      call = call
    )
  }

  sfg_class <- inherits_which(x, valid_sfg_classes)
  switch(sfg_class,
    "POINT" = sfg_point_as_point(x, sr),
    "MULTIPOINT" = sfg_multipoint_as_multipoint(x, sr),
    "LINESTRING" = sfg_linestring_as_polyline(x, sr),
    "MULTILINESTRING" = sfg_multilinestring_as_polyline(x, sr),
    "POLYGON" = sfg_polygon_as_polygon(x, sr),
    "MULTIPOLYGON" = sfg_multipolygon_as_polygon(x, sr),
    cli::cli_abort()
  )
}

# Helper to determine which classes are inherited
inherits_which <- function(x, class) {
  inherited <- vapply(class, inherits, logical(1), x = x)
  class[inherited]
}
