# esrisop

Reads Esri's Spatially Optimized Parquet with spatial filters, without IO. The `geometry` column is a `binary` column of Esri shape buffers with `esri.encoding` and `esri.sr_wkt` field metadata, as ArcGIS reads them from Arrow.
