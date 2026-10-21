# AGENTS.md — Lunar Cleaner (Bevy Edition)

## 1. Projektübersicht & Rolle des Agenten
Du bist ein autonomer Senior Rust & Bevy Game Engine Architect. Deine Aufgabe ist es, das Spiel **Lunar Cleaner** als vollständiges, spielbares 3D-First-Person-Koop-Spiel in **Rust** mit der **Bevy Engine** zu entwickeln.

### Das Spielkonzept (Core Loop)
Das *Lunar Surface Department (LSD)* schickt bis zu 16 Spieler auf den verdreckten Mond, um die gesamte Oberfläche von Mondstaub zu befreien.
1. **Putzen & Sammeln:** Spieler starten chronisch unterfinanziert mit einer bloßen **Zahnbürste**. Sie tragen den Staub physisch und visuell sichtbar vom Mond-Terrain ab, wodurch sich ihr Inventar füllt.
2. **Abliefern & Verschiffen:** Ist das Inventar voll, bringen die Spieler den Staub zu einem **Depot**, lagern ihn ein und starten eine Frachtkapsel (Cargo Pod), um den Staub an das LSD zu verkaufen.
3. **Upgraden & Expandieren:** Mit den verdienten Credits kaufen Spieler größere Rucksäcke, bessere Werkzeuge (Besen, Mond-Sauger, Industrie-Fräsen), errichten neue Depots direkt an der Arbeitsfront und schalten fahrbare **Reinigungsfahrzeuge (Lunar Rovers)** frei.
4. **Globales Ziel:** Ein Fortschrittsbalken zeigt den globalen Reinigungsgrad der offenen Mondwelt in Prozent (von $0.00\%$ bis $100.00\%$). Es gibt keinen Zeitdruck oder Überlebensstress – der Fokus liegt auf befriedigendem Fortschritt, Basisbau und Koop-Chaos.

---

## 2. Tech-Stack & Abhängigkeiten (`Cargo.toml`)
Verwende aktuelles, idiomatisches Bevy (ECS-Architektur). Halte die Abhängigkeiten sauber und kompatibel:

* **Core Engine:** `bevy = "=0.20.0"` — **strikt gepinnt!** Die `Cargo.toml` existiert bereits in diesem Repo und der vorhandene Code (`height_maps`, `world_origin`, `starfield`, `sky`) ist genau gegen diese Version geschrieben. Nicht upgraden.
* **Physik & Kollision:** `avian3d` (bevorzugt wegen nativer Bevy-ECS-Integration) oder `bevy_rapier3d` — **nur in einer Version, die mit Bevy 0.20 kompatibel ist** (vor dem Anlegen prüfen).
* **Input Management:** `leafwing-input-manager` (für nahtlosen Support von Maus/Tastatur sowie Gamepads wie DualShock/DualSense) — ebenso Bevy-0.20-kompatibel wählen.
* **Networking (16-Spieler Koop):** `bevy_renet` oder `lightyear` (Autoritativer Host/Server mit Delta-Kompression für Staub-Chunks) — kommt erst in Phase 6; dann die Version zur gepinnten Bevy-Version wählen.
* **Serialisierung:** `serde`, `bincode` (für Netzwerk-Pakete und Spielstände)
* **Zufall & Terrain-Noise:** **kein `noise`-Crate nötig** — `src/noise.rs` bringt deterministisches f32/f64-Noise (Perlin, fBm, Worley, Rng) bereits mit.

### Cargo.toml Optimierung (bereits erledigt!)
Die Profile stehen schon in der `Cargo.toml` dieses Repos (`opt-level = 1` fürs eigene Crate, `3` für Dependencies, `dev`-Feature für `dynamic_linking`). Nicht doppelt anlegen.

---

## 2.5 Vorhandene Grundlagen in diesem Repo (NICHT neu erfinden!)
Dieses Repo enthält bereits lauffähige, getestete Grundlagenmodule (portiert aus dem Schwesterprojekt `spcgame`; `cargo test` ist grün). Baue darauf auf, statt sie neu zu schreiben:

