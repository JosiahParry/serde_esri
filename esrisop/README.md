# esrisop

`esrisop` reads Esri's Spatially Optimized Parquet with spatial filters and does no IO itself. Its decoder writes the `geometry` column as a `binary` column of Esri shape buffers with `esri.encoding` and `esri.sr_wkt` field metadata, the form ArcGIS reads from Arrow.
