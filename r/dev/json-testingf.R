library(sf)

st_point() |>
  as_point()

st_point(c(1, 2)) |> as_point()

st_point(c(1, 2, 3)) |> as_point()
st_point(c(1, 2, 3), dim = "XYM") |> as_point()
st_point(c(1, 2, 3, 4)) |> as_point()

# linestring -------------------------------------------------------------

m <- matrix(c(as.double(1:10)), ncol = 2)

st_linestring(m) |>
  as_linestring_polyline()

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

st_multilinestring() |>
  as_polyline()

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
sfdep::guerry$geometry[[1]] |>
  as_polygon()

sfdep::guerry$geometry[[1]] |>
  st_cast("POLYGON") |>
  as_poly_polygon()



# xy with xyz ------------------------------------------------------------

sf::st_sfc(st_linestring(z), st_linestring(z, "XYM"))