| Datei | Inhalt | Rolle im Spiel |
| :--- | :--- | :--- |
| `src/moon_data.rs` | Der echte Mond als Höhenfeld: echte LRO-LOLA-Höhen + prozedurale Kleinkrater (Zellen 2400/700 m, kraterarme Maria) + Feindetail. `Region::at(lon, lat, size)` sampelt eine Tangentialebene (x = Ost, z = Nord); `apollo_11()` ist die Startregion. | **Höhenquelle für alle `DustChunk`s** (`region.height(x, z)`) und Mare-Färbung (`moon_data::mare`) |
| `src/height_maps.rs` | Dekodierung + bilineares Sampling der echten Höhenkarten (Mond + Erde), `lon_lat`/`dir_of`-Konversion | Wird von `moon_data` und `sky` benutzt |
| `src/sky.rs` | Sonne (HDR-Kugel + Corona-Billboard + gerichtetes Licht, scheinbare Größe 0,53°), Erde als Kugel (1,9°) mit beim Start aus echten ETOPO1-/Blue-Marble-Daten erzeugter Textur, Restlicht — alles an einem kamerafolgenden SkyRig | Der fertige Himmel: Spieler sehen Sonne, Erde und Sterne |
| `src/starfield.rs` | Prozedurale Sternen-Cubemap als Skybox | Fertig, als Plugin behalten |
| `src/world_origin.rs` | Floating Origin (f64-Absolutpositionen, Recenter ab 1000 m) | **Pflicht für alles, was sich durch die Welt bewegt** — Spieler, Fahrzeuge, Depots erhalten `AbsolutePosition` |
| `src/noise.rs` | Deterministisches Noise (f64-Perlin/fBm für Terrain, f32-Value-Noise für Farben), eigener Rng | Für Effekte; niemals ein Noise-Crate nachziehen |
| `src/palette.rs` | Zentrale Farbpalette | Alle neuen Farben hier ergänzen |
| `src/moon_surface.rs` | Vorschau-Mesh der Region (8-m-Raster mit Vertexfarben) + `PlayRegion`-Resource | Referenz; wird von den feinen `DustChunk`s (0,25 m) abgelöst |
| `src/fly_camera.rs` | Freiflug-Kamera (WASD + Maus, Klick fängt die Maus, Esc gibt frei) | Wird in Phase 1 durch den echten Spieler-Controller ersetzt |
| `assets/moon/`, `assets/earth/` | Echte NASA-/NOAA-Daten (Quellen und Lizenzen in den dortigen `README.md`) | Beim Kompilieren eingebettet (`include_bytes!`) |

`cargo run` startet die Vorschau: die Apollo-11-Region unter Sonne, Erde und Sternenhimmel.

**Verbindliches Design-Dokument für Wirtschafts- und Progression-Zahlen:** `design_wirtschaft_und_progression.md` (Werkzeuge, Rucksäcke, Anzug-Upgrades, Fahrzeuge, Automatisierung, Gebäude, Meilensteine, Fund-Tabelle, technische Anbindung an die DustChunks). Bei Konflikten zwischen diesem Dokument und älteren Tabellen hier gilt das Design-Dokument.

---

## 2.6 Weltgröße & Skalierung (verbindlich!)
**Kein Mond in „Originalgröße":** Der echte Mond hat 37,9 Mio. km² Oberfläche — das wären ~600 TB nur an Staubdaten und ~10.000 Jahre Putzzeit selbst mit dem Rover. Stattdessen:

* Die Spielwelt ist die **Startregion auf dem echten Mond**: zunächst 4 × 4 km um die Apollo-11-Landestelle im Mare Tranquillitatis (23.473° O, 0.674° N), Ausbau auf die **Zielgröße ~50 km² (≈ 7×7 km)** laut `design_wirtschaft_und_progression.md` — derselbe `Region::at(...)`-Ausschnitt, nur größer gezogen. Die „dunkle Seite" (Meilenstein 25 %) ist eine zweite echte Region auf der Mond-Fernseite.
* Höhen sind **realmaßstäblich** (echte LOLA-Höhen, echte Kratertiefen). Der „perfekte Mond" entsteht durch echte Daten, nicht durch Originalgröße.
* **Floating Origin ist Pflicht** (`world_origin.rs` existiert): Alles, was Positionen über größere Distanzen hält, in f64 `AbsolutePosition` speichern. Nie f32-Weltkoordinaten über wenige Kilometer hinaus akkumulieren.
* Sonne (0,53°) und Erde (1,9°) stehen in korrekter scheinbarer Größe und fest am Himmel (gebundene Rotation) — bereits in `sky.rs` umgesetzt.

