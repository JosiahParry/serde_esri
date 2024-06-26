#' Create Esri JSON Geometry Objects
#'
#' @param x an object of class `sfg`. Must be one of  `"POINT"`, `"MULTIPOINT"`, `"LINESTRING"`, `"MULTILINESTRING"`, `"POLYGON"`, or `"MULTIPOLYGON"`.
#' @param crs an object of class `crs`.
#' @param call an environment or call object as such as created with `rlang::caller_env()`.
#' @examples
#' # POINT
#' # create sfg points
#' xy <- st_point(c(1, 2))
#' xyz <- st_point(c(1, 2, 3))
#' xym <- st_point(c(1, 2, 3), dim = "XYM")
#' xyzm <- st_point(c(1, 2, 3, 4))
#'
#' as_esri_geometry(xy)
#' as_esri_geometry(xyz)
#' as_esri_geometry(xym)
#' as_esri_geometry(xyzm)
#'
#' # MULTIPOINT
#' # vector to create matrix points
#' set.seed(0)
#' x <- rnorm(12)
#'
#' xy <- st_multipoint(matrix(x, ncol = 2))
#' xyz <- st_multipoint(matrix(x, ncol = 3))
#' xym <- st_multipoint(matrix(x, ncol = 3), dim = "XYM")
#' xyzm <- st_multipoint(matrix(x, ncol = 4), dim = "XYM")
#'
#' as_esri_geometry(xy)
#' as_esri_geometry(xyz)
#' as_esri_geometry(xym)
#' as_esri_geometry(xyzm)
#'
#' # LINESTRING
#' xy <- st_linestring(matrix(x, ncol = 2))
#' xyz <- st_linestring(matrix(x, ncol = 3))
#' xym <- st_linestring(matrix(x, ncol = 3), dim = "XYM")
#' xyzm <- st_linestring(matrix(x, ncol = 4), dim = "XYM")
#'
#' as_esri_geometry(xy)
#' as_esri_geometry(xyz)
#' as_esri_geometry(xym)
#' as_esri_geometry(xyzm)
#'
#' # MULTILINESTRING
#' as_esri_geometry(st_multilinestring(list(xy, xy)))
#' as_esri_geometry(st_multilinestring(list(xyz, xyz)))
#' as_esri_geometry(st_multilinestring(list(xym, xym)))
#' as_esri_geometry(st_multilinestring(list(xyzm, xyzm)))
#'
#' # POLYGON
#' coords <- rbind(
#'   c(0, 0, 0, 1),
#'   c(0, 1, 0, 1),
#'   c(1, 1, 1, 1),
#'   c(1, 0, 1, 1),
#'   c(0, 0, 0, 1)
#' )
#'
#' xy <- st_polygon(list(coords[, 1:2]))
#' xyz <- st_polygon(list(coords[, 1:3]))
#' xym <- st_polygon(list(coords[, 1:3]), dim = "XYM")
#' xyzm <- st_polygon(list(coords))
#'
#' as_esri_geometry(xy)
#' as_esri_geometry(xyz)
#' as_esri_geometry(xym)
#' as_esri_geometry(xyzm)
#'
#' # MULTIPOLYGON
#' as_esri_geometry(st_multipolygon(list(xy, xy)))
#' as_esri_geometry(st_multipolygon(list(xyz, xyz)))
#' as_esri_geometry(st_multipolygon(list(xym, xym)))
#' as_esri_geometry(st_multipolygon(list(xyzm, xyzm)))
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
