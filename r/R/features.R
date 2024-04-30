# xy <- st_sfc(st_point(c(1, 2)))
# xyz <- st_sfc(st_point(c(1, 2, 3)))
# xym <- st_sfc(st_point(c(1, 2, 3), dim = "XYM"))

# sfc_point_features_2d(xy, list())
# sfc_point_features_3d(xyz, list())
# sfc_point_features_3d(xym, list())
# NOTE 4D features cannot be supported
as_esri_features_sfc <- function(x, crs = NULL, call = rlang::caller_env()) {
  # TODO handle CRS
  sr <- crs %||% list()

  # class check
  valid_sfg_classes <- c(
    "sfc_POINT",
    "sfc_MULTIPOINT",
    "sfc_LINESTRING",
    "sfc_MULTILINESTRING",
    "sfc_POLYGON",
    "sfc_MULTIPOLYGON"
  )

  # exit if an invalid geometry type is provided
  if (!rlang::inherits_any(x, valid_sfg_classes)) {
    cli::cli_abort(
      "{.arg x} must inherit one of the following classes {.cls {valid_sfg_classes}}",
      call = call
    )
  }

  # store the class name to be used in a switch statement
  sfc_class <- inherits_which(x, valid_sfg_classes)

  # dimension check
  z <- has_z(x)
  m <- has_m(x)

  # abort if 4D
  if (z && m) {
    cli::cli_abort(
      c(
        "{.cls XYZM} geometries detected. Only {.cls XY}, {.cls XYZ}, and {.cls XYM} geometries supported. ",
        ">" = "{.href [please make an issue on GitHub](https://github.com/r-arcgis/arcgisutils)}"
      ),
      call = call
    )
  }

  # determine if 3D
  three_dim <- z || m

  # switch based on dimensions
  if (three_dim) {
    switch(sfc_class,
      "sfc_POINT" = sfc_point_features_3d(x, sr),
      "sfc_MULTIPOINT" = sfc_multipoint_features_3d(x, sr),
      "sfc_LINESTRING" = sfc_linestring_features_3d(x, sr),
      "sfc_MULTILINESTRING" = sfc_multilinestring_features_3d(x, sr),
      "sfc_POLYGON" = sfc_polygon_features_3d(x, sr),
      "sfc_MULTIPOLYGON" = sfc_multipolygon_features_3d(x, sr),
    )
  } else {
    switch(sfc_class,
      "sfc_POINT" = sfc_point_features_2d(x, sr),
      "sfc_MULTIPOINT" = sfc_multipoint_features_2d(x, sr),
      "sfc_LINESTRING" = sfc_linestring_features_2d(x, sr),
      "sfc_MULTILINESTRING" = sfc_multilinestring_features_2d(x, sr),
      "sfc_POLYGON" = sfc_polygon_features_2d(x, sr),
      "sfc_MULTIPOLYGON" = sfc_multipolygon_features_2d(x, sr),
    )
  }
}