---

## 3. Verzeichnis- und Plugin-Architektur
Schreibe niemals monolithische Dateien. Jedes Subsystem muss als eigenes Bevy `Plugin` gekapselt sein. Die Fundament-Module (unten ohne `# NEU`-Markierung) **existieren bereits** — erweitere sie, statt sie zu ersetzen:

```text
src/
├── main.rs                 # App-Einstiegspunkt, WindowPlugin, DefaultPlugins
├── lib.rs                  # NEU: Bündelt alle Spiel-Plugins im LunarCleanerPlugin
├── states.rs               # NEU: AppState (MainMenu, Connecting, InGame, Paused) & UI-SubStates
├── moon_data.rs            # Echte Mond-Höhen + Krater + Region (bestehen lassen!)
├── height_maps.rs          # Echte Karten: Dekodierung/Sampling (bestehen lassen!)
├── sky.rs                  # Sonne, Erde, Licht (bestehen lassen!)
├── starfield.rs            # Sternen-Skybox (bestehen lassen!)
├── world_origin.rs         # Floating Origin (bestehen lassen!)
├── noise.rs                # Deterministisches Noise (bestehen lassen!)
├── palette.rs              # Farb-Palette (bestehen lassen!)
├── moon_surface.rs         # Vorschau-Mesh + PlayRegion-Resource (von DustChunks ablösen)
├── fly_camera.rs           # Vorschau-Kamera (in Phase 1 durch Spieler-Controller ersetzen)
├── assets/
│   └── procedural_meshes.rs# NEU: Mesh-Generatoren für Zahnbürste, Depot, Rover, …
└── plugins/
    ├── mod.rs              # NEU
    ├── player.rs           # NEU: FPS-Controller, Mond-Gravitation, Kamera, Interaktions-Raycast
    ├── dust_terrain.rs     # NEU: Chunk-basiertes Mond-Terrain, Splatmap-Maske, Fortschrittsberechnung
    ├── tools.rs            # NEU: Werkzeuge (Zahnbürste bis Laser-Sauger), Tool-Animationen, Abtrag-Logik
    ├── inventory_economy.rs# NEU: Staubbehälter & Kompression, LSD-Credits, Upgrade-Shop-Preise
    ├── depots.rs           # NEU: Platzierbare Depots, Staub-Transfer, Frachtkapsel-Startsequenz
    ├── vehicles.rs         # NEU: Reinigungsfahrzeuge, Ein-/Aussteigen, Fahrphysik, Walzen-Reinigung
    ├── networking.rs       # NEU: 16-Spieler Lobby, Spieler-Sync, Chunk-Delta-Sync
    └── ui.rs               # NEU: HUD (Inventar, Credits, Mond-Sauberkeit in %), Upgrade-Menü, Bau-Menü
```

---

## 4. Detaillierte Feature-Spezifikationen (ECS-Design)

### 4.1. Sichtbares Staub-System (`dust_terrain.rs`)
Das Kernstück des Spiels ist das visuell befriedigende Abtragen von Mondstaub.
* **Architektur:** Die Spielregion (siehe 2.6, `PlayRegion`-Resource) ist in ein Grid aus Chunks unterteilt (`DustChunk`, z. B. $32 \times 32$ Meter pro Chunk, aufgeteilt in ein $128 \times 128$ Staub-Zellen-Raster).
* **Höhenquelle (WICHTIG):** Chunk-Vertices sampeln **nicht** zufälliges Noise, sondern `region.height(x, z)` aus `moon_data` — echte LOLA-Höhen plus die prozeduralen Kleinkrater. Chunk-Grenzen sind höhenstetig, weil alle Chunks dieselbe Weltfunktion fragen. Die Farbe mischt wie im Vorschau-Mesh `palette::ROCK`/`palette::MOON_MARE` nach `moon_data::mare`.
* **Datenstruktur:**
  ```rust
  #[derive(Component)]
  pub struct DustChunk {
      pub chunk_coords: IVec2,
      pub cells: Vec<u8>, // 0 = komplett sauber, 255 = maximaler Staub
      pub dirty_render: bool,
      pub dirty_network: bool,
      pub cleaned_cells_count: usize,
  }

  #[derive(Resource)]
  pub struct GlobalCleaningProgress {
      pub total_cells: u64,
      pub cleaned_cells: u64,
      pub percentage: f32,
  }
  ```
