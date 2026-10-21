# Erddaten

Die Erde im Spiel basiert auf der echten Erde (Entscheidung 2026-10-21, ersetzt D2 aus `docs/phase-3-terrain-feinschliff.md`). Die Dateien hier sind **selbst erzeugt** von `game/tools/earth_data.py` aus gemeinfreien Quellen; die Rohdaten liegen nicht im Repo.

| Datei | Inhalt |
|---|---|
| `heights.png` | 16-Bit-Graustufen, echte Höhe in m + 11 000, 3600 × 1801 (0,1°), Zeile 0 = 90° N, Spalte 0 = 180° W. Binnensenken ohne Meeresanschluss sind Land, Seen tragen ihren Wasserspiegel. |
| `climate.png` | RGB im selben Raster: R = Wüstenanteil, G = Eisanteil, B = Seeanteil. |
| `rivers.bin` | Flussläufe flussabwärts mit fallendem Wasserspiegel und Breite (Format im Tool beschrieben). |

## Quellen und Lizenzen

- **Höhen:** NOAA ETOPO1 (Amante, C. and B.W. Eakins, 2009: ETOPO1 1 Arc-Minute Global Relief Model. NOAA Technical Memorandum NESDIS NGDC-24), abgerufen als 0,1°-Raster über den ERDDAP-Server `upwell.pfeg.noaa.gov` (Datensatz `etopo180`). US-Regierungswerk, gemeinfrei.
- **Klima (Wüste/Eis):** NASA Visible Earth, Blue Marble Next Generation (`world.topo.bathy.200412.3x5400x2700.jpg`), Reto Stöckli, NASA Earth Observatory. Gemeinfrei, Quellenangabe erbeten.
- **Flüsse und Seen:** Natural Earth, 1:10m `rivers_lake_centerlines` und `lakes` (naturalearthdata.com, über `github.com/nvkelso/natural-earth-vector`). Gemeinfrei.

## Neu erzeugen

```powershell
python -P game/tools/earth_data.py <Ordner mit den Rohdaten> game/assets/earth
```
