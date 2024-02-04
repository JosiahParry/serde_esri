# library(serdesri)
#
# no_geo_url <- "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/ACS_Population_by_Race_and_Hispanic_Origin_Boundaries/FeatureServer/2/query?where=1=1&outFields=*&resultRecordCount=2000&f=json&returnGeometry=true"
#
# req <- httr2::request(no_geo_url)
# resp <- httr2::req_perform(req)
# json <- httr2::resp_body_string(resp)
#
# stream <- parse_esri_json_str(json, 2)
# df <- as.data.frame(stream)
# head(df$geometry)
#
#
# bm <- bench::mark(
#   current = arcgisutils::parse_esri_json(json),
#   # arrow = arrow::as_record_batch_reader(parse_esri_json_str(json, 2))$read_table() ,
#   serde_json = as.data.frame(parse_esri_json_str(json, 2)),
#   # simd_json = as.data.frame(parse_esri_json_str_simd(json, 2)),
#   iterations = 10,
#   check = FALSE,
#   # relative = TRUE
# ) |>
#   dplyr::select(1:7) |>
#   print()
#
#
