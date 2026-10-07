## Gemeinsame Wörter, relative Zeitangaben und Einstellungen.

language-system = Systemsprache

time-just-now = Gerade eben
time-ago-minutes = { $count ->
    [one] vor { $count } Minute
   *[other] vor { $count } Minuten
 }
time-ago-hours = { $count ->
    [one] vor { $count } Stunde
   *[other] vor { $count } Stunden
 }
time-ago-days = { $count ->
    [one] vor { $count } Tag
   *[other] vor { $count } Tagen
 }
time-yesterday = Gestern
time-ago-months = { $count ->
    [one] vor { $count } Monat
   *[other] vor { $count } Monaten
 }
time-ago-years = { $count ->
    [one] vor { $count } Jahr
   *[other] vor { $count } Jahren
 }

## Bedienelemente

mixed = Gemischt
swatch-fill-tip = Fläche (X zum Wechseln)
swatch-stroke-tip = Kontur (X zum Wechseln)
picker-saturation-value = Sättigung und Helligkeit
picker-opacity = Deckkraft der Farbe
gradient-bar = Verlaufsleiste
gradient-stop = Farbregler { $index } bei { $location } %
panel-close-named = { $title } schließen
step-of = Schritt { $step } von { $total } – { $title }
