# sfc_POINT --------------------------------------------------------------

pnt2d <- sf::st_multipoint(matrix(rnorm(10), 5)) |>
  sf::st_sfc() |>
  sf::st_cast("POINT")

pnt2d[[1]] |> class()

arcgisutils::as_esri_features(pnt2d)


# XYZ
pnt3d <- sf::st_multipoint(matrix(rnorm(12), 4)) |>
  sf::st_sfc() |>
  sf::st_cast("POINT")


# XYZM
pnt3d <- sf::st_multipoint(matrix(rnorm(12), 3)) |>
  sf::st_sfc() |>
  sf::st_cast("POINT")


(mpnt <- sf::st_multipoint(matrix(rnorm(12), 4)))

sf::st_sfc(mpnt) |>
  sf::st_cast("POINT") |>

  as_point_features_3d()

sf::st_sfc(mpnt) |> sf::st_zm() |> 
  as_point_featureset_2d(list(wkid = 4326))

