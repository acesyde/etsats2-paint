## Exportgrößen und Fehler

size-mb = { $value } MB
size-kb = { $value } KB
size-about = etwa { $value }
export-stopped = der Export wurde unerwartet abgebrochen

## Projekte speichern und öffnen

file-save-failed-title = Speichern nicht möglich
file-save-failed = { $file } konnte nicht gespeichert werden.
file-save-failed-reason = { $file } konnte nicht gespeichert werden ({ $reason }). Ihre Änderungen sind weiterhin geöffnet.
file-open-failed-title = Projekt kann nicht geöffnet werden
file-restore-failed-title = Projekt kann nicht wiederhergestellt werden
file-recovered-copy = Die wiederhergestellte Kopie
file-not-a-project = { $file } ist kein TruckPaint-Projekt.
file-damaged = { $file } ist beschädigt und kann nicht geöffnet werden.
file-newer-version = { $file } wurde mit einer neueren Version von TruckPaint erstellt.
file-io = { $file } konnte nicht gelesen oder geschrieben werden ({ $reason }).

## Dateidialoge

dialog-open-project = Projekt öffnen
dialog-save-project = Projekt speichern
dialog-export-texture = Textur exportieren
filter-project = TruckPaint-Projekt
filter-images = Bilder

## Startbildschirm

home-recovered = Wiederhergestellte Projekte
home-recovered-hint = TruckPaint wurde unerwartet beendet. Diese Projekte enthielten nicht gespeicherte Änderungen.
home-never-saved = Nie gespeichert
home-recovery-damaged = Beschädigte Wiederherstellungskopie
home-recovery-damaged-hint = Diese Kopie kann nicht wiederhergestellt werden.
home-recovered-item = { $name } wiederhergestellt
home-restore = Wiederherstellen
home-discard = Verwerfen
app-tagline = Lackierungseditor für Euro Truck Simulator 2 und American Truck Simulator
home-new-project = Neues Projekt
home-new-project-tip = Neues Lackierungsprojekt erstellen ({ $shortcut })
home-preferences = Einstellungen
home-recent = Zuletzt verwendete Projekte
home-recent-empty = Keine zuletzt verwendeten Projekte
home-recent-empty-hint = Geöffnete Projekte werden hier angezeigt. Erstellen Sie zunächst ein neues Projekt.
home-recent-missing-item = { $name } (Datei nicht gefunden)
home-remove-recent = Aus Liste entfernen
home-file-not-found = Datei nicht gefunden
home-open-recent = { $path } öffnen
home-file-moved = Die Datei dieses Projekts wurde verschoben oder gelöscht.

## Dialoge

unsaved-title = Änderungen an „{ $name }“ vor dem Schließen speichern?
unsaved-hint = Ihre Änderungen gehen verloren, wenn Sie sie nicht speichern.
button-dont-save = Nicht speichern
button-cancel = Abbrechen
button-ok = OK
button-create = Erstellen
button-done = Fertig
button-close = Schließen
new-project-step-name = Name
new-project-name = Projektname
prefs-interface = Oberfläche
prefs-ui-scale = Skalierung der Oberfläche
prefs-text-size = Textgröße
prefs-scale-hint = Änderungen werden sofort übernommen. 100 % entspricht dem Skalierungsfaktor Ihres Bildschirms.
prefs-canvas = Arbeitsfläche
prefs-grid-spacing = Rasterabstand
prefs-reset = Auf Standard zurücksetzen
shortcuts-temporary-hand = Vorübergehend Hand-Werkzeug
shortcuts-hold-space = Leertaste gedrückt halten
about-version = Version { $version }
about-tagline = Lackierungseditor für Euro Truck Simulator 2 und American Truck Simulator.

## Dialog „Textur exportieren“

export-done = { $file } exportiert
export-failed-title = Export nicht möglich
export-failed = { $file } konnte nicht geschrieben werden ({ $reason }).
export-preview = Exportvorschau
export-info = { $size } × { $size } px · { $format } · { $bytes }
export-progress = { $file } wird exportiert…
export-start = Exportieren…
export-format = Format
export-dds-bc3 = BC3 / DXT5 (empfohlen)
export-dds-rgba = Unkomprimiertes RGBA
export-dds-encoding = DDS-Kodierung
export-size = Größe
export-background = Hintergrund
export-transparent = Transparent
export-background-color = Hintergrundfarbe
prefs-language = Sprache
file-dev-format = { $file } verwendet ein Entwicklungsformat, das diese Version nicht öffnen kann.

## Vehicle packages

