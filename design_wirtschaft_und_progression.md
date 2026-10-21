# Design: Fortschritt & Wirtschaft (Progression Economy)

> **Randbedingungen:** Die Startregion wächst von 16 km² (Vorschau, 4×4 km) auf bis zu **50 km² (≈ 7×7 km)** — derselbe `Region::at(...)`-Ausschnitt auf dem echten Mond, nur größer gezogen (siehe Spezifikation Abschnitt 2.6). Staub lebt in **physischen Behältern** (Säule B); jede Logistik transportiert Behälter, Entladen läuft über **Chutes** (Entladetrichter, Säule D/E). Alle Wirtschaftszahlen stehen hier an einem Ort.
>
> **Dieses Dokument ist die verbindliche Quelle für alle Wirtschafts- und Progression-Zahlen** (Werkzeuge, Rucksäcke, Fahrzeuge, Gebäude, Meilensteine). Die `AGENTS.md`-Tabellen dazu sind zugunsten dieses Dokuments gekürzt.

Dieses Progressions- und Wirtschaftssystem ist darauf ausgelegt, eine **50 km² große Mondkarte** über 15 bis 25 Spielstunden hinweg spannend zu halten. Es kombiniert exponentielles Wachstum mit spürbaren Gameplay-Wechseln: Was als absurde Sisyphus-Arbeit mit einer Zahnbürste beginnt, eskaliert über drifttaugliche Spaß-Fahrzeuge bis hin zur vollautomatisierten Mega-Fabrik.

---

## 1. Wirtschafts-Mathematik & Core-Loop (Die Ökonomie)

Damit sich das Spiel weder zu schnell durchgespielt noch wie eine zähe Wand anfühlt, nutzt die Wirtschaft drei ineinandergreifende Hebel: **Volumen**, **Kompression** und **Veredelung**.

* **Basis-Einheit:** 1 Quadratmeter ($1\text{ m}^2$) Mondoberfläche ist im Schnitt mit **10 Litern (L) Regolith-Staub** bedeckt.
* **Basis-Verkaufswert:** $1\text{ L Staub} = 1\text{ LSD-Credit (₡)}$.
* **Zufallsfunde (Dopamin-Drops):** Beim Abtragen von Staub besteht pro Quadratmeter eine Basis-Chance von **2 %**, vergrabenen Weltraumschrott oder seltene Mineralien freizulegen (z. B. *Apollo-Goldfolie* = 250 ₡, *Helium-3-Knolle* = 1.500 ₡, *Versteinerter Alien-Gartenzwerg* = 10.000 ₡).
* **Tiefen-Funde:** In Kraterinnenenen und an Kraterböden (über die echte Höhe aus `moon_data` erkennbar) verdoppelt sich die Fund-Chance und die selteneren Funde werden wahrscheinlicher — die echten Höhendaten werden damit zur Gameplay-Quelle.
* **Bezirks-Bonus (Zusammenhangs-Bonus):** Liter Staub aus Zellen, die an bereits saubere Flächen angrenzen, bringen **+25 %** Wert. Belohnt gründliches Putzen statt Rosinenpflücken und macht den Fortschrittsbalken strategisch.
* **Koop-Wallet-Regel:** Das LSD-Budget (`TeamWallet`) ist geteilt — aber **Zufallsfunde gehen als privates Taschengeld** an den Finder. So bleibt Einzelleistung sichtbar, während Fahrzeuge und Basen aus dem gemeinsamen Topf bezahlt werden.
* **Kosten-Skalierung für wiederholbare Upgrades:**

$$\text{Preis}(\text{Stufe } n) = \text{Basispreis} \times 1{,}35^{(n - 1)}$$

---

## 2. Säule A: Hand-Werkzeuge (Der aktive Reinigungs-Loop)

Jedes Werkzeug verändert nicht nur die Zahlen, sondern auch das Spielgefühl (Steuerung, Rückstoß in der Mondgravitation, Soundeffekte).