* **Visuelle Darstellung:**
  * Jeder Chunk besitzt eine dynamische `Image`-Textur (R8Unorm Maske), die als Splatmap/Blend-Maske im Material dient.
  * Wo `cells[i] > 0` ist, wird dunkler, rauer Regolith-Staub gerendert. Wo `cells[i] == 0` ist, kommt der glänzende, helle, saubere Mondfels zum Vorschein.
  * Wenn `dirty_render == true`, aktualisiert ein System die `Image`-Daten des Chunks auf der GPU.
* **Reinigungsvorgang:** Ein Event `CleanDustEvent { world_pos: Vec3, radius: f32, strength: u8, collector: Entity }` reduziert die Werte im Radius, berechnet die tatsächlich abgetragene Staubmenge, fügt sie dem Inventar des `collector` hinzu und spawnt Partikel.

### 4.2. Spieler & First-Person-Controller (`player.rs`)
* **Mond-Physik:** Gravitation ist auf Mond-Niveau eingestellt ($\approx 1.62\,\text{m/s}^2$ oder für besseres Spielgefühl leicht erhöht auf $3.5\,\text{m/s}^2$).
* **Steuerung (`leafwing-input-manager`):**
  * `Move` (WASD / Left Stick)
  * `Look` (Mouse Delta / Right Stick)
  * `Jump` (Space / South Button)
  * `UseTool` (Left Click / Right Trigger)
  * `Interact` (E / West Button – für Depots, Shop-Terminals und Fahrzeuge)
  * `BuildMenu` (B / North Button – um neue Depots zu platzieren)
  * `CycleTool` (Mouse Wheel / Bumpers)

### 4.3. Werkzeuge & Progression (`tools.rs` & `inventory_economy.rs`)
**Verbindliche Quelle für alle Zahlen ist `design_wirtschaft_und_progression.md`** (8 Werkzeug-Stufen T1–T8, 7 Staubbehälter-Stufen, Anzug-Upgrades, Kosten-Skalierung ×1,35, Zufalls- & Tiefen-Funde, Bezirks-Bonus, Koop-Wallet, Meilensteine, Automatisierung mit Fließbändern, Veredelungs-Gebäude, Offline-Fortschritt). Kurzfassung der wichtigsten Regeln:

* Einheit ist **Liter Regolith**: 1 m² Oberfläche = 10 L Staub, 1 L Basisverkaufswert = 1 ₡. Staubmenge, Behälter-Kapazität und Werkzeug-Abtrag (L/s) laufen alle in Litern.
* **Kein abstraktes Inventar — Staubbehälter sind physische Entities** (`DustContainer { tier, current_liters, max_liters }`): Hand-Geräte mit Sichtfenster und LED-Füllstand; getragen (Gewicht bremst), daneben abgestellt (fängt den Auffang-Strahl ein), gestapelt, kickbar, verladbar auf Fließbänder/Rohrpost/Fahrzeuge (siehe Design-Dokument Säule B/D). **Der Anzug selbst speichert keinen Staub** — abgetragener Staub fliegt als sichtbarer Partikelstrahl in den Behälterstutzen.
* **Logistik immer über Behälter, Entladen nur über Chutes:** `DustChute` (an Depot/Fahrzeug, auch frei platzierbar) ist das einzige Bauteil, das Behälter entleert — animierter Partikel-Wasserfall ins Ziel. Fahrzeugtanks sind fest verbaut und werden nur über ihr eigenes Chute gefüllt.
* Jede Werkzeug-Stufe ändert nicht nur Zahlen, sondern das **Spielgefühl** (T4 Laubbläser = Low-G-Jetpack-Rückstoß, T7 Plasma verglast den Boden zu schnellen Pfaden, …) — die Spezialeffekte der Tabelle sind Pflicht, nicht Deko.
* Alle Wirtschafts-Konstanten in **einem Balancing-Modul `economy.rs`** sammeln, damit Balancing ohne Logik-Änderung möglich ist. Keine magischen Zahlen in den Systemen.
* Zufallsfunde: 2 % Chance pro abgetragenem m² (Apollo-Goldfolie 250 ₡, Helium-3-Knolle 1.500 ₡, Alien-Gartenzwerg 10.000 ₡); in Kraterinnenen (erkennbar über `moon_data`-Höhen) verdoppelte Chance. Funde gehen als privates Taschengeld an den Finder, alles andere ins geteilte `TeamWallet`.

