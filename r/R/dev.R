# library(serdesri)
#
# url <- "https://services.arcgis.com/P3ePLMYs2RVChkJx/arcgis/rest/services/ACS_Population_by_Race_and_Hispanic_Origin_Boundaries/FeatureServer/2/query?where=1=1&outFields=objectid&resultRecordCount=10&f=json"
#
# req <- httr2::request(url)
# resp <- httr2::req_perform(req)
# json <- httr2::resp_body_string(resp)
#
# stream <- parse_esri_json_str(json, 2)
# stream
#
# df <- as.data.frame(stream)
#
# str(df, 1)