| Stufe | Werkzeug | Kosten (₡) | Radius (m) | Abtrag (L/s) | Spezialeffekt & Spielgefühl |
| --- | --- | --- | --- | --- | --- |
| **T1** | **Standard-Zahnbürste** | 0 | 0,15 m | 2 L/s | Muss manuell durch Mausbewegung geschrubbt werden. |
| **T2** | **Kehrblech & Besen** | 50 | 0,6 m | 12 L/s | Schiebt Staub bei jedem Schwung als kleine physikalische Welle vor sich her. |
| **T3** | **Akku-Tischsauger "DustBuster"** | 350 | 1,2 m | 40 L/s | Kontinuierliches Saugen per Knopfdruck; saugt lose Kleinteile automatisch ein. |
| **T4** | **Industrie-Laubbläser (Umgekehrt)** | 1.800 | 2,8 m | 150 L/s | **Low-G-Rückstoß:** Wenn man springt und nach unten zielt, schwebt man wie mit einem Jetpack. |
| **T5** | **Rotierender Doppel-Bürstenkopf** | 8.500 | 4,5 m | 500 L/s | Zieht den Spieler leicht nach vorne (+25 % Lauftempo beim Putzen). |
| **T6** | **Elektrostatischer Magnet-Rechen** | 42.000 | 8,0 m | 2.200 L/s | Zieht Staub im Umkreis wie ein Magnet an; verdoppelt die Chance auf seltene Schrott-Funde. |
| **T7** | **Plasma-Schneidbrenner** | 190.000 | 14,0 m | 9.500 L/s | Verglast den Boden sofort (hinterlässt spiegelglatte, glänzende Pfade für schnelleres Laufen). |
| **T8** | **Taschen-Singularität "Black Hole Mk. I"** | 1.000.000 | 25,0 m | 50.000 L/s | Verschießt eine langsame Gravitationskugel, die eine ganze Kraterhälfte im Vorbeiflug leersaugt. |

**Auffang-Regel (Schnittstelle zu Säule B):** Abgetragener Staub verschwindet nie im Nichts: Er fliegt als sichtbarer, hörbarer Strahl vom Werkzeug in den **Stutzen eines Behälters** — getragen in der freien Hand oder daneben auf dem Boden abgestellt. Der Anzug selbst speichert keinen Staub. Ist kein Behälter in Reichweite oder alles voll, stoppt das Aufnehmen mit Warnhinweis.

---

## 3. Säule B: Staubbehälter, Kompression & Anzug-Mobilität

Es gibt **kein abstraktes Inventar**: Der Staub sitzt in **physischen Behältern**. Behälter werden getragen, können abgestellt, gestapelt, gekickt, auf Fahrzeuge verladen und ab dem Mid-Game auf Fließbändern zur Basis geschickt werden.

**Behälter-Grundregeln (immer aktiv):**
* Behälter sind **Hand-Geräte**: Tragegriff oben, halbtransparenter Körper mit Glas-Sichtfenster und LED-Füllstand-Leiste — der Füllstand ist immer ablesbar, ohne ein Menü zu öffnen.
* Beim Putzen steht der Behälter **neben dem Spieler** (oder hängt in der freien Hand) und fängt den Auffang-Strahl ein (siehe Säule A).
* Tragen: **ein Behälter in der Hand**; das Gewicht bremst das Laufen (Low-G macht es erträglich).
* Volle Behälter: abstellbar, stapelbar, kickbar — und werden **zum Sale Point getragen**. Reinigung stoppt mit Warnhinweis, wenn kein Behälter mehr Platz hat; dann leeren Behälter greifen oder vollen zur Abgabe bringen.
* **Alles läuft über Behälter:** Jede Logistik (Chute, Fließband, Rohrpost, Katapult, Fahrzeug-Verladung) transportiert den Staub als Behälter — niemals als abstrakte Zahl. Ein alter Eimer bleibt nützlich, auch wenn man längst bessere Behälter besitzt.