### 4.4. Depots & Basisbau (`depots.rs`)
* **Funktionsweise:** Da die Mondwelt groß ist, werden Laufwege zum Start-Depot schnell zu weit. Spieler können Credits ausgeben, um im **Build-Modus** neue Depots an der Arbeitsfront zu errichten.
* **Geister-Vorschau (Placement System):** Im Baumodus projiziert ein Raycast vor dem Spieler ein halbtransparentes Depot-Modell (Grün = baubar, Rot = Kollision/zu steil).
* **Abgabe über Entlade-Chute:** Jedes Depot (und jedes Reinigungsfahrzeug) hat ein `DustChute` mit Einhänge-Slot: Behälter einhängen, umkippen — der Staub rauscht als animierter Partikel-Wasserfall ins Silo. Chutes sind die einzige Art, wie Behälter ihren Inhalt loswerden.
* **Depot-Interaktion:**
  * Hat ein Depot gespeicherten Staub, kann ein Spieler die **Frachtkapsel (Cargo Pod)** starten.
  * Die Kapsel hebt physisch/visuell mit einem Partikelschweif in den Himmel ab, fliegt in den Orbit und schreibt nach $5$ Sekunden die Credits auf dem Team-Konto (`TeamWallet`) gut — Zufallsfunde gehen als Taschengeld an den Finder (siehe Design-Dokument).

### 4.5. Reinigungsfahrzeuge (`vehicles.rs`)
**Alle Fahrzeug-Typen, Kosten, Tanks und Spaß-Mechaniken stehen in `design_wirtschaft_und_progression.md` (Säule C):** 7 Fahrzeuge vom Aufsitzmäher (1.500 ₡) bis zum 16-Sitz-Bagger (8,5 Mio. ₡). Gemeinsame Mechanik für alle:

* **Spawnen:** Fahrzeuge werden am Fahrzeug-Pad bzw. an der Basis für Credits gespawnt (16 Sitzplätze beim Bagger = volles Koop-Potenzial).
* **State-Transition:** Drückt der Spieler `Interact` neben einem Fahrzeug:
  1. Spieler-Collider wird deaktiviert, Spieler-Entity wird als Kind an den Fahrersitz geheftet.
  2. Kamera wechselt zu einer Third-Person-Orbitalkamera (oder Cockpit-Ansicht).
  3. Input-Map wechselt von `PlayerAction` auf `VehicleAction` (Gas, Bremse, Lenken, Reinigung Einschalten, Aussteigen).
* **Fahrzeug-Reinigung:** Aktive Reinigung sendet kontinuierlich `CleanDustEvent`s mit der Reinigungs-Breite des Fahrzeugs (siehe Design-Tabelle) und speichert den Staub im `VehicleInventory` (jeweilige Tankgröße aus der Tabelle).
* **Entladen:** Fährt das Fahrzeug in die Entladezone eines Depots, wird der Tank automatisch in das Depot gepumpt.
* **Fahrphysik:** Übertriebene 1/6-Gravitations-Arcade-Physik (Kippen, Driften, Schweben) — siehe Spaß-Mechaniken der Tabelle.

