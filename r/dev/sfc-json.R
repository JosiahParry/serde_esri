# TODO skip serialize non for Features e.g. objectIdFieldName, globalIdFieldName, etc.
# and notably fields
# sfc_POINT --------------------------------------------------------------

x <- rnorm(12)

# XY
pnt2d <- sf::st_multipoint(matrix(x, ncol = 2)) |>
  sf::st_sfc() |>
  sf::st_cast("POINT")

sfc_point_features_2d(pnt2d)
sfc_point_featureset_2d(pnt2d, list())

# XYZ
pnt3d <- sf::st_multipoint(matrix(x, ncol = 3)) |>
  sf::st_sfc() |>
  sf::st_cast("POINT")

sfc_point_features_3d(pnt3d)
sfc_point_featureset_3d(pnt3d, list())

# XYM
pnt3d <- matrix(rnorm(12), ncol = 3) |>
  apply(1, sf::st_point, dim = "XYM", simplify = FALSE) |>
  sf::st_sfc()

sfc_point_features_3d(pnt3d)
sfc_point_featureset_3d(pnt3d, list())

# sfc_MULTIPOINT ---------------------------------------------------------

# XY
mpnt2d <- st_sfc(st_multipoint(matrix(x, ncol = 2)))

sfc_multipoint_features_2d(mpnt2d)
sfc_multipoint_featureset_2d(mpnt2d, list(latest_wkid = 121001))


# XYZ
mpnt3d <- st_sfc(st_multipoint(matrix(x, ncol = 3)))

sfc_multipoint_features_3d(mpnt3d)
sfc_multipoint_featureset_3d(mpnt3d, list(latest_wkid = 3857))

# XYM
mpnt3d <- st_sfc(st_multipoint(matrix(x, ncol = 3), "XYM"))

sfc_multipoint_features_3d(mpnt3d)
sfc_multipoint_featureset_3d(mpnt3d, list(latest_wkid = 3857))


# sfc_LINESTRING ---------------------------------------------------------

lns2d <- st_sfc(st_linestring(matrix(x, ncol = 2)))

sfc_linestring_features_2d(lns2d)
sfc_linestring_featureset_2d(lns2d, list())

# XY
mpnt2d <- st_sfc(st_multipoint(matrix(x, ncol = 2)))

sfc_multipoint_features_2d(mpnt2d)
sfc_multipoint_featureset_2d(mpnt2d, list(latest_wkid = 121001))


# XYZ
lns3d <- st_sfc(st_linestring(matrix(x, ncol = 3)))

sfc_linestring_features_3d(lns3d)
sfc_linestring_featureset_3d(lns3d, list(latest_wkid = 3857))

# XYM
lns3d <- st_sfc(st_linestring(matrix(x, ncol = 3), "XYM"))

sfc_linestring_features_3d(lns3d)
sfc_linestring_featureset_3d(lns3d, list(latest_wkid = 3857))


# sfc_MULTILINESTRING ----------------------------------------------------

# XY
mlns <- st_sfc(st_multilinestring(list(lns2d[[1]], lns2d[[1]])))

sfc_multilinestring_features_2d(mlns)
sfc_multilinestring_featureset_2d(mlns, list())

# XYZ
lns3d <- st_linestring(matrix(x, ncol = 3))
mlns3d <- st_sfc(st_multilinestring(list(lns3d, lns3d)))

sfc_multilinestring_features_3d(mlns3d)
sfc_multilinestring_featureset_3d(mlns3d, list())

# XYM
lns3d <- st_linestring(matrix(x, ncol = 3))
mlns3d <- st_sfc(st_multilinestring(list(lns3d, lns3d), "XYM"))

sfc_multilinestring_features_3d(mlns3d)
sfc_multilinestring_featureset_3d(mlns3d, list())

# sfc_POLYGON ------------------------------------------------------------

coords <- rbind(
  c(0, 0, 0),
  c(0, 1, 0),
  c(1, 1, 1),
  c(1, 0, 1),
  c(0, 0, 0)
)

# XY
poly2d <- sf::st_sfc(st_polygon(list(coords[, 1:2])))

sfc_polygon_features_2d(poly2d)
sfc_polygon_featureset_2d(poly2d, list())

# XYZ
poly3d <- sf::st_sfc(st_polygon(list(coords)))

sfc_polygon_features_3d(poly3d)
sfc_polygon_featureset_3d(poly3d, list())

# XYM
poly3d <- sf::st_sfc(st_polygon(list(coords), "XYM"))

sfc_polygon_features_3d(poly3d)
sfc_polygon_featureset_3d(poly3d, list())

# sfc_MULTIPOLYGON -------------------------------------------------------

# XY
mpoly2d <- st_sfc(
  st_multipolygon(list(poly2d[[1]], poly2d[[1]]))
)

sfc_multipolygon_features_2d(mpoly2d)
sfc_multipolygon_featureset_2d(mpoly2d, list())

# XYZ
mpoly3d <- st_sfc(st_multipolygon(list(poly3d[[1]], poly3d[[1]])))
sfc_multipolygon_features_3d(mpoly3d)
sfc_multipolygon_featureset_3d(mpoly3d, list())

mpoly3d <- st_sfc(st_multipolygon(list(poly3d[[1]], poly3d[[1]]), "XYM"))
sfc_multipolygon_features_3d(mpoly3d)
sfc_multipolygon_featureset_3d(mpoly3d, list())