| Stufe | Behälter | Kosten (₡) | Kapazität (L) | Kompressions-Bonus & Feature |
| --- | --- | --- | --- | --- |
| **T1** | **Ziploc-Gefrierbeutel (in der Hand)** | 0 | 20 L | Keine Kompression. Reicht für 2 Quadratmeter. |
| **T2** | **Baumarkt-Eimer mit Tragegriff** | 40 | 100 L | Voll abstellbar und kickbar — der Klassiker: vollen Eimer das Gefälle hinunter zum Depot treten. |
| **T3** | **Vakuum-Kanister** | 300 | 500 L | Komprimiert Staub 2:1 (zählt als 1.000 L effektives Volumen); saugt beim Tragen selbstständig auf. |
| **T4** | **Hydraulische Ziegel-Presse (Station + Presse-Behälter)** | 2.000 | 2.500 L | Presst Staub live zu **Mond-Ziegeln** (+25 % Verkaufswert). Ziegel sind stapelbare Objekte — Rampen bauen inklusive! |
| **T5** | **Kryo-Kompressor-Tank** | 12.000 | 15.000 L | Flüssigstickstoff-Kühlung verdichtet Staub extrem; +50 % Verkaufswert. |
| **T6** | **Quanten-Behälter** | 75.000 | 100.000 L | Wiegt praktisch nichts; verbundene Spieler im Koop können in denselben geteilten Behälter saugen. |
| **T7** | **Teleporter-Behälter** | 450.000 | Unendlich | Beamt seinen Inhalt gegen 5 % Gebühr sofort live in das nächstgelegene Depot. |

**Anzug-Mobilitäts-Upgrades (Nebenzweig):**

* **Feder-Sprungstiefel (250 ₡):** Erhöht die Sprunghöhe in der Mondgravitation um 200 % (Bunnyhop-Mechanik).
* **RCS-Schubdüsen / Jetpack (2.500 ₡):** Erlaubt kontrolliertes Gleiten über große Krater.
* **Mag-Boots (6.000 ₡):** Erlaubt das Laufen an senkrechten Kraterwänden und auf dem Kopf in Höhlen.

---

## 4. Säule C: Spaß-Fahrzeuge (Koop & Physik-Chaos)

Fahrzeuge sind das Herzstück ab dem Mid-Game. Sie haben eigene Tanks, eigene Upgrades und basieren auf einer übertriebenen 1/6-Gravitations-Fahrphysik.

| Fahrzeug | Kosten (₡) | Sitzplätze | Reinigungs-Breite | Tank (L) | Spaß-Mechanik & Besonderheit |
| --- | --- | --- | --- | --- | --- |
| **1. Aufsitzmäher "Lawnmaster Lunar"** | 1.500 | 1 | 3,5 m | 2.000 L | Macht kabinenintern tucksende Rasenmäher-Geräusche; kippt bei scharfen Kurven leicht auf zwei Räder. |
| **2. Mond-Golf-Cart mit Schneepflug** | 6.500 | 4 | 6,0 m | 8.000 L | **Koop-Liebling:** Fahrer lenkt, Beifahrer bedienen seitliche Saugschläuche. Hat eine laute Hupe und Nitro-Boost. |
| **3. Eisflächen-Zamboni "Crater-Smoother"** | 28.000 | 2 | 10,0 m | 40.000 L | Extrem schwer und träge, poliert den Boden auf Hochglanz (+100 % Fahrtempo für alle Fahrzeuge auf polierten Flächen). |
| **4. Hover-Drifter "Slipstream 9000"** | 110.000 | 2 | 14,0 m | 150.000 L | Null Bodenreibung! Wird wie in einem Arcade-Racer quer durch Krater gedriftet. Je länger der Drift-Kombo-Multiplikator, desto höher die Saugkraft. |
| **5. Spinnen-Mech "Arachno-Vac"** | 380.000 | 1 | 18,0 m | 500.000 L | Hat 6 hydraulische Beine statt Räder; klettert mühelos über Felsen und steile Kraterwände, stampft Brocken klein. |
| **6. Turbinen-Monstertruck "Dust Yeeter"** | 1.200.000 | 4 | 25,0 m | Kein Tank | Saugt nicht ein, sondern **schleudert** den Staub in einer gigantischen ballistischen Parabel bis zu 500 Meter weit direkt in einen Depot-Trichter. |
| **7. Schaufelradbagger "Bagger 288 Moon"** | 8.500.000 | 16 (Basis) | 60,0 m | 10.000.000 L | Eine fahrende Festung mit eigenem Raketensilo, Shop-Terminal und Drohnen-Hangar an Bord. Frisst ganze Landstriche. |