pkg-install-failed = { $file } konnte nicht installiert werden: { $reason }
pkg-no-library = es gibt keinen Ordner für die Fahrzeugbibliothek.
pkg-io = sie konnte nicht gelesen oder geschrieben werden ({ $reason }).
pkg-not-a-zip = es ist kein Fahrzeugpaket.
pkg-no-manifest = es enthält kein vehicle.json-Manifest.
pkg-bad-manifest = sein Manifest ist ungültig ({ $reason }).
pkg-newer-format = es wurde für eine neuere TruckPaint-Version erstellt.
pkg-bad-id = seine ID „{ $id }“ ist ungültig.
pkg-no-main-texture = es hat keine Haupttextur.
pkg-bad-game-path = sein Spielpfad „{ $path }“ ist ungültig (Wörter aus a–z, 0–9 und _, durch Punkte getrennt).
pkg-bad-game-id = die Spiel-ID „{ $id }“ ist ungültig (Wörter aus a–z, 0–9 und _, durch Punkte getrennt).
pkg-missing-game-ids = „{ $texture }“ hat keine Spiel-ID.
pkg-duplicate-part = zwei Texturen haben die ID „{ $id }“.
pkg-duplicate-game-id = die Spiel-ID „{ $id }“ ist doppelt aufgeführt.
pkg-bad-size = die Textur „{ $texture }“ hat eine ungültige Größe ({ $size }).
pkg-unsafe-path = es enthält einen unsicheren Pfad ({ $path }).
pkg-too-large = es ist zu groß.
pkg-missing-template = die Vorlage von „{ $texture }“ fehlt.
pkg-bad-template = die Vorlage von „{ $texture }“ ist kein lesbares PNG- oder SVG-Bild.
pkg-template-too-large = die Vorlage von „{ $texture }“ ist zu groß.

## Vehicle library and updates

filter-packages = Fahrzeugpakete
vehicles-search = Fahrzeuge suchen
vehicles-all-games = Alle Spiele
vehicles-game-filter = Spiel
vehicles-all-kinds = Lkw und Anhänger
vehicles-kind-filter = Art
vehicles-truck = Lkw
vehicles-trailer = Anhänger
vehicles-installed = { $name } { $version } installiert
vehicles-empty = Kein Fahrzeug installiert
vehicles-empty-hint = Fahrzeugpakete (.tpv) enthalten die Vorlagen eines Lkw oder Anhängers. Installieren Sie eines, um eine Lackierung zu beginnen.
vehicles-details = Spielversionen { $versions } · { $main } · Zubehör: { $accessories }
vehicles-version = Version { $version }
vehicles-remove-version = { $name } { $version } entfernen
vehicles-remove-confirm = { $name } aus der Bibliothek entfernen? Projekte behalten ihre Vorlagen.
vehicles-remove = Entfernen
vehicles-install = Installieren…
vehicles-install-sample = Beispielfahrzeuge installieren
update-versions = { $name }: Version { $from } → { $to }
update-replaced = { $name }: Vorlage ersetzt
update-layout-changed = { $name }: Layout geändert, Grafik prüfen
update-resized = { $name }: Größe { $old } → { $new } px, Grafik skaliert
update-new-textures = Neu in dieser Version:
update-removed = { $name }: in dieser Version nicht mehr enthalten, Grafik bleibt erhalten
update-artwork-kept = Ihre Grafik bleibt erhalten. Sie können die Aktualisierung rückgängig machen.
update-apply = Aktualisieren

## New Project wizard

new-project-step-vehicle = Fahrzeug
new-project-no-vehicles-hint = Noch kein Fahrzeug? Probieren Sie den Beispiel-Lkw aus oder installieren Sie ein Paket.
new-project-textures = Texturen
button-next = Weiter
button-back = Zurück

## Fleet

add-vehicle-none = Kein weiteres Fahrzeug dieses Spiels ist installiert. Installieren Sie ein Paket, um eines hinzuzufügen.
add-vehicle-add = Hinzufügen
fleet-error-other-game = Dieses Fahrzeug gehört zu einem anderen Spiel als das Projekt.
fleet-error-already-there = Dieses Fahrzeug ist bereits im Projekt.
fleet-error-textures = Wählen Sie mindestens eine Haupttextur dieses Fahrzeugs.
fleet-error-version = Die installierte Version unterscheidet sich von der des Projekts.
fleet-error-unknown = Dieses Fahrzeug ist nicht im Projekt.
textures-title = Texturen von { $name }
textures-missing-version = Version { $version } dieses Fahrzeugs ist nicht installiert. Aktualisieren Sie seine Vorlage, um die Texturen zu ändern.
textures-remove-confirm = Die Gestaltung von { $textures } wird mit ihnen entfernt.
textures-remove = Entfernen
textures-apply = Anwenden
remove-vehicle-confirm = { $name } und seine Gestaltung aus dem Projekt entfernen?