### 4.6. 16-Spieler Online-Koop (`networking.rs`)
* **Topologie:** Host-Server oder Dedicated Server für bis zu $16$ Clients.
* **Synchronisierte Elemente:**
  * `PlayerTransform` & `EquippedTool` (Unreliable, 20 Hz).
  * `CleanDustEvent`: Wird an den Server gesendet, Server validiert, aktualisiert den `DustChunk` und broadcastet komprimierte Chunk-Updates an alle Clients.
  * `TeamWallet` (Credits sind im Koop geteilt oder optional individuell, Standard: Geteiltes LSD-Budget).
  * Platzierte `Depot`-Entities und `Vehicle`-Positionen.
* **Offline/Singleplayer-Fallback:** Das Spiel muss im Singleplayer exakt dieselbe Logik nutzen (lokaler Loopback oder direkte Ausführung der Server-Systeme), sodass es sofort per `cargo run` ohne Netzwerk-Setup spielbar ist.

---

## 5. Prozedurale Assets & Visuelles Feedback (WICHTIG!)
Da du als KI-Agent keinen Zugriff auf externe 3D-Modelle (`.glb`) oder Texturen hast, **musst du alle Modelle und Texturen zur Laufzeit prozedural in Code generieren** (`src/assets/procedural_meshes.rs`):