---

## 5. Säule D: Automatisierung, Fließbänder & Faule Ingenieure

Damit man die 50 km² nicht komplett selbst abfahren muss, baut man schrittweise ein autonomes Reinigungsnetzwerk auf. **Sanfte Wartung:** Mobile Helfer (Roombas, Drohnen) haben Batterien und docken selbstständig an Ladestationen an — ohne Strafen; ohne Strom stehen sie nur still.

1. **Panzertape-Roomba "Sweepy v0.1" (Kosten: 200 ₡ pro Stück):**
   Fährt stur geradeaus, bis er gegen einen Stein stößt, dreht sich zufällig und fährt weiter. Hat nur 50 L Tank, piept traurig, wenn er feststeckt, und trägt Wackelaugen. Muss regelmäßig an der Ladestation andocken.
2. **Entlade-Chute (Standard an jedem Depot und Reinigungsfahrzeug; frei platzierbare Variante: 150 ₡):**
   Das universelle Übergabestück der Logistik: Ein Trichter mit Klemmhalterung — Behälter einhängen, Hebel umlegen, der Staub rauscht als grauer Wasserfall ins Silo, auf das Fließband, in den Fahrzeugtank oder auf den Katapult-Teller. Chutes sind die einzige Art, wie Behälter ihren Inhalt loswerden.
3. **Fließband (Kosten: 1.500 ₡ pro 10 m Band):**
   Rollt **volle Behälter** zur Basis und bringt **leere Behälter zurück** an die Arbeitsfront (auch Kurven und leichte Steigungen). Das entspannteste Bild des Spiels: eine Schlange voller Eimer, gemächlich den Krater hangabwärts.
4. **Staub-Katapult / Ballistischer Werfer (Kosten: 3.500 ₡):**
   Kleine Zwischenstation: Bots und Spieler kippen Behälter in den Werfer-Teller; das Katapult feuert alle 10 Sekunden einen **gepressten Einweg-Druckbehälter** quer über die Karte in Richtung Hauptbasis, wo ein Auffangnetz ihn einängt. (Vorsicht: Kann Spieler aus der Luft treffen!)
5. **Schwarm-Drohnen-Hangar "Dust-Bees" (Kosten: 15.000 ₡):**
   Beherbergt 10 fliegende Drohnen, die selbstständig im Umkreis von 250 Metern schmutzige Stellen suchen, absaugen und mit ihren mitgebrachten Mini-Behältern zum Hangar zurückbringen. Drohnen laden im Hangar.
6. **Pneumatische Vakuum-Rohrpost (Kosten: 100 ₡ pro 10 m Rohr):**
   Das schnelle Endgame-Upgrade des Fließbands: verbindet Feld-Depots mit der Hauptbasis. Man sieht die **Behälter durch die durchsichtigen Glasröhren** rasen (und Spieler, die hineinspringen!), mit 150 km/h.
7. **Stationärer Thumper-Saugturm (Kosten: 65.000 ₡):**
   Schlägt rhythmisch auf den Mondboden. Durch die niedrige Gravitation schwebt der Staub im Umkreis von 150 Metern nach oben und wird vom Turm eingesaugt; unten hängt ein Behälterregal, in dem die vollen Behälter ausgewechselt werden.
