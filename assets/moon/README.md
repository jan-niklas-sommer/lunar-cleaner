# Höhendaten Mond

`heights.png` ist **selbst erzeugt** von `game/tools/planet_data.py` (16-Bit-Graustufen, echte Höhe in m + 32 768, 2880 × 1441 Knoten im 0,125°-Raster, Zeile 0 = 90° N, Spalte 0 = 180° W, Längengrad nach Osten). Die Rohdaten liegen nicht im Repo.

## Quelle und Lizenz

- LRO LOLA Gridded Data Record `ldem_16` (16 Pixel/Grad), NASA PDS Geosciences Node (`pds-geosciences.wustl.edu`). NASA-Daten, gemeinfrei.

## Neu erzeugen

```powershell
python -P game/tools/planet_data.py <Ordner mit den Rohdaten> game/assets
```
