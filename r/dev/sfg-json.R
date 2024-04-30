library(sf)

st_point() |>
  sfg_point_as_point()

st_point(c(1, 2)) |> as_point()

st_point(c(1, 2, 3)) |> as_point()
st_point(c(1, 2, 3), dim = "XYM") |> as_point()
st_point(c(1, 2, 3, 4)) |> as_point()

# linestring -------------------------------------------------------------

m <- matrix(c(as.double(1:10)), ncol = 2)

st_linestring(m) |>
  sfg_linestring_as_polyline()

st_linestring(m, dim = "XYZ") |>
  as_linestring_polyline()


z <- matrix(c(as.double(1:12)), ncol = 3)

st_linestring(z) |>
  as_linestring_polyline()

st_linestring(z, "XYM") |>
  as_linestring_polyline()

zm <- matrix(round(rnorm(12), 1), ncol = 4)

st_linestring(zm) |>
  as_linestring_polyline()




# multilinestring --------------------------------------------------------
m <- matrix(x, ncol = 2)
z <- matrix(x, ncol = 3)
zm <- matrix(x, ncol = 4)
st_multilinestring(m)

# XY
st_multilinestring(list(m, m)) |>
  as_polyline()


# XYZ
st_multilinestring(list(z, z)) |>
  as_polyline()

# XYM
st_multilinestring(list(z, z), dim = "XYM") |>
  as_polyline()


# XYZM
st_multilinestring(list(zm, zm)) |>
  as_polyline()


# polygon ----------------------------------------------------------------

coords <- rbind(
  c(0, 0, 0),
  c(0, 1, 0),
  c(1, 1, 1),
  c(1, 0, 1),
  c(0, 0, 0)
)


# xy
poly2d <- st_polygon(list(coords[, ]))
as_poly_polygon(poly2d)


# xyz
poly3d <- st_polygon(list(coords))
as_poly_polygon(poly3d)


# xyzm
poly4d <- st_polygon(list(cbind(coords, 1)), dim = "XYZM")
as_poly_polygon(poly4d)


# multipolygon -----------------------------------------------------------

# xy
mpoly2d <- st_multipolygon(list(list(coords[, 1:2])))
as_polygon(mpoly2d)

mpoly3d <- st_multipolygon(list(list(coords)))
as_polygon(poly_3d)


mpoly4d <- st_multipolygon(list(list(cbind(coords, 1))))
as_polygon(mpoly4d)