8. **Autonome Rover-Wegpunkte (Kosten: 120.000 ₡):**
   Erlaubt es, gekaufte Fahrzeuge (wie den Zamboni) mit einem KI-Kern auszustatten und auf feste Patrouillen-Routen zu schicken.
9. **Orbitaler Reinigungs-Satellit "Helios-Lense" (Kosten: 15.000.000 ₡ – Endgame):**
   Brennt mit einem gebündelten Sonnenstrahl aus dem Orbit kontinuierlich Bahnen in die Mondoberfläche und befördert die verdampften Mineralien direkt auf das Bankkonto.

**Offline-Fortschritt:** Die Automatisierung (Fließbänder, Roombas, Drohnen, Thumper, Satellit) arbeitet im Savegame weiter, während man offline ist — bei der nächsten Session wird der angelaufene Fortschritt mit einer kleinen Zusammenfassung („Sweepy hat 3 Stunden treu gedient") gutgeschrieben, auf maximal 8 Stunden Offline-Zeit gedeckelt, damit Active-Play immer die Hauptrolle behält.

---

## 6. Säule E: Basisbau, Veredelung & Logistik (Der Geld-Multiplikator)

Rohstaub bringt nur 1 ₡ pro Liter. Wer reich werden will, baut eine Verarbeitungs-Pipeline vor das Raketensilo:

| Gebäude | Kosten (₡) | Funktion & Wirtschafts-Effekt |
| --- | --- | --- |
| **Mobiles Feld-Depot (Pod)** | 500 | Kleiner Abgabepunkt mit faltbarem Entlade-Chute und einer Billig-Rakete (Kapazität: 1.000 L, Flugzeit: 20 Sek.). |
| **Sale Point (Hauptverkaufsstelle)** | Teil der Start-Basis | Der zentrale Abgabeort: Behälter in den Entlade-Chute, Staub rauscht ins Silo, Liter wandern aufs Konto. Hier stehen auch LSD-Terminal und Shop. |
| **Rüttelsieb-Anlage** | 4.000 | Filtert vor dem Verkauf automatisch Schrott und Erze heraus (**+50 % Gesamterlös**). |
| **Sinter-Ofen (Ziegelwerk)** | 22.000 | Schmelzt Staub mit Solarenergie zu Keramikplatten (**2,5× Verkaufswert**). |
| **Helium-3 Zentrifuge** | 140.000 | Extrahiert seltenen Fusionsbrennstoff aus dem Regolith (**6× Verkaufswert**). |
| **Schwerlast-Frachtrampe** | 90.000 | Ersetzt kleine Raketen durch ein massives Cargo-Shuttle (Kapazität: 500.000 L pro Flug). |
| **Orbitaler Weltraumlift (Endgame)** | 25.000.000 | Ein endloses Kabel in den Orbit. Kein Raketen-Cooldown mehr – **Echtzeit-Verkauf** von Millionen Litern pro Sekunde mit **10× Wert-Multiplikator**. |

---

## 7. Prestige- & Meilenstein-Belohnungen (LSD-Unternehmensränge)

Immer wenn ein bestimmter Prozentsatz der gesamten Mondkarte (50 km²) sauber ist, schickt das *Lunar Surface Department (LSD)* einen Bonus:

* **0,1 % sauber – "Praktikant des Monats":** Schaltet den Radar-Scanner frei (zeigt schmutzige Nester auf der Minimap).
* **1,0 % sauber – "Krater-Kehrwart":** Schaltet das Fahrzeug-Tuning-Terminal frei (Lackierungen, Hupen, Unterbodenbeleuchtung).
* **5,0 % sauber – "Regolith-Baron":** Schaltet die Rohrpost-Logistik und Drohnen frei.
* **25,0 % sauber – "Dunkle-Seite-Pionier":** Öffnet die **Rückseite des Mondes** (radioaktiver grüner Staub mit 5-fachem Basiswert, erfordert Blei-Anzug-Upgrade). *Technischer Hinweis: Das ist schlicht eine zweite `Region::at(...)` auf der echten Mond-Fernseite — die echten LOLA-Höhen der Fernseite machen den Look kostenlos anders.*
* **100 % sauber – "Der Mond glänzt":** Der Mond reflektiert das Sonnenlicht nun so stark wie ein Spiegel. Vor dem Start-Depot steht ab jetzt das **goldene Zahnbürsten-Denkmal**, und die Frachtraketen starten mit Konfetti-Schweif. (Ein späterer New-Game+-Modus ist bewusst noch nicht beschlossen.)

---

## 8. Technische Anbindung (an Mond-Daten & DustChunks)

* **Liter ↔ Zellen:** 1 m² trägt 10 L Staub. Empfohlene Zellgröße der `DustChunk`s: **0,5 m × 0,5 m** (= 0,25 m² = **2,5 L pro voller Zelle**, `u8`-Füllstand 0–255 → ~0,01-L-Granularität). Die Zahnbürste (Radius 0,15 m) trifft dann 1–2 Zellen — gewollt mühsam.
* **Speicher:** 50 km² wären bei 0,5-m-Zellen ~200 Mio. Zellen (~200 MB `u8`). Deshalb **lazy Allokation**: Ein Chunk-Allokation nur, wenn er zum ersten Mal gereinigt wird (Default = voller Staub, nicht gespeichert). `GlobalCleaningProgress` zählt abgeräumte Liter als `u64` — passt locker (500 Mio. L gesamt).
* **Staubbehälter als Entities:** `DustContainer { tier, current_liters, max_liters }` als physische Entity mit Zuständen: getragen (in der Hand, Gewicht bremst) / abgestellt / auf Fließband / im Rohrpost / im Katapult-Teller / im Chute. Sichtbares Sichtfenster + LED-Füllstand = reine Renderfrage aus `current_liters`.
* **Auffang-Strahl:** Der `CleanDustEvent`-Ertrag wird nicht in ein Spieler-Inventar geschrieben, sondern per Partikelstrahl (Start: Werkzeugspitze, Ziel: Behälterstutzen des nächsten Behälters in Reichweite — getragen oder abgestellt) in den Behälter gefüllt. Kein Ziel → Aufnahme stoppt mit Warnhinweis.
* **Chute als Interaktions-Entity:** `DustChute` (an Depot, Fahrzeug oder frei platziert) hat einen Einhänge-Slot; Interaktion = Behälter einhängen + umkippen → Liter fließen (animierter Partikel-Wasserfall) in das angeschlossene Ziel (Silo/Tank/Band). Nur über Chutes wechseln Behälter ihren Inhalt — damit ist der Transferpfad immer sichtbar und synchronisierbar.
* **Fahrzeug-Tank fest verbaut:** `VehicleInventory` ist **kein** `DustContainer` — der Tank sitzt fest im Fahrzeug und wird ausschließlich über das Fahrzeug-Chute gefüllt. Behälter → Tank geht also nie direkt in der Hand, sondern immer über die Station.
* **Fließbänder & Rohrpost:** Band-Segmente als Entities mit Richtung; Behälter auf dem Band bewegen sich pro Frame entlang der Segmente (einfach stateless pro Segment, keine globale Logik nötig). Die Rohrpost ist der speziellere Fall: Punkt-zu-Punkt-Verbindungen mit festen Enden, Behälter-Entities fliegen als Projektil.
* **Geld & Werte:** Alle Multiplikatoren (Veredelung, Kompression, Bezirks-Bonus, Fernseiten-Bonus) multiplizieren am Verkauf, nicht beim Sammeln — so bleibt `1 L = 1 ₡` als Basis überall erhalten.
* **Balancing-Modul `economy.rs`:** Sämtliche Zahlen dieses Dokuments (Preise, Raten, Radien, Kapazitäten, Fund-Werte, Bonus-Sätze, Offline-Deckel) leben als Konstantentabelle in **einer** Datei — Balancing ohne Logikänderung. Keine magischen Zahlen in den Systemen.