1. **Die Zahnbürste:** Kombiniere einen schmalen langen `Cuboid` (Griff in LSD-Gelb/Orange) mit einem kleinen weißen `Cuboid` an der Spitze (Borsten). Animierbar über eine Sinus-Wellen-Rotation auf dem Viewmodel, wenn die Maustaste gehalten wird.
2. **Bessere Werkzeuge:** Baue Besen, Staubsauger (aus `Cylinder`, `Torus`, `Capsule3d`) und Schläuche modular als Child-Entities zusammen.
3. **Der Mondboden:** Kein Noise-Terrain! Die Höhen kommen aus `moon_data` (echte LRO-LOLA-Karte + prozedurale Kleinkrater, siehe 2.5). Nur die Staub-Textur wird prozedural im Speicher erzeugt (`Image::new_fill`).
4. **Depots & Frachtkapseln:** Baue das Depot aus einer Plattform (`Cuboid`), einem Silo-Tank (`Cylinder`), einer Status-Leuchte (`Sphere` mit `emissive` Material) und einer darauf sitzenden Raketenkapsel (`Cylinder` + `Cone`).
5. **Reinigungs-Rover:** Chassiskörper + 4 rotierende Räder (`Cylinder`) + breite rotierende Putzwalze an der Front + Warnblinklicht auf dem Dach.
6. **Himmel & Atmosphäre:** **Existiert bereits fertig in `sky.rs` + `starfield.rs`** — Sonne als HDR-Kugel mit Corona (0,53° scheinbar), Erde als echte Textur-Kugel (1,9°, „volle Erde", langsam rotierend), Sternenfeld als Skybox, gerichtetes Licht mit 14.000 Lux plus schwaches blaues Restlicht. Nicht neu bauen; bei Bedarf nur anpassen (z. B. Erdrotation langsamer).

---

## 6. Schrittweiser Entwicklungsplan (Roadmap für den Agenten)

Arbeite diese Phasen strikt nacheinander ab. Nach **jeder** Phase muss `cargo check` und `cargo test` fehlerfrei durchlaufen!

### Phase 1: Projekt-Setup & Mond-Umgebung
- [x] `Cargo.toml` existiert (Bevy gepinnt, Profile gesetzt); `lib.rs`, `states.rs` und `plugins/` anlegen.
- [x] Fenster, Kamera, Weltraum-Beleuchtung, Sonne/Erde/Sterne existieren (`sky.rs`, `starfield.rs`, `fly_camera.rs`) — nur prüfen und behalten.
- [ ] Mond-Terrain als `DustChunk`-Grid aus `moon_data` sampeln (ersetzt das Vorschau-Mesh `moon_surface.rs`) und Physik-Collider aus den Chunk-Höhen bauen.
- [ ] First-Person-Spieler mit Mond-Gravitation, Sprungmechanik und Maus/Controller-Umschauen implementieren (ersetzt `fly_camera.rs`; Spieler-Positionen als `AbsolutePosition` führen, siehe 2.6).

### Phase 2: Das Splatmap-Staub-System & Die Zahnbürste
- [ ] `DustChunk`-Grid über das Terrain legen und dynamische Splatmap-Textur-Updates implementieren.
- [ ] Viewmodel für die **Zahnbürste** (und die weiteren 4 Werkzeuge) prozedural bauen.
- [ ] Raycast vom Fadenkreuz auf den Boden: Bei gedrückter Taste Staub im Werkzeug-Radius abtragen, Splatmap-Textur live aktualisieren und Staub-Partikel spawnen.
- [ ] `GlobalCleaningProgress` berechnen und testen.

### Phase 3: Staubbehälter, Depots & Wirtschaft (Core Loop)
- [ ] Staubbehälter als physische Entities (`DustContainer`, T1–T2) implementieren: in der Hand tragen (Gewicht bremst), neben sich abstellen, wieder aufnehmen, kicken; Sichtfenster + LED-Füllstand.
- [ ] **Auffang-Strahl:** Reinigung füllt den getragenen oder daneben abgestellten Behälter per sichtbarem Partikelstrahl (Werkzeugspitze → Behälterstutzen); der Anzug speichert nichts, kein Ziel/Platz → Warnhinweis.
- [ ] Start-Depot neben dem Spawn-Punkt platzieren, mit Entlade-Chute: Behälter einhängen → Staub fliegt ins Silo.
- [ ] Interaktion am Depot: Frachtkapsel-Startanimation auslösen -> Credits ans `TeamWallet`, Funde als Taschengeld an den Finder.
- [ ] Basisbau-Modus: Neue Depots mit Geister-Vorschau überall auf dem Mond gegen Credits platzieren.

### Phase 4: LSD-Upgrade-Terminal & Werkzeug-Progression
- [ ] Interaktives Terminal am Depot erstellen, das ein Bevy-UI-Shop-Menü öffnet.
- [ ] Kauf von Werkzeug-Stufen (T1–T8) und Behälter-Upgrades (T1–T7) gemäß `design_wirtschaft_und_progression.md` implementieren, dazu Anzug-Nebenzweig (Sprungstiefel, Jetpack, Mag-Boots).
- [ ] Sichtbares Wechseln des Viewmodels in der Hand des Spielers je nach ausgerüstetem Werkzeug.

### Phase 5: Reinigungsfahrzeuge (Lunar Rovers)
- [ ] Fahrzeug-Framework (Spawnen am Pad, Ein-/Aussteige-Logik mit Kamerawechsel und Input-Umschaltung, automatischer Staub-Transfer am Depot) bauen.
- [ ] Die 7 Fahrzeuge aus `design_wirtschaft_und_progression.md` (Säule C) prozedural modellieren — beginnend mit Mäher und Golf-Cart, Bagger 288 zuletzt.
- [ ] Arcade-Fahrphysik auf dem Mond-Terrain (1/6 g, Kippen/Driften) + automatisches Großflächen-Putzen während der Fahrt.
- [ ] Spaß-Mechaniken der Tabelle umsetzen (Hupe & Nitro beim Golf-Cart, Hochglanz-Polieren beim Zamboni, Drift-Kombos beim Hover-Drifter).

### Phase 6: 16-Spieler Koop-Networking & UI-Polish
- [ ] Host/Join-Menü im Hauptmenü für bis zu 16 Spieler einbauen.
- [ ] Spieler-Avatare (prozedurale Raumanzüge mit farbigem Visier und Namensschild) für Mitspieler rendern.
- [ ] Delta-Synchronisation der `DustChunk`-Daten für neu beitretende und aktive Spieler sicherstellen.
- [ ] Vollständiges HUD (Fadenkreuz, Behälter-Füllstand, Credits, aktuelles Werkzeug, globaler Mond-Fortschrittsbalken, Tasten-Hilfe) finalisieren.

### Phase 7: Automatisierung, Fließbänder & Veredelung (Design-Dokument Säule D & E)
- [ ] **Entlade-Chutes** als Standard-Übergabe an Depots und Fahrzeugen + frei platzierbare Variante (150 ₡); Partikel-Wasserfall-Animation beim Entleeren.
- [ ] Fließbänder (1.500 ₡ / 10 m): volle Behälter rollen zur Basis, leere kehren zurück; Kurven und Steigungen.
- [ ] Roomba „Sweepy v0.1" (200 ₡) mit Batterie- und Ladestations-Logik (sanfte Wartung, keine Strafen).
- [ ] Staub-Katapult (Einweg-Druckbehälter als Projektil), Drohnen-Hangar „Dust-Bees" (Mini-Behälter), Thumper-Saugturm (Behälterwechsel am Turmfuß) und Rover-Wegpunkte (KI-Patrouillen) umsetzen.
- [ ] Veredelungs-Gebäude vor dem Verkauf bauen: Rüttelsieb, Sinter-Ofen, Helium-3-Zentrifuge; Multiplikatoren wirken am Verkauf.
- [ ] Meilenstein-Belohnungen (0,1 / 1 / 5 / 25 / 100 % sauber) inkl. Radar-Scanner, Tuning-Terminal und Öffnen der Fernseiten-Region (zweite echte `Region::at(...)` mit grünem Staub, 5× Wert).
- [ ] Offline-Fortschritt: Automatisierung läuft im Savegame nach (max. 8 h gedeckelt), Gutschrift mit Zusammenfassung bei Rückkehr.
- [ ] Endgame: Schwerlast-Frachtrampe, Weltraumlift und orbitaler „Helios-Lense"-Satellit.

---

## 7. Strikt einzuhaltende Coding-Richtlinien (Guardrails)

1. **Keine unvollständigen Platzhalter (`todo!()`):** Jedes System muss komplett ausprogrammiert sein.
2. **Bevy ECS Konventionen:**
   * Nutze klare System-Sets (`#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]`), um Reihenfolge-Konflikte zwischen Input, Physik, Staub-Abtrag und UI-Update zu vermeiden.
   * Vermeide `.unwrap()` in Laufzeit-Systemen; nutze stattdessen `let Ok(...) = query.get_single() else { return; };` (bzw. `.single()` je nach Bevy-Version).
   * Trenne Logik-Events (`CleanDustEvent`, `DepositDustEvent`, `PurchaseUpgradeEvent`) strikt von der Darstellung.
3. **Performance:** Aktualisiere die GPU-Textur eines `DustChunk` **nur** in dem Frame, in dem `dirty_render == true` ist, und setze das Flag sofort zurück.
4. **Selbstverifikation:** Führe nach jedem größeren Edit `cargo check` und `cargo clippy` aus und behebe sämtliche Compiler-Fehler und Borrow-Checker-Konflikte selbstständig, bevor du zum nächsten Schritt übergehst. Die vorhandenen 12 Unit-Tests (`cargo test`) müssen jederzeit grün bleiben.

---

## 8. Quellen & Lizenzen (Credits — Pflicht im Spiel!)
Die Geländedaten sind echte, gemeinfreie Messdaten; die Quellenzuschreibung gehört in einen Credits-Bildschirm:

* **Mond-Höhen:** LRO LOLA Gridded Data Record `ldem_16`, NASA PDS Geosciences Node (`pds-geosciences.wustl.edu`). NASA-Daten, gemeinfrei. Details: `assets/moon/README.md`.
* **Erd-Höhen:** NOAA ETOPO1 (Amante & Eakins 2009, NESDIS NGDC-24). US-Regierungswerk, gemeinfrei.
* **Erd-Klima (Wüste/Eis):** NASA Visible Earth, Blue Marble Next Generation, Reto Stöckli, NASA Earth Observatory. Gemeinfrei, **Quellenangabe erbeten** — nenne die NASA Earth Observatory in den Credits.
* **Seen/Flüsse** (bislang ungenutzt im Spiel): Natural Earth, 1:10m, naturalearthdata.com. Gemeinfrei.
* Details und Neu-Erzeugung: `assets/earth/README.md` und `assets/moon/README.md`.